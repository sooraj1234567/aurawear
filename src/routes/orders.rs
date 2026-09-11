use axum::{Extension, Json, extract::Path};
use mongodb::{bson::{doc, oid::ObjectId, Document}, Collection, Database};
use serde::Deserialize;
use serde_json::{json, Value};
use futures_util::StreamExt;

#[derive(Deserialize)]
pub struct PayoutBody {
    pub store_name: String,
}

pub async fn get_orders(Extension(db): Extension<Database>) -> Json<Value> {
    let col: Collection<Document> = db.collection("orders");
    let mut cursor = match col.find(doc!{}).await {
        Ok(c) => c,
        Err(_) => return Json(json!({"success": true, "orders": []}))
    };
    let mut list = Vec::new();
    while let Some(Ok(d)) = cursor.next().await { list.push(d); }
    Json(json!({"success":true, "orders": list}))
}

pub async fn get_vendor_orders(
    Extension(db): Extension<Database>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>
) -> Json<Value> {
    let col: Collection<Document> = db.collection("orders");
    let vendor_str = params.get("vendor_id").cloned().unwrap_or_default();

    let filter = if !vendor_str.is_empty() {
        if let Ok(v_oid) = ObjectId::parse_str(&vendor_str) {
            doc! { "$or": [
                { "items.vendor_id": v_oid },
                { "items.vendor_id": vendor_str.clone() }
            ]}
        } else {
            doc! { "items.vendor_id": vendor_str.clone() }
        }
    } else {
        doc! {}
    };

    let mut cursor = match col.find(filter).await {
        Ok(c) => c,
        Err(_) => return Json(json!({"success": false, "orders": []}))
    };

    let mut list = Vec::new();
    while let Some(Ok(d)) = cursor.next().await {
        list.push(d);
    }
    Json(json!({"success": true, "orders": list}))
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
    let raw_items = body.get("items").and_then(|v| v.as_array()).cloned().unwrap_or_default();

    let mut enriched_items = Vec::new();

    for item in &raw_items {
        let pid = item.get("_id").or(item.get("id")).and_then(|v| v.as_str()).unwrap_or("");
        let qty = item.get("qty").or(item.get("quantity")).and_then(|v| v.as_i64()).unwrap_or(1) as i32;
        
        if let Ok(oid) = ObjectId::parse_str(pid) {
            if let Ok(Some(p)) = products.find_one(doc!{"_id": oid}).await {
                let stock = p.get_i32("stock").unwrap_or(p.get_i64("stock").unwrap_or(10) as i32);
                if stock < qty {
                    return Json(json!({"success":false, "error": format!("{} out of stock! Only {} left", p.get_str("name").unwrap_or("Product"), stock)}));
                }
                
                let vendor_id = p.get_object_id("vendor_id").ok();
                let vendor_str = p.get_str("vendor_id").ok().map(|s| s.to_string());

                let mut item_doc = Document::new();
                if let Some(obj) = item.as_object() {
                    for (k, v) in obj {
                        item_doc.insert(k.clone(), mongodb::bson::to_bson(v).unwrap());
                    }
                }
                if let Some(v_id) = vendor_id {
                    item_doc.insert("vendor_id", v_id);
                } else if let Some(v_str) = vendor_str {
                    item_doc.insert("vendor_id", v_str);
                }
                enriched_items.push(mongodb::bson::Bson::Document(item_doc));
            }
        }
    }

    for item in &raw_items {
        let pid = item.get("_id").or(item.get("id")).and_then(|v| v.as_str()).unwrap_or("");
        let qty = item.get("qty").or(item.get("quantity")).and_then(|v| v.as_i64()).unwrap_or(1) as i32;
        if let Ok(oid) = ObjectId::parse_str(pid) {
            let _ = products.update_one(doc!{"_id": oid}, doc!{"$inc": {"stock": -qty}}).await;
        }
    }

    let mut doc = Document::new();
    if let Some(obj) = body.as_object() {
        for (k,v) in obj {
            if k != "items" {
                doc.insert(k.clone(), mongodb::bson::to_bson(v).unwrap());
            }
        }
    }
    doc.insert("items", mongodb::bson::Bson::Array(enriched_items));
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
                        let pid = doc.get_str("_id").or(doc.get_str("id")).unwrap_or("");
                        if let Ok(p_oid) = ObjectId::parse_str(pid) {
                            let qty = doc.get_i32("qty").unwrap_or(doc.get_i32("quantity").unwrap_or(1));
                            let _ = products.update_one(doc!{"_id": p_oid}, doc!{"$inc": {"stock": qty}}).await;
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

pub async fn get_payouts(Extension(db): Extension<Database>) -> Json<Value> {
    let orders_col: Collection<Document> = db.collection("orders");
    let payouts_col: Collection<Document> = db.collection("payouts");

    let mut payouts_status_map = std::collections::HashMap::new();
    if let Ok(mut cursor) = payouts_col.find(doc!{}).await {
        while let Some(Ok(p)) = cursor.next().await {
            if let Some(store) = p.get_str("store_name").ok() {
                let status = p.get_str("status").unwrap_or("pending");
                payouts_status_map.insert(store.to_string(), status.to_string());
            }
        }
    }

    let mut cursor = match orders_col.find(doc!{}).await {
        Ok(c) => c,
        Err(_) => return Json(json!({"success": false, "payouts": []}))
    };

    let mut store_ready_financials: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    let mut store_escrow_financials: std::collections::HashMap<String, f64> = std::collections::HashMap::new();

    while let Some(Ok(order)) = cursor.next().await {
        let order_status = order.get_str("status").unwrap_or("pending").to_lowercase();
        if order_status == "cancelled" {
            continue;
        }

        // Orders must be delivered or completed to count toward ready payout
        let is_delivered = order_status == "delivered" || order_status == "completed";

        if let Ok(items) = order.get_array("items") {
            for item in items {
                if let Some(doc_item) = item.as_document() {
                    let store_name = doc_item.get_str("store_name")
                        .or_else(|_| doc_item.get_str("store"))
                        .or_else(|_| doc_item.get_str("vendor_name"))
                        .unwrap_or("Heritage Archive Co.")
                        .to_string();

                    let price = match doc_item.get("price").or_else(|| doc_item.get("unit_price")) {
                        Some(mongodb::bson::Bson::Double(v)) => *v,
                        Some(mongodb::bson::Bson::Int32(v)) => *v as f64,
                        Some(mongodb::bson::Bson::Int64(v)) => *v as f64,
                        _ => 0.0,
                    };

                    let qty = match doc_item.get("qty").or_else(|| doc_item.get("quantity")) {
                        Some(mongodb::bson::Bson::Int32(v)) => *v as f64,
                        Some(mongodb::bson::Bson::Int64(v)) => *v as f64,
                        Some(mongodb::bson::Bson::Double(v)) => *v,
                        _ => 1.0,
                    };

                    let item_total = price * qty;

                    if is_delivered {
                        *store_ready_financials.entry(store_name.clone()).or_insert(0.0) += item_total;
                    } else {
                        *store_escrow_financials.entry(store_name.clone()).or_insert(0.0) += item_total;
                    }
                }
            }
        }
    }

    let mut all_stores = std::collections::HashSet::new();
    for k in store_ready_financials.keys() { all_stores.insert(k.clone()); }
    for k in store_escrow_financials.keys() { all_stores.insert(k.clone()); }

    let mut payouts_list = Vec::new();
    for store_name in all_stores {
        let ready_gross = *store_ready_financials.get(&store_name).unwrap_or(&0.0);
        let escrow_gross = *store_escrow_financials.get(&store_name).unwrap_or(&0.0);
        let total_gross = ready_gross + escrow_gross;

        let platform_fee = ready_gross * 0.15;
        let net_payout = ready_gross - platform_fee;
        let status = payouts_status_map.get(&store_name).cloned().unwrap_or_else(|| "pending".to_string());
        let payout_id = format!("PAY-{}", &store_name.chars().take(5).collect::<String>().to_uppercase());

        let payout_doc = doc! {
            "payout_id": payout_id,
            "store_name": store_name,
            "gross_sales": total_gross,
            "ready_gross_sales": ready_gross,
            "escrow_gross_sales": escrow_gross,
            "platform_fee": platform_fee,
            "net_payout": net_payout,
            "status": status
        };
        payouts_list.push(payout_doc);
    }

    Json(json!({"success": true, "payouts": payouts_list}))
}

pub async fn process_payout(Extension(db): Extension<Database>, Json(body): Json<PayoutBody>) -> Json<Value> {
    let col: Collection<Document> = db.collection("payouts");
    let store_name = body.store_name.trim().to_string();
    
    let doc = doc! {
        "store_name": store_name.clone(),
        "status": "paid",
        "paid_at": chrono::Utc::now().to_rfc3339()
    };

    let filter = doc! { "store_name": &store_name };
    let update = doc! { "$set": doc };

    match col.update_one(filter, update).upsert(true).await {
        Ok(_) => Json(json!({"success": true})),
        Err(e) => Json(json!({"success": false, "error": e.to_string()}))
    }
}