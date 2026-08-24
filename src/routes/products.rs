use axum::{Extension, Json, extract::Path};
use mongodb::{bson::{doc, oid::ObjectId, Document}, Collection, Database};
use serde_json::{json, Value};
use futures_util::StreamExt;

pub async fn get_products(Extension(db): Extension<Database>) -> Json<Value> {
    let col: Collection<Document> = db.collection("products");
    let uploads: Collection<Document> = db.collection("uploads");
    let mut cursor = col.find(doc!{}).await.unwrap();
    let mut list = Vec::new();
    while let Some(Ok(mut d)) = cursor.next().await {
        // AUTO-FIX old /api/file/id -> /uploads/filename
        if let Ok(path) = d.get_str("image_path") {
            if path.starts_with("/api/file/") {
                let id = path.replace("/api/file/", "");
                if let Ok(oid) = ObjectId::parse_str(&id) {
                    if let Ok(Some(up)) = uploads.find_one(doc!{"_id": oid}).await {
                        if let Ok(fname) = up.get_str("filename") {
                            let new_path = format!("/uploads/{}", fname);
                            d.insert("image_path", new_path.clone());
                            d.insert("image", new_path);
                        }
                    }
                }
            }
        }
        list.push(d);
    }
    Json(json!({"success":true, "products": list}))
}

pub async fn get_product_by_id(Extension(db): Extension<Database>, Path(id): Path<String>) -> Json<Value> {
    let col: Collection<Document> = db.collection("products");
    if let Ok(oid) = ObjectId::parse_str(&id) {
        if let Ok(Some(p)) = col.find_one(doc!{"_id": oid}).await {
            return Json(json!({"success":true, "product": p}));
        }
    }
    Json(json!({"success":false, "error":"not found"}))
}

pub async fn create_product(Extension(db): Extension<Database>, Json(body): Json<Value>) -> Json<Value> {
    let col: Collection<Document> = db.collection("products");
    let mut doc = Document::new();
    if let Some(obj) = body.as_object() {
        for (k,v) in obj {
            if k == "sizes" {
                if let Some(arr) = v.as_array() {
                    let vec_str: Vec<String> = arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect();
                    doc.insert(k.clone(), mongodb::bson::to_bson(&vec_str).unwrap());
                } else if let Some(s) = v.as_str() {
                    let vec_str: Vec<String> = s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect();
                    doc.insert(k.clone(), mongodb::bson::to_bson(&vec_str).unwrap());
                }
            } else if k == "images" {
                if let Some(arr) = v.as_array() {
                    let vec_str: Vec<String> = arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect();
                    doc.insert(k.clone(), mongodb::bson::to_bson(&vec_str).unwrap());
                }
            } else if let Some(s) = v.as_str() { 
                doc.insert(k.clone(), s.to_string()); 
            } else if let Some(n) = v.as_i64() { 
                doc.insert(k.clone(), n); 
            } else if let Some(n) = v.as_f64() { 
                doc.insert(k.clone(), n); 
            } else { 
                doc.insert(k.clone(), mongodb::bson::to_bson(v).unwrap_or(mongodb::bson::Bson::Null)); 
            }
        }
    }
    doc.insert("created_at", chrono::Utc::now().to_rfc3339());
    match col.insert_one(doc).await {
        Ok(r) => Json(json!({"success":true, "id": r.inserted_id})),
        Err(e) => Json(json!({"success":false, "error": e.to_string()}))
    }
}

pub async fn delete_product(Extension(db): Extension<Database>, Path(id): Path<String>) -> Json<Value> {
    let col: Collection<Document> = db.collection("products");
    if let Ok(oid) = ObjectId::parse_str(&id) { let _ = col.delete_one(doc!{"_id": oid}).await; }
    Json(json!({"success":true}))
}

pub async fn update_product(Extension(db): Extension<Database>, Path(id): Path<String>, Json(body): Json<Value>) -> Json<Value> {
    let col: Collection<Document> = db.collection("products");
    let oid = match ObjectId::parse_str(&id) {
        Ok(o) => o,
        Err(_) => return Json(json!({"success":false, "error":"invalid id"}))
    };
    let mut set = doc!{};
    if let Some(v) = body.get("name").and_then(|x| x.as_str()) { set.insert("name", v.to_string()); }
    if let Some(v) = body.get("price") { if let Some(n) = v.as_f64() { set.insert("price", n); } else if let Some(n) = v.as_i64() { set.insert("price", n as f64); } }
    if let Some(v) = body.get("stock") { if let Some(n) = v.as_i64() { set.insert("stock", n); } else if let Some(n) = v.as_f64() { set.insert("stock", n as i64); } }
    if let Some(v) = body.get("category").and_then(|x| x.as_str()) { set.insert("category", v.to_string()); }
    if let Some(v) = body.get("gender").and_then(|x| x.as_str()) { set.insert("gender", v.to_string()); }
    if let Some(v) = body.get("fabric").and_then(|x| x.as_str()) { set.insert("fabric", v.to_string()); }
    if let Some(v) = body.get("era").and_then(|x| x.as_str()) { set.insert("era", v.to_string()); }
    if let Some(v) = body.get("description").and_then(|x| x.as_str()) { set.insert("description", v.to_string()); }
    if let Some(v) = body.get("image").and_then(|x| x.as_str()) { set.insert("image", v.to_string()); set.insert("image_path", v.to_string()); }
    if let Some(v) = body.get("image_path").and_then(|x| x.as_str()) { set.insert("image_path", v.to_string()); set.insert("image", v.to_string()); }
    
    // Handle sizes array parsing safely whether string or array provided
    if let Some(v) = body.get("sizes") {
        if let Some(arr) = v.as_array() {
            let vec_str: Vec<String> = arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect();
            set.insert("sizes", mongodb::bson::to_bson(&vec_str).unwrap());
        } else if let Some(s) = v.as_str() {
            let vec_str: Vec<String> = s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect();
            set.insert("sizes", mongodb::bson::to_bson(&vec_str).unwrap());
        }
    }

    // Handle multiple product images array
    if let Some(v) = body.get("images") {
        if let Some(arr) = v.as_array() {
            let vec_str: Vec<String> = arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect();
            set.insert("images", mongodb::bson::to_bson(&vec_str).unwrap());
        }
    }

    set.insert("updated_at", chrono::Utc::now().to_rfc3339());

    match col.update_one(doc!{"_id": oid}, doc!{"$set": set}).await {
        Ok(_) => Json(json!({"success":true})),
        Err(e) => Json(json!({"success":false, "error": e.to_string()}))
    }
}