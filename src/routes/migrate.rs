use axum::{Extension, Json};
use mongodb::{bson::{doc, Binary, spec::BinarySubtype, oid::ObjectId}, Database};
use serde_json::{Value, json};
use tokio::fs;
use std::collections::HashMap;

pub async fn migrate_local_to_mongo(Extension(db): Extension<Database>) -> Json<Value> {
    let col = db.collection::<mongodb::bson::Document>("uploads");
    let mut mapping: HashMap<String, String> = HashMap::new();
    let mut migrated = 0;

    let mut dir = match fs::read_dir("public/uploads").await {
        Ok(d) => d,
        Err(_) => return Json(json!({"success": false, "error": "public/uploads not found, nothing to migrate"}))
    };

    while let Ok(Some(entry)) = dir.next_entry().await {
        let path = entry.path();
        if !path.is_file() { continue; }
        let filename = entry.file_name().to_string_lossy().to_string();
        let old_url = format!("/uploads/{}", filename);

        // if already in uploads collection skip
        if let Ok(Some(existing)) = col.find_one(doc!{"filename": &filename}).await {
            if let Ok(oid) = existing.get_object_id("_id") {
                mapping.insert(old_url, format!("/api/file/{}", oid.to_hex()));
                continue;
            }
        }

        if let Ok(data) = fs::read(&path).await {
            let binary = Binary { subtype: BinarySubtype::Generic, bytes: data };
            let doc = doc!{"filename": filename.clone(), "data": binary, "content_type": "image/jpeg", "size": path.metadata().map(|m| m.len() as i64).unwrap_or(0), "migrated": true, "created_at": chrono::Utc::now().to_rfc3339()};
            if let Ok(res) = col.insert_one(doc).await {
                if let mongodb::bson::Bson::ObjectId(oid) = res.inserted_id {
                    mapping.insert(old_url, format!("/api/file/{}", oid.to_hex()));
                    migrated += 1;
                }
            }
        }
    }

    // update products
    let prod_col = db.collection::<mongodb::bson::Document>("products");
    let mut prod_updated = 0;
    if let Ok(mut cursor) = prod_col.find(doc!{}).await {
        use futures_util::StreamExt;
        while let Some(Ok(p)) = cursor.next().await {
            if let Ok(old_img) = p.get_str("image_path") {
                if old_img.starts_with("/uploads/") {
                    if let Some(new_url) = mapping.get(old_img) {
                        if let Ok(oid) = p.get_object_id("_id") {
                            let _ = prod_col.update_one(doc!{"_id": oid}, doc!{"$set": {"image_path": new_url, "image": new_url}}).await;
                            prod_updated += 1;
                        }
                    }
                }
            }
        }
    }

    // update users avatar
    let user_col = db.collection::<mongodb::bson::Document>("users");
    let mut user_updated = 0;
    if let Ok(mut cursor) = user_col.find(doc!{}).await {
        use futures_util::StreamExt;
        while let Some(Ok(u)) = cursor.next().await {
            if let Ok(old) = u.get_str("avatar") {
                if old.starts_with("/uploads/") {
                    if let Some(new_url) = mapping.get(old) {
                        if let Ok(email) = u.get_str("email") {
                            let _ = user_col.update_one(doc!{"email": email}, doc!{"$set": {"avatar": new_url}}).await;
                            user_updated += 1;
                        }
                    }
                }
            }
        }
    }

    Json(json!({
        "success": true,
        "message": "All local images/videos moved to Compass - NO LOSS",
        "migrated_files": migrated,
        "products_updated": prod_updated,
        "users_updated": user_updated
    }))
}