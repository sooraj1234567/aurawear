mod db;
mod models;
mod routes;
mod email;

use axum::{
    routing::{get, post, delete, put},
    Router, Extension,
    extract::{DefaultBodyLimit, Query},
    response::Html,
};
use tower_http::services::ServeDir;
use std::{net::SocketAddr, collections::HashMap};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tokio::fs::create_dir_all("public/uploads").await.unwrap();
    let database = db::init_db().await;

    let app = Router::new()
       .route("/api/upload", post(routes::uploads::upload_image))
       .route("/api/file/{id}", get(routes::uploads::get_file))
       .route("/api/hero", get(routes::hero::get_hero))
       .route("/api/hero-upload", post(routes::uploads::upload_hero))
       .route("/api/contact-section", get(routes::contact_section::get_contact_section).put(routes::contact_section::update_contact_section))
       .route("/api/auth/register", post(routes::auth::register))
       .route("/api/auth/login", post(routes::auth::login))
       .route("/api/auth/me", post(routes::auth::me))
       .route("/api/auth/update", post(routes::auth::update))
       .route("/api/users", get(routes::auth::get_users))
       .route("/api/users/{id}", delete(routes::auth::delete_user))
       .route("/api/contact", post(routes::contact::create_contact).get(routes::contact::get_contacts))
       .route("/api/contact/{id}/reply", post(routes::contact::reply_contact))
       .route("/api/contact/{id}", delete(routes::contact::delete_contact))
       .route("/api/contact/track", get(routes::contact::track_contact))
       .route("/api/announcement", get(routes::announcement::get_announcement).put(routes::announcement::save_announcement))
       .route("/api/products", get(routes::products::get_products).post(routes::products::create_product))
       .route("/api/products/{id}", get(routes::products::get_product_by_id).delete(routes::products::delete_product).put(routes::products::update_product))
       .route("/api/orders", get(routes::orders::get_orders).post(routes::orders::create_order))
       .route("/api/orders/{id}/status", post(routes::orders::update_status))
       .route("/api/orders/{id}/tracking", post(routes::orders::update_tracking))
       .route("/api/orders/{id}/note", post(routes::orders::update_note))
       .route("/api/orders/{id}/cancel", post(routes::orders::cancel_order))
       .route("/api/orders/{id}/payment_status", post(routes::orders::update_payment_status))
       .route("/api/orders/{id}", get(routes::orders::get_order_by_id).delete(routes::orders::delete_order))
       // Inside your Axum Router setup (e.g., app router)
.route("/api/auth/delete-account", post(crate::routes::auth::delete_user_account))
.route("/api/admin/deletion-logs", get(crate::routes::auth::get_deletion_logs))
       // AURA AI CONCIERGE ENDPOINT
       .route("/api/aura-ai", post(routes::ai::aura_ai_handler))
       
       .route("/", get(index_page))
       .route("/index.html", get(index_page))
       .route("/admin", get(admin_page))
       .route("/admin.html", get(admin_page))
       .route("/login", get(login_page))
       .route("/login.html", get(login_page))
       .route("/profile", get(profile_page))
       .route("/profile.html", get(profile_page))
       .route("/checkout", get(checkout_page))
       .route("/checkout.html", get(checkout_page))
       .route("/product", get(product_page))
       .route("/product.html", get(product_page))
       .route("/success", get(success_page))
       .route("/success.html", get(success_page))
       .route("/health", get(|| async { "OK" }))
       .nest_service("/css", ServeDir::new("public/css"))
       .nest_service("/uploads", ServeDir::new("public/uploads"))
       .nest_service("/js", ServeDir::new("public/js"))
       .fallback_service(ServeDir::new("templates").append_index_html_on_directories(true))
       .layer(DefaultBodyLimit::disable())
       .layer(Extension(database));

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = SocketAddr::from(([127, 0, 0, 1], port.parse::<u16>().unwrap()));
    println!("AuraWear running on http://{} - Admin http://{}/admin - Login http://{}/login", addr, addr, addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index_page() -> Html<String> { Html(tokio::fs::read_to_string("templates/index.html").await.unwrap_or_else(|_| "<h1>index.html missing</h1>".into())) }
async fn admin_page() -> Html<String> { Html(tokio::fs::read_to_string("templates/admin.html").await.unwrap_or_else(|_| "<h1>admin.html missing</h1>".into())) }
async fn login_page() -> Html<String> { Html(tokio::fs::read_to_string("templates/login.html").await.unwrap_or_else(|_| "<h1>login.html missing</h1>".into())) }
async fn profile_page() -> Html<String> { Html(tokio::fs::read_to_string("templates/profile.html").await.unwrap_or_else(|_| "<h1>profile.html missing</h1>".into())) }
async fn checkout_page() -> Html<String> { Html(tokio::fs::read_to_string("templates/checkout.html").await.unwrap_or_else(|_| "<h1>checkout.html missing</h1>".into())) }
async fn product_page() -> Html<String> { Html(tokio::fs::read_to_string("templates/product.html").await.unwrap_or_else(|_| "<h1>product.html missing</h1>".into())) }
async fn success_page(Query(params): Query<HashMap<String, String>>) -> Html<String> {
    let id = params.get("id").cloned().unwrap_or_else(|| "AURAW-".to_string() + &chrono::Utc::now().timestamp().to_string()[6..]);
    let template = tokio::fs::read_to_string("templates/success.html").await.unwrap_or_default();
    let html = if template.is_empty() {
        format!(r#"<!DOCTYPE html><html><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Success - AURAWEAR</title>
        <link href="https://fonts.googleapis.com/css2?family=Fraunces&display=swap" rel="stylesheet">
        <style>body{{margin:0;background:#FBF8F3;font-family:Inter,sans-serif}}.wrap{{min-height:80vh;display:grid;place-items:center;text-align:center;padding:40px}}
       .card{{background:#fff;border:1px solid rgba(0,0,0,.12);padding:48px;max-width:480px}} h1{{font-family:Fraunces;font-size:52px;font-weight:300;margin:0}}.id{{margin-top:20px;font-size:11px;letter-spacing:2px;opacity:.5;border:1px dashed rgba(0,0,0,.2);padding:12px}}
       .btn{{display:inline-block;margin-top:20px;background:#1A1611;color:#fff;padding:14px 22px;text-decoration:none;font-size:11px;letter-spacing:2px}}</style></head>
        <body><nav style="padding:20px 4vw;border-bottom:1px solid rgba(0,0,0,.12);font-family:Fraunces;letter-spacing:4px">AURAWEAR</nav>
        <div class="wrap"><div class="card"><div style="width:56px;height:56px;border-radius:50%;background:#1A1611;color:#fff;display:grid;place-items:center;margin:0 auto 20px">✓</div>
        <h1>Order<br><em style="font-family:serif">Placed</em></h1><p style="opacity:.6;margin-top:16px">Your archive is being packed.</p>
        <div class="id">ORDER ID: {}</div><a class="btn" href="/">BACK TO ARCHIVE</a></div></div>
        <script>localStorage.removeItem('aurawear_cart')</script></body></html>"#, id)
    } else { template.replace("{{ORDER_ID}}", &id) };
    Html(html)
}