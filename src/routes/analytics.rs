use axum::{
    extract::Extension,
    http::StatusCode,
    Json,
};
use mongodb::{Database, bson::doc};
use futures::stream::TryStreamExt;

pub async fn get_admin_analytics(
    Extension(db): Extension<Database>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let orders_col = db.collection::<mongodb::bson::Document>("orders");
    let products_col = db.collection::<mongodb::bson::Document>("products");
    let subscribers_col = db.collection::<mongodb::bson::Document>("subscribers");

    let total_orders = orders_col.count_documents(doc! {}).await.unwrap_or(0);
    let total_products = products_col.count_documents(doc! {}).await.unwrap_or(0);
    let total_subscribers = subscribers_col.count_documents(doc! {}).await.unwrap_or(0);

    // Calculate total revenue (excluding cancelled orders)
    let mut total_revenue = 0.0;
    if let Ok(mut cursor) = orders_col.find(doc! {}).await {
        while let Ok(Some(order)) = cursor.try_next().await {
            let status = order.get_str("status").unwrap_or("pending").to_lowercase();
            if status != "cancelled" {
                let total = order.get_f64("total").unwrap_or_else(|_| {
                    order.get_i64("total").map(|v| v as f64).unwrap_or_else(|_| {
                        order.get_f64("subtotal").unwrap_or(0.0)
                    })
                });
                total_revenue += total;
            }
        }
    }

    // Count low stock products (stock <= 5)
    let mut low_stock_count = 0;
    if let Ok(mut cursor) = products_col.find(doc! {}).await {
        while let Ok(Some(prod)) = cursor.try_next().await {
            let stock = prod.get_i32("stock").unwrap_or_else(|_| {
                prod.get_i64("stock").map(|v| v as i32).unwrap_or(10)
            });
            if stock <= 5 {
                low_stock_count += 1;
            }
        }
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "analytics": {
            "total_revenue": total_revenue,
            "total_orders": total_orders,
            "total_products": total_products,
            "total_subscribers": total_subscribers,
            "low_stock_count": low_stock_count
        }
    })))
}