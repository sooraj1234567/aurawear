use axum::{Extension, Json, extract::Path};
use mongodb::{bson::doc, Database, Collection};
use mongodb::bson::{Document, oid::ObjectId};
use serde::Deserialize;
use serde_json::{Value, json};
use futures_util::StreamExt;

#[derive(Deserialize)]
pub struct AuthBody {
    pub email: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub name: Option<String>,
    pub phone: Option<String>,
}

#[derive(Deserialize)]
pub struct MeBody { pub token: String }

#[derive(Deserialize)]
pub struct UpdateBody {
    pub token: String,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Deserialize)]
pub struct DeleteAccountRequest {
    pub email: String,
    pub reason: String,
    pub custom_reason: Option<String>,
}

pub async fn register(Extension(db): Extension<Database>, Json(body): Json<AuthBody>) -> Json<Value> {
    let col: Collection<Document> = db.collection("users");
    let email = body.email.to_lowercase().trim().to_string();
    if col.find_one(doc!{"email": &email}).await.unwrap_or(None).is_some() {
        return Json(json!({"success":false, "error":"Email already exists"}));
    }
    let hashed = bcrypt::hash(&body.password, 10).unwrap_or(body.password.clone());
    let f_name = body.first_name.clone().unwrap_or_default();
    let final_first = if !f_name.is_empty() { f_name } else { body.name.clone().unwrap_or_default() };
    let doc = doc!{
        "email": email.clone(),
        "password": hashed,
        "first_name": final_first,
        "last_name": body.last_name.unwrap_or_default(),
        "phone": body.phone.unwrap_or_default(),
        "avatar": "",
        "created_at": chrono::Utc::now().to_rfc3339()
    };
    let _ = col.insert_one(doc).await;
    Json(json!({"success":true, "token": email, "email": email}))
}

pub async fn login(Extension(db): Extension<Database>, Json(body): Json<AuthBody>) -> Json<Value> {
    let col: Collection<Document> = db.collection("users");
    let email = body.email.to_lowercase().trim().to_string();
    let user_opt = col.find_one(doc!{"email": &email}).await.unwrap_or(None);
    if let Some(u) = user_opt {
        let stored = u.get_str("password").unwrap_or("");
        let ok = bcrypt::verify(&body.password, stored).unwrap_or(false) || stored == body.password;
        if ok {
            return Json(json!({"success":true, "email": email, "token": email}));
        }
    }
    Json(json!({"success":false, "error":"Invalid email or password"}))
}

pub async fn me(Extension(db): Extension<Database>, Json(body): Json<MeBody>) -> Json<Value> {
    let col: Collection<Document> = db.collection("users");
    let email = body.token.to_lowercase().trim().to_string();
    match col.find_one(doc!{"email": &email}).await {
        Ok(Some(u)) => {
            let first = u.get_str("first_name").unwrap_or("").to_string();
            let last = u.get_str("last_name").unwrap_or("").to_string();
            let phone = u.get_str("phone").unwrap_or("").to_string();
            let avatar = u.get_str("avatar").unwrap_or("").to_string();
            let name = if !first.is_empty() { format!("{} {}", first, last).trim().to_string() } else { "User".to_string() };
            Json(json!({
                "success": true,
                "name": name, "email": email, "phone": phone, "first_name": first, "last_name": last, "avatar": avatar,
                "user": { "name": name, "email": email, "phone": phone, "first_name": first, "last_name": last, "avatar": avatar }
            }))
        },
        _ => Json(json!({"success":false, "error":"User not found"}))
    }
}

