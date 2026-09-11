mod db;
mod models;
mod routes;
mod email;

use axum::{
    routing::{get, post, delete, put},
    Router, Extension,
    extract::{Query},
    response::Html,
};
use tower_http::services::ServeDir;
use std::{net::SocketAddr, collections::HashMap};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    println!(
        "Brevo API Key loaded: {}",
        std::env::var("BREVO_API_KEY").is_ok()
    );

    tokio::fs::create_dir_all("public/uploads").await.unwrap();
    let database = db::init_db().await;

    // Seed default admin user into the database if none exists
    db_seed_admin(&database).await;

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
       .route("/api/admin/users/{id}", delete(routes::auth::delete_user))
       .route("/api/admin/crm/users/{id}", delete(routes::auth::delete_user))
       .route("/api/users/{id}/anonymize", post(routes::auth::anonymize_user))
       .route("/api/admin/users/{id}/anonymize", post(routes::auth::anonymize_user))
       
       // Contact Routes (supporting both singular & plural endpoints to eliminate admin fetching errors)
       .route("/api/contact", post(routes::contact::create_contact).get(routes::contact::get_contacts))
       .route("/api/contacts", get(routes::contact::get_contacts).post(routes::contact::create_contact))
       .route("/api/contact/{id}/reply", post(routes::contact::reply_contact))
       .route("/api/contacts/{id}/reply", post(routes::contact::reply_contact))
       .route("/api/contact/{id}", delete(routes::contact::delete_contact))
       .route("/api/contacts/{id}", delete(routes::contact::delete_contact))
       .route("/api/contact/track", get(routes::contact::track_contact))
       .route("/api/announcement", get(routes::announcement::get_announcement).put(routes::announcement::save_announcement))
       
       // Product Routes (including multi-vendor endpoint)
       .route("/api/products", get(routes::products::get_products).post(routes::products::create_product))
       .route("/api/vendor/products", get(routes::products::get_vendor_products))
       .route("/api/products/{id}", get(routes::products::get_product_by_id).delete(routes::products::delete_product).put(routes::products::update_product))
       
       // Order Routes (including multi-vendor endpoint and PUT support for status updates)
       .route("/api/orders", get(routes::orders::get_orders).post(routes::orders::create_order))
       .route("/api/vendor/orders", get(routes::orders::get_vendor_orders))
       .route("/api/orders/{id}/status", put(routes::orders::update_status).post(routes::orders::update_status))
       .route("/api/orders/{id}/tracking", post(routes::orders::update_tracking))
       .route("/api/orders/{id}/note", post(routes::orders::update_note))
       .route("/api/orders/{id}/cancel", post(routes::orders::cancel_order))
       .route("/api/orders/{id}/payment_status", post(routes::orders::update_payment_status))
       .route("/api/orders/{id}", get(routes::orders::get_order_by_id).delete(routes::orders::delete_order))
       
       // Admin CRM, Sellers & Status Routes
       .route("/api/admin/crm", get(routes::auth::get_admin_crm_customers))
       .route("/api/admin/sellers", get(routes::auth::get_admin_sellers))
       .route("/api/vendors/{id}/status", post(routes::auth::update_vendor_status))

       .route("/api/newsletter", post(routes::newsletter::subscribe))
       .route("/api/admin/analytics", get(routes::analytics::get_admin_analytics))
       .route("/api/reviews", post(routes::reviews::create_review))
       .route("/api/reviews/{productId}", get(routes::reviews::get_reviews))
       .route("/api/auth/delete-account", post(crate::routes::auth::delete_user_account))
       .route("/api/admin/deletion-logs", get(crate::routes::auth::get_deletion_logs))
       .route("/api/aura-ai", post(routes::ai::aura_ai_handler))
       .route("/api/finance/payouts", get(routes::orders::get_payouts))
       .route("/api/finance/payout", post(routes::orders::process_payout))
       .route("/api/cms/popup", get(routes::cms::get_popup).post(routes::cms::update_popup))

       .route("/", get(index_page))
       .route("/index.html", get(index_page))
       .route("/admin", get(admin_page))
       .route("/admin.html", get(admin_page))
       .route("/admin/login", get(admin_login_page))
       .route("/admin-login.html", get(admin_login_page))
       .route("/vendor", get(vendor_page))
       .route("/vendor.html", get(vendor_page))
       .route("/login", get(login_page))
       .route("/login.html", get(login_page))
       .route("/profile", get(profile_page))
       .route("/profile.html", get(profile_page))
       .route("/checkout", get(checkout_page))
       .route("/checkout.html", get(checkout_page))
       .route("/product", get(product_page))
       .route("/product.html", get(product_page))
       .route("/success", get(success_page))
       .route("/health", get(|| async { "OK" }))
       .nest_service("/css", ServeDir::new("public/css"))
       .nest_service("/uploads", ServeDir::new("public/uploads"))
       .nest_service("/js", ServeDir::new("public/js"))
       .fallback_service(ServeDir::new("templates").append_index_html_on_directories(true))
       .layer(axum::extract::DefaultBodyLimit::disable())
       .layer(Extension(database));

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = SocketAddr::from(([0, 0, 0, 0], port.parse::<u16>().unwrap()));

    println!(
        "AuraWear running on http://localhost:3000 - Admin http://localhost:3000/admin"
    );

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn db_seed_admin(db: &mongodb::Database) {
    use mongodb::bson::doc;
    let users_collection = db.collection::<mongodb::bson::Document>("users");
    
    if let Ok(Some(_)) = users_collection.find_one(doc! { "role": "admin" }).await {
        return;
    }

    if let Ok(hashed_password) = bcrypt::hash("Admin@123", bcrypt::DEFAULT_COST) {
        let admin_doc = doc! {
            "name": "System Admin",
            "email": "admin@aurawear.com",
            "password": hashed_password,
            "role": "admin",
            "status": "active",
            "created_at": mongodb::bson::DateTime::now()
        };
        
        let _ = users_collection.insert_one(admin_doc).await;
        println!("Default admin user created: admin@aurawear.com / Admin@123");
    }
}

