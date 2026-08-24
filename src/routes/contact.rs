use axum::{Extension, Json, extract::{Path, Query}};
use mongodb::{bson::{doc, Document, oid::ObjectId}, Collection, Database};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use futures_util::StreamExt;
use crate::email::send_reply_email;

#[derive(Deserialize)]
pub struct ContactBody {
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    #[serde(rename = "message_type")]
    pub message_type: String,
    pub message: String,
    pub image: Option<String>,
    pub user_email: Option<String>,
}
#[derive(Deserialize)]
pub struct ReplyBody { pub reply: String }

pub async fn create_contact(Extension(db): Extension<Database>, Json(body): Json<ContactBody>) -> Json<Value> {
    let col: Collection<Document> = db.collection("contacts");
    let doc = doc!{
        "name": body.name,
        "email": body.email.to_lowercase(),
        "user_email": body.user_email.unwrap_or_default().to_lowercase(),
        "phone": body.phone.unwrap_or_default(),
        "message_type": body.message_type.to_lowercase(),
        "message": body.message,
        "image": body.image.unwrap_or_default(),
        "status": "pending",
        "admin_reply": "",
        "created_at": chrono::Utc::now().to_rfc3339()
    };
    match col.insert_one(doc).await {
        Ok(r) => Json(json!({"success":true, "id": r.inserted_id.to_string()})),
        Err(e) => Json(json!({"success":false, "error": e.to_string()}))
    }
}

pub async fn get_contacts(Extension(db): Extension<Database>) -> Json<Value> {
    let col: Collection<Document> = db.collection("contacts");
    let mut cursor = col.find(doc!{}).await.unwrap();
    let mut list = Vec::new();
    while let Some(Ok(d)) = cursor.next().await { list.push(d); }
    list.sort_by(|a,b| b.get_str("created_at").unwrap_or("").cmp(a.get_str("created_at").unwrap_or("")) );
    Json(json!({"success":true, "contacts": list}))
}

pub async fn reply_contact(Extension(db): Extension<Database>, Path(id): Path<String>, Json(body): Json<ReplyBody>) -> Json<Value> {
    let col: Collection<Document> = db.collection("contacts");
    let oid = match ObjectId::parse_str(&id) { Ok(o) => o, Err(_) => return Json(json!({"success":false})) };
    if let Ok(Some(contact)) = col.find_one(doc!{"_id": oid}).await {
        let to_email = contact.get_str("email").unwrap_or("").to_string();
        let name = contact.get_str("name").unwrap_or("User").to_string();
        let orig = contact.get_str("message").unwrap_or("").to_string();
        let _ = col.update_one(doc!{"_id": oid}, doc!{"$set": {"admin_reply": body.reply.clone(), "status": "replied"}}).await;
        let reply_clone = body.reply.clone();
        tokio::spawn(async move {
            let _ = send_reply_email(&to_email, &name, &id, &orig, &reply_clone).await;
        });
        return Json(json!({"success":true}));
    }
    Json(json!({"success":false}))
}

pub async fn delete_contact(Extension(db): Extension<Database>, Path(id): Path<String>) -> Json<Value> {
    let col: Collection<Document> = db.collection("contacts");
    if let Ok(oid) = ObjectId::parse_str(&id) { let _ = col.delete_one(doc!{"_id": oid}).await; }
    Json(json!({"success":true}))
}

pub async fn track_contact(Extension(db): Extension<Database>, Query(params): Query<HashMap<String,String>>) -> Json<Value> {
    let col: Collection<Document> = db.collection("contacts");
    let email = params.get("email").map(|s| s.to_lowercase()).unwrap_or_default();
    let id_raw = params.get("id").cloned().unwrap_or_default().replace("AUR-","").replace("aur-","");
    if let Ok(oid) = ObjectId::parse_str(&id_raw) {
        if let Ok(Some(c)) = col.find_one(doc!{"_id": oid}).await {
            return Json(json!({"success":true, "contact": c}));
        }
    }
    Json(json!({"success":false}))
}