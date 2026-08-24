use axum::{
    Extension, Json,
    response::{IntoResponse, Response},
    http::{StatusCode, header},
};
use mongodb::{bson::{doc, Binary}, Collection, Database, bson::Document};
use serde_json::{json, Value};

pub async fn get_hero(Extension(db): Extension<Database>) -> Response {
    let col: Collection<Document> = db.collection("hero");
    
    if let Ok(Some(doc)) = col.find_one(doc!{}).await {
        // If the database document has raw binary data saved, serve it directly from MongoDB!
        if let Ok(bin) = doc.get_binary_col("data") {
            let bytes = bin.bytes.clone();
            return (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "image/jpeg")],
                bytes,
            ).into_response();
        }
        
        // Fallback to text paths if binary isn't present
        if let Ok(img) = doc.get_str("image") {
            if img.starts_with("http") {
                return Json(json!({"image": img, "image_path": img, "path": img})).into_response();
            }
        }
    }
    
    Json(json!({"image": "https://images.unsplash.com/photo-1495385794356-15371f348c19?q=80&w=1000", "path": "https://images.unsplash.com/photo-1495385794356-15371f348c19?q=80&w=1000"})).into_response()
}