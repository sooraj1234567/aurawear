use mongodb::{Client, Database};

pub async fn init_db() -> Database {
    let mongo_uri = std::env::var("MONGODB_URI")
        .unwrap_or_else(|_| "mongodb://localhost:27017/aurawear_db".to_string());
    
    let client = Client::with_uri_str(&mongo_uri).await.unwrap();
    
    // FIXED - use aurawear_db as seen in Compass
    let db = client.database("aurawear_db");
    
    let count = db.collection::<mongodb::bson::Document>("products")
        .count_documents(mongodb::bson::doc!{}).await.unwrap_or(0);
    println!("Connected to DB: {} - Products: {}", db.name(), count);
    
    db
}