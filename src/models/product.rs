use serde::{Deserialize, Serialize};
use mongodb::bson::oid::ObjectId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Gender { Men, Women, Unisex, All }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Product {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub price: f64,
    pub gender: Gender,
    pub category: String,
    pub sizes: Vec<String>,
    pub description: String,
    pub image_path: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}