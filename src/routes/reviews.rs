use axum::{
    extract::{Path, Extension},
    http::StatusCode,
    Json,
};
use mongodb::{Database, bson::doc};
use futures::stream::TryStreamExt;
use crate::models::{Review, CreateReviewRequest};

// POST /api/reviews - Save review to MongoDB
pub async fn create_review(
    Extension(db): Extension<Database>,
    Json(payload): Json<CreateReviewRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let collection = db.collection::<Review>("reviews");

    let new_review = Review {
        id: None,
        product_id: payload.product_id,
        name: payload.name.trim().to_string(),
        text: payload.text.trim().to_string(),
        rating: payload.rating.unwrap_or(5),
        created_at: chrono::Utc::now(),
    };

    match collection.insert_one(new_review).await {
        Ok(_) => Ok(Json(serde_json::json!({ "success": true, "message": "Review saved successfully" }))),
        Err(e) => {
            eprintln!("Failed to insert review: {}", e);
            // Send the exact database error back to the frontend popup for easy debugging
            Ok(Json(serde_json::json!({ "success": false, "error": e.to_string() })))
        }
    }
}

// GET /api/reviews/{productId} - Fetch reviews from MongoDB for a product
pub async fn get_reviews(
    Extension(db): Extension<Database>,
    Path(product_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let collection = db.collection::<Review>("reviews");
    let filter = doc! { "productId": product_id };

    match collection.find(filter).await {
        Ok(cursor) => {
            let reviews: Vec<Review> = match cursor.try_collect().await {
                Ok(v) => v,
                Err(_) => vec![],
            };
            Ok(Json(serde_json::json!({ "success": true, "reviews": reviews })))
        }
        Err(e) => {
            eprintln!("Failed to fetch reviews: {}", e);
            Ok(Json(serde_json::json!({ "success": true, "reviews": [] })))
        }
    }
}