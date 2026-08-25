use axum::{
    Extension,
    Json,
    extract::{Path, Query},
};
use mongodb::{
    bson::{
        doc,
        Document,
        oid::ObjectId,
    },
    Collection,
    Database,
};
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
pub struct ReplyBody {
    pub reply: String,
}

pub async fn create_contact(
    Extension(db): Extension<Database>,
    Json(body): Json<ContactBody>,
) -> Json<Value> {
    let col: Collection<Document> = db.collection("contacts");

    let email = body.email.trim().to_lowercase();
    let user_email = body
        .user_email
        .unwrap_or_default()
        .trim()
        .to_lowercase();

    let doc = doc! {
        "name": body.name,
        "email": email,
        "user_email": user_email,
        "phone": body.phone.unwrap_or_default(),
        "message_type": body.message_type.to_lowercase(),
        "message": body.message,
        "image": body.image.unwrap_or_default(),
        "status": "pending",
        "admin_reply": "",
        "created_at": chrono::Utc::now().to_rfc3339(),
    };

    match col.insert_one(doc).await {
        Ok(r) => {
            let id = r.inserted_id.to_string();

            println!(
                "Contact message created successfully. Contact ID: {}",
                id
            );

            Json(json!({
                "success": true,
                "id": id
            }))
        }

        Err(e) => {
            eprintln!(
                "Failed to create contact message: {}",
                e
            );

            Json(json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}

pub async fn get_contacts(
    Extension(db): Extension<Database>,
) -> Json<Value> {
    let col: Collection<Document> = db.collection("contacts");

    let mut cursor = match col.find(doc! {}).await {
        Ok(cursor) => cursor,

        Err(e) => {
            eprintln!(
                "Failed to load contacts: {}",
                e
            );

            return Json(json!({
                "success": false,
                "error": e.to_string(),
                "contacts": []
            }));
        }
    };

    let mut list = Vec::new();

    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => {
                list.push(document);
            }

            Err(e) => {
                eprintln!(
                    "Failed to read contact document: {}",
                    e
                );
            }
        }
    }

    list.sort_by(|a, b| {
        b.get_str("created_at")
            .unwrap_or("")
            .cmp(
                a.get_str("created_at")
                    .unwrap_or("")
            )
    });

    Json(json!({
        "success": true,
        "contacts": list
    }))
}

pub async fn reply_contact(
    Extension(db): Extension<Database>,
    Path(id): Path<String>,
    Json(body): Json<ReplyBody>,
) -> Json<Value> {
    let col: Collection<Document> = db.collection("contacts");

    let reply = body.reply.trim().to_string();

    if reply.is_empty() {
        return Json(json!({
            "success": false,
            "error": "Reply cannot be empty"
        }));
    }

    let oid = match ObjectId::parse_str(&id) {
        Ok(o) => o,

        Err(e) => {
            eprintln!(
                "Invalid contact ID '{}': {}",
                id,
                e
            );

            return Json(json!({
                "success": false,
                "error": "Invalid contact ID"
            }));
        }
    };

    let contact = match col
        .find_one(doc! {
            "_id": oid
        })
        .await
    {
        Ok(Some(contact)) => contact,

        Ok(None) => {
            eprintln!(
                "Contact not found for reply. ID: {}",
                id
            );

            return Json(json!({
                "success": false,
                "error": "Contact not found"
            }));
        }

        Err(e) => {
            eprintln!(
                "Failed to find contact {}: {}",
                id,
                e
            );

            return Json(json!({
                "success": false,
                "error": e.to_string()
            }));
        }
    };

    let to_email = contact
        .get_str("email")
        .unwrap_or("")
        .trim()
        .to_lowercase();

    let name = contact
        .get_str("name")
        .unwrap_or("User")
        .to_string();

    let orig = contact
        .get_str("message")
        .unwrap_or("")
        .to_string();

    let update_result = match col
        .update_one(
            doc! {
                "_id": oid
            },
            doc! {
                "$set": {
                    "admin_reply": &reply,
                    "status": "replied",
                    "updated_at": chrono::Utc::now().to_rfc3339()
                }
            },
        )
        .await
    {
        Ok(result) => result,

        Err(e) => {
            eprintln!(
                "Failed to save admin reply for contact {}: {}",
                id,
                e
            );

            return Json(json!({
                "success": false,
                "error": format!(
                    "Failed to save reply: {}",
                    e
                )
            }));
        }
    };

    if update_result.matched_count == 0 {
        eprintln!(
            "MongoDB did not match contact {} while saving reply.",
            id
        );

        return Json(json!({
            "success": false,
            "error": "Contact was not found while saving the reply"
        }));
    }

    println!(
        "Admin reply saved for contact {}. Matched: {}, Modified: {}",
        id,
        update_result.matched_count,
        update_result.modified_count
    );

    let updated_contact = match col
        .find_one(doc! {
            "_id": oid
        })
        .await
    {
        Ok(Some(contact)) => contact,

        Ok(None) => {
            eprintln!(
                "Contact disappeared after reply update. ID: {}",
                id
            );

            return Json(json!({
                "success": false,
                "error": "Reply was saved but contact could not be retrieved"
            }));
        }

        Err(e) => {
            eprintln!(
                "Failed to verify saved reply for contact {}: {}",
                id,
                e
            );

            return Json(json!({
                "success": false,
                "error": format!(
                    "Reply was saved but verification failed: {}",
                    e
                )
            }));
        }
    };

    let saved_reply = updated_contact
        .get_str("admin_reply")
        .unwrap_or("")
        .to_string();

    if saved_reply != reply {
        eprintln!(
            "Reply verification failed for contact {}. Expected '{}', found '{}'",
            id,
            reply,
            saved_reply
        );

        return Json(json!({
            "success": false,
            "error": "Reply could not be verified in the database"
        }));
    }

    // Sends the email in the background using Resend HTTP API
    if !to_email.is_empty() {
        let reply_clone = reply.clone();
        let id_string = id.clone();
        let email_clone = to_email.clone();
        let name_clone = name.clone();
        let orig_clone = orig.clone();

        tokio::spawn(async move {
            match send_reply_email(
                &email_clone,
                &name_clone,
                &id_string,
                &orig_clone,
                &reply_clone,
            )
            .await
            {
                Ok(_) => {
                    println!(
                        "Reply email successfully sent to {}",
                        email_clone
                    );
                }

                Err(e) => {
                    eprintln!(
                        "Reply email failed for {}: {}",
                        email_clone,
                        e
                    );
                }
            }
        });
    } else {
        eprintln!(
            "Contact {} has no email address. Reply saved to database, but email was not sent.",
            id
        );
    }

    Json(json!({
        "success": true,
        "id": id,
        "contact": updated_contact
    }))
}

pub async fn delete_contact(
    Extension(db): Extension<Database>,
    Path(id): Path<String>,
) -> Json<Value> {
    let col: Collection<Document> = db.collection("contacts");

    let oid = match ObjectId::parse_str(&id) {
        Ok(oid) => oid,

        Err(_) => {
            return Json(json!({
                "success": false,
                "error": "Invalid contact ID"
            }));
        }
    };

    match col
        .delete_one(doc! {
            "_id": oid
        })
        .await
    {
        Ok(result) => {
            if result.deleted_count == 0 {
                return Json(json!({
                    "success": false,
                    "error": "Contact not found"
                }));
            }

            Json(json!({
                "success": true
            }))
        }

        Err(e) => {
            eprintln!(
                "Failed to delete contact {}: {}",
                id,
                e
            );

            Json(json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}

pub async fn track_contact(
    Extension(db): Extension<Database>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Value> {
    let col: Collection<Document> = db.collection("contacts");

    let id_raw = params
        .get("id")
        .cloned()
        .unwrap_or_default()
        .trim()
        .replace("AUR-", "")
        .replace("aur-", "");

    if !id_raw.is_empty() {
        match ObjectId::parse_str(&id_raw) {
            Ok(oid) => {
                match col
                    .find_one(doc! {
                        "_id": oid
                    })
                    .await
                {
                    Ok(Some(contact)) => {
                        println!(
                            "Contact tracking successful by ID: {}",
                            id_raw
                        );

                        return Json(json!({
                            "success": true,
                            "contact": contact
                        }));
                    }

                    Ok(None) => {
                        println!(
                            "No contact found for ID: {}",
                            id_raw
                        );
                    }

                    Err(e) => {
                        eprintln!(
                            "Contact tracking database error for ID {}: {}",
                            id_raw,
                            e
                        );

                        return Json(json!({
                            "success": false,
                            "error": e.to_string()
                        }));
                    }
                }
            }

            Err(_) => {}
        }
    }

    let email = params
        .get("email")
        .map(|s| s.trim().to_lowercase())
        .unwrap_or_default();

    if !email.is_empty() {
        match col
            .find_one(doc! {
                "email": &email
            })
            .sort(doc! {
                "created_at": -1
            })
            .await
        {
            Ok(Some(contact)) => {
                println!(
                    "Contact tracking successful by email: {}",
                    email
                );

                return Json(json!({
                    "success": true,
                    "contact": contact
                }));
            }

            Ok(None) => {}

            Err(e) => {
                eprintln!(
                    "Contact email lookup failed for {}: {}",
                    email,
                    e
                );
            }
        }

        match col
            .find_one(doc! {
                "user_email": &email
            })
            .sort(doc! {
                "created_at": -1
            })
            .await
        {
            Ok(Some(contact)) => {
                println!(
                    "Contact tracking successful by user_email: {}",
                    email
                );

                return Json(json!({
                    "success": true,
                    "contact": contact
                }));
            }

            Ok(None) => {}

            Err(e) => {
                eprintln!(
                    "Contact user_email lookup failed for {}: {}",
                    email,
                    e
                );
            }
        }
    }

    Json(json!({
        "success": false,
        "error": "Contact message not found"
    }))
}