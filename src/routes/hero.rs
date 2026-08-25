use axum::{Extension, Json};
use mongodb::{bson::doc, Collection, Database, bson::Document};
use serde_json::{json, Value};

pub async fn get_hero(Extension(db): Extension<Database>) -> Json<Value> {
    let col: Collection<Document> = db.collection("hero");
    if let Ok(Some(doc)) = col.find_one(doc!{}).await {
        if let Ok(img) = doc.get_str("image") {
            if !img.is_empty() {
                return Json(json!({"image": img, "image_path": img, "path": img}));
            }
        }
        if let Ok(img) = doc.get_str("image_path") {
            if !img.is_empty() {
                return Json(json!({"image": img, "path": img}));
            }
        }
    }
    
    // Fallback hero image path when no database document exists yet
    Json(json!({
        "image": "/uploads/hero_6a86c72252ff92d890379272.jpg",
        "image_path": "/uploads/hero_6a86c72252ff92d890379272.jpg",
        "path": "/uploads/hero_6a86c72252ff92d890379272.jpg"
    }))
}