async fn index_page() -> Html<String> {
    Html(
        tokio::fs::read_to_string("templates/index.html")
            .await
            .unwrap_or_else(|_| "<h1>index.html missing</h1>".into()),
    )
}

async fn admin_page() -> Html<String> {
    Html(
        tokio::fs::read_to_string("templates/admin.html")
            .await
            .unwrap_or_else(|_| "<h1>admin.html missing</h1>".into()),
    )
}

async fn admin_login_page() -> Html<String> {
    Html(
        tokio::fs::read_to_string("templates/admin-login.html")
            .await
            .unwrap_or_else(|_| "<h1>admin-login.html missing</h1>".into()),
    )
}

async fn vendor_page() -> Html<String> {
    Html(
        tokio::fs::read_to_string("templates/vendor.html")
            .await
            .unwrap_or_else(|_| "<h1>vendor.html missing</h1>".into()),
    )
}

async fn login_page() -> Html<String> {
    Html(
        tokio::fs::read_to_string("templates/login.html")
            .await
            .unwrap_or_else(|_| "<h1>login.html missing</h1>".into()),
    )
}

async fn profile_page() -> Html<String> {
    Html(
        tokio::fs::read_to_string("templates/profile.html")
            .await
            .unwrap_or_else(|_| "<h1>profile.html missing</h1>".into()),
    )
}

async fn checkout_page() -> Html<String> {
    Html(
        tokio::fs::read_to_string("templates/checkout.html")
            .await
            .unwrap_or_else(|_| "<h1>checkout.html missing</h1>".into()),
    )
}

async fn product_page() -> Html<String> {
    Html(
        tokio::fs::read_to_string("templates/product.html")
            .await
            .unwrap_or_else(|_| "<h1>product.html missing</h1>".into()),
    )
}

async fn success_page(
    Query(params): Query<HashMap<String, String>>
) -> Html<String> {
    let id = params
        .get("id")
        .cloned()
        .unwrap_or_else(|| {
            "AURAW-".to_string()
                + &chrono::Utc::now().timestamp().to_string()[6..]
        });

    let template = tokio::fs::read_to_string("templates/success.html")
        .await
        .unwrap_or_default();

    let html = if template.is_empty() {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width,initial-scale=1">
    <title>Success - AURAWEAR</title>

    <link
        href="https://fonts.googleapis.com/css2?family=Fraunces&display=swap"
        rel="stylesheet"
    >

    <style>
        body {{
            margin:0;
            background:#FBF8F3;
            font-family:Inter,sans-serif
        }}

        .wrap {{
            min-height:80vh;
            display:grid;
            place-items:center;
            text-align:center;
            padding:40px
        }}

        .card {{
            background:#fff;
            border:1px solid rgba(0,0,0,.12);
            padding:48px;
            max-width:480px
        }}

        h1 {{
            font-family:Fraunces;
            font-size:52px;
            font-weight:300;
            margin:0
        }}

        .id {{
            margin-top:20px;
            font-size:11px;
            letter-spacing:2px;
            opacity:.5;
            border:1px dashed rgba(0,0,0,.2);
            padding:12px
        }}

        .btn {{
            display:inline-block;
            margin-top:20px;
            background:#1A1611;
            color:#fff;
            padding:14px 22px;
            text-decoration:none;
            font-size:11px;
            letter-spacing:2px
        }}
    </style>
</head>

<body>

<nav style="
    padding:20px 4vw;
    border-bottom:1px solid rgba(0,0,0,.12);
    font-family:Fraunces;
    letter-spacing:4px
">
    AURAWEAR
</nav>

<div class="wrap">
    <div class="card">

        <div style="
            width:56px;
            height:56px;
            border-radius:50%;
            background:#1A1611;
            color:#fff;
            display:grid;
            place-items:center;
            margin:0 auto 20px
        ">
            ✓
        </div>

        <h1>
            Order<br>
            <em style="font-family:serif">Placed</em>
        </h1>

        <p style="opacity:.6;margin-top:16px">
            Your archive is being packed.
        </p>

        <div class="id">
            ORDER ID: {}
        </div>

        <a class="btn" href="/">
            BACK TO ARCHIVE
        </a>

    </div>
</div>

<script>
    localStorage.removeItem('aurawear_cart')
</script>

</body>
</html>"#,
            id
        )
    } else {
        template.replace("{{ORDER_ID}}", &id)
    };

    Html(html)
}