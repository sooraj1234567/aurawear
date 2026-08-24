use axum::{Extension, Json, http::StatusCode};
use mongodb::{Database, bson::{doc, Document}};
use serde::{Deserialize, Serialize};
use futures::stream::StreamExt;

#[derive(Deserialize)]
pub struct AuraQuery {
    message: String,
}

#[derive(Serialize)]
pub struct AuraResponse {
    reply: String,
}

pub async fn aura_ai_handler(
    Extension(db): Extension<Database>,
    Json(payload): Json<AuraQuery>,
) -> Result<Json<AuraResponse>, StatusCode> {
    let msg = payload.message.to_lowercase();

    // 1. Fashion & Style Intent Filter
    let fashion_keywords = [
        "dress", "jacket", "shirt", "suit", "jeans", "pant", "wear", 
        "style", "fashion", "outfit", "fabric", "leather", "corduroy", 
        "era", "look", "size", "fit", "match", "color", "archive", "hi", "hello", "hey"
    ];
    let is_fashion_related = fashion_keywords.iter().any(|&kw| msg.contains(kw));

    if !is_fashion_related {
        return Ok(Json(AuraResponse {
            reply: "I am your AURAWEAR Haute-Couture Archivist. My expertise is strictly dedicated to 1970s styling, fabric composition, and garment selection from our archive. How may I assist your look today?".to_string()
        }));
    }

    if msg == "hi" || msg == "hello" || msg == "hey" {
        return Ok(Json(AuraResponse {
            reply: "Greetings. I am your personal 1970s Haute-Couture Archivist. Ask me about garment construction, vintage silhouettes, or let me curate a piece from our collection for you.".to_string()
        }));
    }

    // 2. Query MongoDB Compass 'products' collection
    let collection = db.collection::<Document>("products");
    let mut cursor = collection.find(doc! {}).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut matched_product: Option<Document> = None;
    let mut all_products = Vec::new();

    while let Some(result) = cursor.next().await {
        if let Ok(document) = result {
            if let Some(name) = document.get_str("name").ok() {
                if msg.contains(&name.to_lowercase()) {
                    matched_product = Some(document.clone());
                }
            }
            all_products.push(document);
        }
    }

    // Intelligent fallback selection if no exact name match is found
    if matched_product.is_none() && !all_products.is_empty() {
        let rand_idx = rand::random::<usize>() % all_products.len();
        matched_product = Some(all_products[rand_idx].clone());
    }

    // 3. Dynamic Response Generation based on Intent
    let mut reply = "That is a magnificent inquiry. In 1970s tailoring, architectural structure and tactile expression are everything. Consider pairing raw textures with subtle earth tones for an authentic aura.".to_string();

    if msg.contains("size") || msg.contains("fit") {
        reply = "Our small-batch garments follow authentic 1970s relaxed architectural proportions. We recommend selecting your true standard size to achieve the intended fluid drape.".to_string();
    } else if msg.contains("leather") || msg.contains("jacket") {
        reply = "Our leather and outerwear pieces undergo a rigorous vegetable-tanning process, allowing them to mold to your shape over time. Here is a standout piece from our archive that matches this profile:".to_string();
    }

    // Append dynamic product suggestion link if available
    if let Some(prod) = matched_product {
        let prod_name = prod.get_str("name").unwrap_or("Archive Piece");
        
        let prod_price = if let Ok(p) = prod.get_f64("price") {
            p
        } else if let Ok(p) = prod.get_i32("price") {
            p as f64
        } else if let Ok(p) = prod.get_i64("price") {
            p as f64
        } else {
            1450.0
        };

        let prod_fabric = prod.get_str("fabric").unwrap_or("Vintage Blend");
        let prod_id = prod.get_object_id("_id").map(|id| id.to_hex()).unwrap_or_default();

        reply = format!(
            "{}<br><br>✨ **{0}** — ${:.0}<br><em>Fabric: {1}</em><br><a href=\"/product?id={2}\" style=\"color:var(--accent); font-weight:600; text-decoration:underline; display:inline-block; margin-top:6px;\">Explore This Archive Piece →</a>",
            reply, prod_price, prod_id
        );
    }

    Ok(Json(AuraResponse { reply }))
}