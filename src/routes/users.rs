use axum::{Extension, Json, extract::Path};
use mongodb::{bson::{doc, oid::ObjectId, Document}, Collection, Database};
use serde_json::{json, Value};
use futures_util::StreamExt;

pub async fn get_users(Extension(db): Extension<Database>) -> Json<Value> {
    let col: Collection<Document> = db.collection("users");
    let mut cursor = col.find(doc!{}).await.unwrap();
    let mut list = Vec::new();
    while let Some(Ok(d)) = cursor.next().await { list.push(d); }
    Json(json!({"success":true, "users": list}))
}

pub async fn delete_user(Extension(db): Extension<Database>, Path(id): Path<String>) -> Json<Value> {
    let col: Collection<Document> = db.collection("users");
    if let Ok(oid) = ObjectId::parse_str(&id) {
        let _ = col.delete_one(doc!{"_id": oid}).await;
    }
    Json(json!({"success":true}))
}