pub async fn update(Extension(db): Extension<Database>, Json(body): Json<UpdateBody>) -> Json<Value> {
    let col: Collection<Document> = db.collection("users");
    let email = body.token.to_lowercase().trim().to_string();
    if email.is_empty() { return Json(json!({"success":false, "error":"Invalid token"})); }
    let mut set_doc = Document::new();
    if let Some(name) = body.name {
        if !name.trim().is_empty() {
            let parts: Vec<&str> = name.trim().splitn(2, ' ').collect();
            set_doc.insert("first_name", parts[0].to_string());
            set_doc.insert("last_name", parts.get(1).unwrap_or(&"").to_string());
        }
    }
    if let Some(f) = body.first_name { if !f.trim().is_empty() { set_doc.insert("first_name", f.trim().to_string()); } }
    if let Some(l) = body.last_name { set_doc.insert("last_name", l.trim().to_string()); }
    if let Some(p) = body.phone { set_doc.insert("phone", p.trim().to_string()); }
    if let Some(a) = body.avatar { set_doc.insert("avatar", a); }
    if set_doc.is_empty() { return Json(json!({"success":false, "error":"Nothing to update"})); }
    match col.update_one(doc!{"email": &email}, doc!{"$set": set_doc}).await {
        Ok(_) => Json(json!({"success":true})),
        Err(e) => Json(json!({"success":false, "error": e.to_string()}))
    }
}

pub async fn get_users(Extension(db): Extension<Database>) -> Json<Value> {
    let col: Collection<Document> = db.collection("users");
    let mut cursor = match col.find(doc!{}).await {
        Ok(c) => c,
        Err(e) => return Json(json!({"success": false, "error": e.to_string()}))
    };
    let mut users = Vec::new();
    while let Some(Ok(mut u)) = cursor.next().await {
        u.remove("password");
        users.push(u);
    }
    Json(json!({"success": true, "users": users}))
}

pub async fn delete_user(Extension(db): Extension<Database>, Path(id): Path<String>) -> Json<Value> {
    let col: Collection<Document> = db.collection("users");
    let oid = match ObjectId::parse_str(&id) {
        Ok(o) => o,
        Err(_) => return Json(json!({"success":false,"error":"Invalid id"}))
    };
    match col.delete_one(doc!{"_id": oid}).await {
        Ok(r) if r.deleted_count > 0 => Json(json!({"success":true})),
        Ok(_) => Json(json!({"success":false,"error":"User not found"})),
        Err(e) => Json(json!({"success":false,"error":e.to_string()}))
    }
}

// NEW: Handler for user self-deleting their account with reasons logging
pub async fn delete_user_account(
    Extension(db): Extension<Database>,
    Json(payload): Json<DeleteAccountRequest>,
) -> Json<Value> {
    let users_col: Collection<Document> = db.collection("users");
    let logs_col: Collection<Document> = db.collection("account_deletion_logs");
    let email = payload.email.to_lowercase().trim().to_string();

    // 1. Fetch user info before deleting
    let user_opt = users_col.find_one(doc!{"email": &email}).await.unwrap_or(None);

    if let Some(user) = user_opt {
        let first = user.get_str("first_name").unwrap_or("");
        let last = user.get_str("last_name").unwrap_or("");
        let name = if !first.is_empty() { format!("{} {}", first, last) } else { user.get_str("name").unwrap_or("User").to_string() };
        let phone = user.get_str("phone").unwrap_or("N/A");

        // 2. Log deletion for admin panel tracking
        let log_doc = doc! {
            "name": name,
            "email": &email,
            "phone": phone,
            "reason": &payload.reason,
            "custom_reason": payload.custom_reason.unwrap_or_default(),
            "deleted_at": chrono::Utc::now().to_rfc3339()
        };
        let _ = logs_col.insert_one(log_doc).await;

        // 3. Delete user document
        let _ = users_col.delete_one(doc!{"email": &email}).await;

        Json(json!({"success": true}))
    } else {
        Json(json!({"success": false, "error": "User not found"}))
    }
}

// NEW: Handler for Admin to retrieve account deletion logs
pub async fn get_deletion_logs(Extension(db): Extension<Database>) -> Json<Value> {
    let logs_col: Collection<Document> = db.collection("account_deletion_logs");
    let mut cursor = match logs_col.find(doc!{}).await {
        Ok(c) => c,
        Err(e) => return Json(json!({"success": false, "error": e.to_string()}))
    };
    let mut logs = Vec::new();
    while let Some(Ok(l)) = cursor.next().await {
        logs.push(l);
    }
    Json(json!({"success": true, "logs": logs}))
}