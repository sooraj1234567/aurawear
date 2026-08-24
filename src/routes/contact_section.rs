use axum::{Extension, Json};
use mongodb::{bson::{doc, Document}, Collection, Database};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Deserialize, Serialize)]
pub struct ContactSectionBody {
    pub heading: String,
    pub subheading: String,
    pub background_image: String,
    pub background_video: String,
    pub stamp_image: String,
    pub message_types: Vec<String>,
    pub is_active: bool,
}

pub async fn get_contact_section(Extension(db): Extension<Database>) -> Json<Value> {
    let col: Collection<Document> = db.collection("contact_sections");
    if let Ok(Some(doc)) = col.find_one(doc!{}).await {
        return Json(json!({"success":true, "section": doc}));
    }
    let default = doc!{
        "heading": "GET IN TOUCH",
        "subheading": "SEND US A NOTE — WE'D LOVE TO HEAR FROM YOU",
        "background_image": "",
        "background_video": "",
        "stamp_image": "",
        "message_types": ["enquiry","complaint","appreciation","good_review","bad_review","suggestion"],
        "is_active": true
    };
    let _ = col.insert_one(default.clone()).await;
    Json(json!({"success":true, "section": default}))
}

pub async fn update_contact_section(Extension(db): Extension<Database>, Json(body): Json<ContactSectionBody>) -> Json<Value> {
    let col: Collection<Document> = db.collection("contact_sections");
    // try update, if no doc then insert
    let existing = col.find_one(doc!{}).await.unwrap_or(None);
    if existing.is_some() {
        let _ = col.update_one(
            doc!{},
            doc!{"$set": {
                "heading": body.heading,
                "subheading": body.subheading,
                "background_image": body.background_image,
                "background_video": body.background_video,
                "stamp_image": body.stamp_image,
                "message_types": body.message_types,
                "is_active": body.is_active
            }}
        ).await;
    } else {
        let _ = col.insert_one(doc!{
            "heading": body.heading,
            "subheading": body.subheading,
            "background_image": body.background_image,
            "background_video": body.background_video,
            "stamp_image": body.stamp_image,
            "message_types": body.message_types,
            "is_active": body.is_active
        }).await;
    }
    Json(json!({"success":true}))
}