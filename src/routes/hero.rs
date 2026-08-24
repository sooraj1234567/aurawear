use axum::{Extension, Json};
use mongodb::{bson::doc, Collection, Database, bson::Document};
use serde_json::{json, Value};

pub async fn get_hero(Extension(db): Extension<Database>) -> Json<Value> {
    let col: Collection<Document> = db.collection("hero");
    if let Ok(Some(doc)) = col.find_one(doc!{}).await {
        if let Ok(img) = doc.get_str("image") {
            return Json(json!({"image": img, "image_path": img, "path": img}));
        }
        if let Ok(img) = doc.get_str("image_path") {
            return Json(json!({"image": img, "path": img}));
        }
    }
    Json(json!({"image": "/uploads/hero.jpg", "path": "/uploads/hero.jpg"}))
}