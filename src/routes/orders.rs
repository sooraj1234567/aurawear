use axum::{Extension, Json, extract::Path};
use mongodb::{bson::{doc, oid::ObjectId, Document}, Collection, Database};
use serde_json::{json, Value};
use futures_util::StreamExt;

pub async fn get_orders(Extension(db): Extension<Database>) -> Json<Value> {
    let col: Collection<Document> = db.collection("orders");
    let mut cursor = col.find(doc!{}).await.unwrap();
    let mut list = Vec::new();
    while let Some(Ok(d)) = cursor.next().await { list.push(d); }
    Json(json!({"success":true, "orders": list}))
}

pub async fn get_order_by_id(Extension(db): Extension<Database>, Path(id): Path<String>) -> Json<Value> {
    let col: Collection<Document> = db.collection("orders");
    if let Ok(oid) = ObjectId::parse_str(&id) {
        if let Ok(Some(o)) = col.find_one(doc!{"_id": oid}).await {
            return Json(json!({"success":true, "order": o}));
        }
    }
    Json(json!({"success":false, "error":"not found"}))
}

pub async fn create_order(Extension(db): Extension<Database>, Json(body): Json<Value>) -> Json<Value> {
    let orders: Collection<Document> = db.collection("orders");
    let products: Collection<Document> = db.collection("products");
    let items = body.get("items").and_then(|v| v.as_array()).cloned().unwrap_or_default();

    // Check stock
    for item in &items {
        let pid = item.get("_id").or(item.get("id")).and_then(|v| v.as_str()).unwrap_or("");
        let qty = item.get("qty").or(item.get("quantity")).and_then(|v| v.as_i64()).unwrap_or(1) as i32;
        if let Ok(oid) = ObjectId::parse_str(pid) {
            if let Ok(Some(p)) = products.find_one(doc!{"_id": oid}).await {
                let stock = p.get_i32("stock").unwrap_or(p.get_i64("stock").unwrap_or(10) as i32);
                if stock < qty {
                    return Json(json!({"success":false, "error": format!("{} out of stock! Only {} left", p.get_str("name").unwrap_or("Product"), stock)}));
                }
            }
        }
    }
    // Decrease stock
    for item in &items {
        let pid = item.get("_id").or(item.get("id")).and_then(|v| v.as_str()).unwrap_or("");
        let qty = item.get("qty").or(item.get("quantity")).and_then(|v| v.as_i64()).unwrap_or(1) as i32;
        if let Ok(oid) = ObjectId::parse_str(pid) {
            let _ = products.update_one(doc!{"_id": oid}, doc!{"$inc": {"stock": -qty}}).await;
        }
    }

    let mut doc = Document::new();
    if let Some(obj) = body.as_object() {
        for (k,v) in obj { doc.insert(k.clone(), mongodb::bson::to_bson(v).unwrap()); }
    }
    doc.insert("status", body.get("status").and_then(|v| v.as_str()).unwrap_or("pending").to_string());
    doc.insert("payment_status", body.get("payment_status").and_then(|v| v.as_str()).unwrap_or("PAID").to_string());
    doc.insert("created_at", chrono::Utc::now().to_rfc3339());

    match orders.insert_one(doc).await {
        Ok(r) => Json(json!({"success":true, "id": r.inserted_id})),
        Err(e) => Json(json!({"success":false, "error": e.to_string()}))
    }
}

pub async fn update_status(Extension(db): Extension<Database>, Path(id): Path<String>, Json(body): Json<Value>) -> Json<Value> {
    let col: Collection<Document> = db.collection("orders");
    if let Ok(oid) = ObjectId::parse_str(&id) {
        let status = body.get("status").and_then(|v| v.as_str()).unwrap_or("pending");
        let _ = col.update_one(doc!{"_id": oid}, doc!{"$set": {"status": status}}).await;
    }
    Json(json!({"success":true}))
}

pub async fn update_tracking(Extension(db): Extension<Database>, Path(id): Path<String>, Json(body): Json<Value>) -> Json<Value> {
    let col: Collection<Document> = db.collection("orders");
    if let Ok(oid) = ObjectId::parse_str(&id) {
        let courier = body.get("courier").and_then(|v| v.as_str()).unwrap_or("");
        let tracking = body.get("tracking_number").and_then(|v| v.as_str()).unwrap_or("");
        let _ = col.update_one(doc!{"_id": oid}, doc!{"$set": {"courier": courier, "tracking_number": tracking, "status": "shipped"}}).await;
    }
    Json(json!({"success":true}))
}

pub async fn update_note(Extension(db): Extension<Database>, Path(id): Path<String>, Json(body): Json<Value>) -> Json<Value> {
    let col: Collection<Document> = db.collection("orders");
    if let Ok(oid) = ObjectId::parse_str(&id) {
        let note = body.get("note").and_then(|v| v.as_str()).unwrap_or("");
        let _ = col.update_one(doc!{"_id": oid}, doc!{"$set": {"internal_note": note}}).await;
    }
    Json(json!({"success":true}))
}

pub async fn update_payment_status(Extension(db): Extension<Database>, Path(id): Path<String>, Json(body): Json<Value>) -> Json<Value> {
    let col: Collection<Document> = db.collection("orders");
    if let Ok(oid) = ObjectId::parse_str(&id) {
        let ps = body.get("payment_status").and_then(|v| v.as_str()).unwrap_or("PAID");
        let _ = col.update_one(doc!{"_id": oid}, doc!{"$set": {"payment_status": ps}}).await;
    }
    Json(json!({"success":true}))
}

pub async fn cancel_order(Extension(db): Extension<Database>, Path(id): Path<String>) -> Json<Value> {
    let orders: Collection<Document> = db.collection("orders");
    let products: Collection<Document> = db.collection("products");
    if let Ok(oid) = ObjectId::parse_str(&id) {
        if let Ok(Some(order)) = orders.find_one(doc!{"_id": oid}).await {
            if let Ok(items) = order.get_array("items") {
                for item in items {
                    if let Some(doc) = item.as_document() {
                        if let Some(pid) = doc.get_str("_id").or(doc.get_str("id")).ok() {
                            if let Ok(p_oid) = ObjectId::parse_str(pid) {
                                let qty = doc.get_i32("qty").unwrap_or(doc.get_i32("quantity").unwrap_or(1));
                                let _ = products.update_one(doc!{"_id": p_oid}, doc!{"$inc": {"stock": qty}}).await;
                            }
                        }
                    }
                }
            }
            let _ = orders.update_one(doc!{"_id": oid}, doc!{"$set": {"status": "cancelled"}}).await;
        }
    }
    Json(json!({"success":true}))
}

pub async fn delete_order(Extension(db): Extension<Database>, Path(id): Path<String>) -> Json<Value> {
    let col: Collection<Document> = db.collection("orders");
    if let Ok(oid) = ObjectId::parse_str(&id) { let _ = col.delete_one(doc!{"_id": oid}).await; }
    Json(json!({"success":true}))
}