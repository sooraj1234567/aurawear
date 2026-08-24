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
    
    // Fallback professional 1970s aesthetic image URL that always works on Render
    Json(json!({
        "image": "https://images.unsplash.com/photo-1495385794356-15371f348c19?q=80&w=1000",
        "path": "https://images.unsplash.com/photo-1495385794356-15371f348c19?q=80&w=1000"
    }))
}