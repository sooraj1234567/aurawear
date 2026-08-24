use axum::{Extension, Json};
use mongodb::{bson::{doc, Document}, Collection, Database};
use serde_json::{json, Value};

pub async fn get_announcement(Extension(db): Extension<Database>) -> Json<Value> {
    let col: Collection<Document> = db.collection("announcement_bar");
    if let Ok(Some(d)) = col.find_one(doc!{}).await {
        return Json(json!({"success":true, "bar": d}));
    }
    Json(json!({"success":true, "bar": {"enabled": true, "bg_color": "#E6F4FF", "text_color": "#1A1611", "speed": 22, "messages": [{"text":"Discover special offers","icon":"tag"},{"text":"Refresh your favorites","icon":"heart"},{"text":"Explore seasonal picks","icon":"shop"}]}}))
}
pub async fn save_announcement(Extension(db): Extension<Database>, Json(body): Json<Value>) -> Json<Value> {
    let col: Collection<Document> = db.collection("announcement_bar");
    let mut doc = Document::new();
    doc.insert("enabled", body.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true));
    doc.insert("bg_color", body.get("bg_color").and_then(|v| v.as_str()).unwrap_or("#E6F4FF").to_string());
    doc.insert("text_color", body.get("text_color").and_then(|v| v.as_str()).unwrap_or("#1A1611").to_string());
    doc.insert("speed", body.get("speed").and_then(|v| v.as_i64()).unwrap_or(22));
    doc.insert("messages", mongodb::bson::to_bson(body.get("messages").unwrap()).unwrap());
    let _ = col.delete_many(doc!{}).await;
    let _ = col.insert_one(doc).await;
    Json(json!({"success":true}))
}