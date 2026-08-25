use axum::{
    extract::Extension,
    http::StatusCode,
    Json,
};
use mongodb::{Database, bson::doc};
use crate::models::{Subscriber, CreateSubscriberRequest};

pub async fn subscribe(
    Extension(db): Extension<Database>,
    Json(payload): Json<CreateSubscriberRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let email = payload.email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') {
        return Ok(Json(serde_json::json!({ "success": false, "error": "Please enter a valid email address." })));
    }

    let collection = db.collection::<Subscriber>("subscribers");

    // Check if the email is already subscribed
    let existing = collection.find_one(doc! { "email": &email }).await;
    if let Ok(Some(_)) = existing {
        return Ok(Json(serde_json::json!({ 
            "success": true, 
            "message": "You are already subscribed! Use code ARCHIVE10 for 10% off." 
        })));
    }

    let new_sub = Subscriber {
        id: None,
        email,
        created_at: chrono::Utc::now(),
    };

    match collection.insert_one(new_sub).await {
        Ok(_) => Ok(Json(serde_json::json!({ 
            "success": true, 
            "message": "Welcome to the private archive. Use code ARCHIVE10 for 10% off your drop." 
        }))),
        Err(e) => {
            eprintln!("Failed to save subscriber: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}