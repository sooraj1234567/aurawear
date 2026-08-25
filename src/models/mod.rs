pub mod product;
pub mod user;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Review {
    #[serde(skip_serializing_if = "Option::is_none", rename = "_id")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    #[serde(rename = "productId")]
    pub product_id: String,
    pub name: String,
    pub text: String,
    pub rating: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateReviewRequest {
    #[serde(rename = "productId")]
    pub product_id: String,
    pub name: String,
    pub text: String,
    pub rating: Option<i32>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Subscriber {
    #[serde(skip_serializing_if = "Option::is_none", rename = "_id")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateSubscriberRequest {
    pub email: String,
}