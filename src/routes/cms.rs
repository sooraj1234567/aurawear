use axum::{Extension, Json};
use mongodb::{bson::{doc, Document}, Collection, Database};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Deserialize, Serialize, Clone)]
pub struct PopupConfig {
    pub image_url: String,
    pub target_url: Option<String>,
    pub active: bool,
}

pub async fn get_popup(Extension(db): Extension<Database>) -> Json<Value> {
    let col: Collection<Document> = db.collection("cms_popups");
    match col.find_one(doc! {}).await {
        Ok(Some(document)) => {
            let json_val: Value = mongodb::bson::from_document(document).unwrap_or_default();
            Json(json!({"success": true, "popup": json_val}))
        }
        _ => Json(json!({"success": true, "popup": null}))
    }
}

pub async fn update_popup(
    Extension(db): Extension<Database>,
    Json(body): Json<PopupConfig>,
) -> Json<Value> {
    let col: Collection<Document> = db.collection("cms_popups");
    let update_doc = doc! {
        "image_url": &body.image_url,
        "target_url": body.target_url.clone().unwrap_or_default(),
        "active": body.active,
        "updated_at": chrono::Utc::now().to_rfc3339()
    };

    match col.update_one(doc! {}, doc! { "$set": update_doc }).upsert(true).await {
        Ok(_) => Json(json!({"success": true})),
        Err(e) => Json(json!({"success": false, "error": e.to_string()}))
    }
}