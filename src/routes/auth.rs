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
    pub role: Option<String>,
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

#[derive(Deserialize)]
pub struct StatusBody {
    pub status: String,
    pub email: Option<String>,
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
    
    let mut role = body.role.clone().unwrap_or_else(|| "customer".to_string()).to_lowercase();
    if role == "customer" && (email.contains("seller") || email.contains("vendor")) {
        role = "seller".to_string();
    }

    let initial_status = if role == "seller" || role == "vendor" { "pending" } else { "active" };

    let last_name = body.last_name.clone().unwrap_or_default();
    let phone = body.phone.clone().unwrap_or_default();

    let doc = doc!{
        "email": email.clone(),
        "password": hashed,
        "first_name": final_first.clone(),
        "last_name": last_name.clone(),
        "phone": phone.clone(),
        "role": role.clone(),
        "status": initial_status,
        "avatar": "",
        "created_at": chrono::Utc::now().to_rfc3339()
    };
    
    let insert_res = col.insert_one(doc).await;
    let user_id = match insert_res {
        Ok(r) => r.inserted_id.as_object_id().map(|o| o.to_hex()).unwrap_or_default(),
        Err(_) => "".to_string(),
    };

    let name = if !final_first.is_empty() { 
        format!("{} {}", final_first, last_name).trim().to_string() 
    } else { 
        "User".to_string() 
    };

    Json(json!({
        "success": true, 
        "token": email.clone(), 
        "email": email.clone(),
        "role": role.clone(),
        "status": initial_status,
        "user": {
            "id": user_id,
            "name": name,
            "email": email,
            "role": role,
            "status": initial_status
        }
    }))
}

pub async fn login(Extension(db): Extension<Database>, Json(body): Json<AuthBody>) -> Json<Value> {
    let col: Collection<Document> = db.collection("users");
    let email = body.email.to_lowercase().trim().to_string();
    
    let user_opt = match col.find_one(doc!{"email": &email}).await {
        Ok(opt) => opt,
        Err(_) => None
    };

    if let Some(u) = user_opt {
        let stored = u.get_str("password").unwrap_or("");
        let bcrypt_check = bcrypt::verify(&body.password, stored);
        let bcrypt_ok = match bcrypt_check {
            Ok(res) => res,
            Err(_) => false
        };

        let ok = bcrypt_ok || stored == body.password;
        if ok {
            let user_id = if let Ok(oid) = u.get_object_id("_id") {
                oid.to_hex()
            } else if let Ok(s) = u.get_str("_id") {
                s.to_string()
            } else {
                u.get("_id").map(|v| v.to_string()).unwrap_or_default()
            };

            let first = u.get_str("first_name").unwrap_or("");
            let last = u.get_str("last_name").unwrap_or("");
            let name = if !first.is_empty() { format!("{} {}", first, last).trim().to_string() } else { u.get_str("name").unwrap_or("User").to_string() };
            let status = u.get_str("status").unwrap_or("active");
            
            let mut role = u.get_str("role").unwrap_or("customer").to_lowercase();
            if role == "customer" && (email.contains("seller") || email.contains("vendor")) {
                role = "seller".to_string();
            }

            return Json(json!({
                "success": true, 
                "email": email.clone(), 
                "token": email.clone(),
                "role": role.clone(),
                "status": status,
                "user": {
                    "id": user_id,
                    "name": name,
                    "email": email,
                    "role": role,
                    "status": status
                }
            }));
        }
    }
    
    Json(json!({"success":false, "error":"Invalid email or password"}))
}

pub async fn me(Extension(db): Extension<Database>, Json(body): Json<MeBody>) -> Json<Value> {
    let col: Collection<Document> = db.collection("users");
    let email = body.token.to_lowercase().trim().to_string();
    match col.find_one(doc!{"email": &email}).await {
        Ok(Some(u)) => {
            let user_id = if let Ok(oid) = u.get_object_id("_id") {
                oid.to_hex()
            } else if let Ok(s) = u.get_str("_id") {
                s.to_string()
            } else {
                u.get("_id").map(|v| v.to_string()).unwrap_or_default()
            };

            let first = u.get_str("first_name").unwrap_or("").to_string();
            let last = u.get_str("last_name").unwrap_or("").to_string();
            let phone = u.get_str("phone").unwrap_or("").to_string();
            let avatar = u.get_str("avatar").unwrap_or("").to_string();
            let status = u.get_str("status").unwrap_or("active").to_string();
            
            let mut role = u.get_str("role").unwrap_or("customer").to_lowercase();
            if role == "customer" && (email.contains("seller") || email.contains("vendor")) {
                role = "seller".to_string();
            }

            let name = if !first.is_empty() { format!("{} {}", first, last).trim().to_string() } else { "User".to_string() };
            Json(json!({
                "success": true,
                "name": name, "email": email, "phone": phone, "first_name": first, "last_name": last, "avatar": avatar, "role": role, "status": status,
                "user": { "_id": user_id, "id": user_id, "name": name, "email": email, "phone": phone, "first_name": first, "last_name": last, "avatar": avatar, "role": role, "status": status }
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
    let clean_id = id.trim().to_string();
    
    let filter = if let Ok(oid) = ObjectId::parse_str(&clean_id) {
        doc! { "$or": [ { "_id": oid }, { "_id": &clean_id }, { "email": clean_id.to_lowercase() } ] }
    } else {
        doc! { "$or": [ { "_id": &clean_id }, { "email": clean_id.to_lowercase() } ] }
    };

    match col.delete_one(filter).await {
        Ok(r) if r.deleted_count > 0 => Json(json!({"success": true})),
        Ok(_) => Json(json!({"success": false, "error": "User not found"})),
        Err(e) => Json(json!({"success": false, "error": e.to_string()}))
    }
}

pub async fn anonymize_user(Extension(db): Extension<Database>, Path(id): Path<String>) -> Json<Value> {
    let col: Collection<Document> = db.collection("users");
    let clean_id = id.trim().to_string();
    
    let filter = if let Ok(oid) = ObjectId::parse_str(&clean_id) {
        doc! { "$or": [ { "_id": oid }, { "_id": &clean_id }, { "email": clean_id.to_lowercase() } ] }
    } else {
        doc! { "$or": [ { "_id": &clean_id }, { "email": clean_id.to_lowercase() } ] }
    };

    match col.delete_one(filter).await {
        Ok(r) if r.deleted_count > 0 => Json(json!({"success": true})),
        Ok(_) => Json(json!({"success": false, "error": "User not found"})),
        Err(e) => Json(json!({"success": false, "error": e.to_string()}))
    }
}

pub async fn delete_user_account(
    Extension(db): Extension<Database>,
    Json(payload): Json<DeleteAccountRequest>,
) -> Json<Value> {
    let users_col: Collection<Document> = db.collection("users");
    let logs_col: Collection<Document> = db.collection("account_deletion_logs");
    let email = payload.email.to_lowercase().trim().to_string();

    let user_opt = users_col.find_one(doc!{"email": &email}).await.unwrap_or(None);

    if let Some(user) = user_opt {
        let first = user.get_str("first_name").unwrap_or("");
        let last = user.get_str("last_name").unwrap_or("");
        let name = if !first.is_empty() { format!("{} {}", first, last) } else { user.get_str("name").unwrap_or("User").to_string() };
        let phone = user.get_str("phone").unwrap_or("N/A");

        let log_doc = doc! {
            "name": name,
            "email": &email,
            "phone": phone,
            "reason": &payload.reason,
            "custom_reason": payload.custom_reason.unwrap_or_default(),
            "deleted_at": chrono::Utc::now().to_rfc3339()
        };
        let _ = logs_col.insert_one(log_doc).await;
        let _ = users_col.delete_one(doc!{"email": &email}).await;

        Json(json!({"success": true}))
    } else {
        Json(json!({"success": false, "error": "User not found"}))
    }
}

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

pub async fn get_admin_crm_customers(Extension(db): Extension<Database>) -> Json<Value> {
    let users_col: Collection<Document> = db.collection("users");
    let orders_col: Collection<Document> = db.collection("orders");

    let mut user_cursor = match users_col.find(doc!{}).await {
        Ok(c) => c,
        Err(_) => return Json(json!({"success": false, "error": "Failed to fetch users"}))
    };

    let mut all_users = Vec::new();
    while let Some(Ok(user)) = user_cursor.next().await {
        all_users.push(user);
    }

    let mut customer_users = Vec::new();
    for user in all_users {
        let role = user.get_str("role").unwrap_or("customer").to_lowercase();
        let email = user.get_str("email").unwrap_or_default().to_lowercase();
        if role == "seller" || role == "vendor" || role == "admin" || email.contains("seller") || email.contains("vendor") {
            continue;
        }
        customer_users.push(user);
    }

    let total_registered = customer_users.len() as u64;
    let active_users = customer_users.iter().filter(|u| u.get_str("status").unwrap_or("active") == "active").count() as u64;

    let mut order_cursor = match orders_col.find(doc!{}).await {
        Ok(c) => c,
        Err(_) => return Json(json!({"success": false, "error": "Failed to fetch orders"}))
    };

    let mut user_stats: std::collections::HashMap<String, (i64, f64)> = std::collections::HashMap::new();
    let mut overall_total_spend: f64 = 0.0;

    while let Some(Ok(order)) = order_cursor.next().await {
        let email = order.get_str("email").or_else(|_| order.get_str("user_email")).unwrap_or_default().to_lowercase();
        let total = order.get_f64("total").or_else(|_| order.get_f64("price")).unwrap_or_else(|_| {
            order.get_i64("total").map(|v| v as f64).unwrap_or(0.0)
        });

        if !email.is_empty() {
            let entry = user_stats.entry(email).or_insert((0, 0.0));
            entry.0 += 1;    
            entry.1 += total; 
        }
        overall_total_spend += total;
    }

    let mut customers_list = Vec::new();
    for user in customer_users {
        let user_id = if let Ok(oid) = user.get_object_id("_id") {
            oid.to_hex()
        } else if let Ok(s) = user.get_str("_id") {
            s.to_string()
        } else {
            user.get("_id").map(|v| v.to_string()).unwrap_or_default()
        };

        let name = if let Ok(n) = user.get_str("name") {
            if !n.is_empty() { n.to_string() } else {
                let first = user.get_str("first_name").unwrap_or("");
                let last = user.get_str("last_name").unwrap_or("");
                if !first.is_empty() { format!("{} {}", first, last) } else { "Unknown".to_string() }
            }
        } else {
            let first = user.get_str("first_name").unwrap_or("");
            let last = user.get_str("last_name").unwrap_or("");
            if !first.is_empty() { format!("{} {}", first, last) } else { "Unknown".to_string() }
        };

        let email = user.get_str("email").unwrap_or_default();
        let status = user.get_str("status").unwrap_or("active");
        
        let stats = user_stats.get(&email.to_lowercase()).cloned().unwrap_or((0, 0.0));

        customers_list.push(json!({
            "id": user_id,
            "_id": user_id,
            "name": name,
            "email": email,
            "status": status,
            "total_orders": stats.0,
            "lifetime_spend": stats.1
        }));
    }

    Json(json!({
        "success": true,
        "metrics": {
            "total_registered": total_registered,
            "active_users": active_users,
            "overall_total_spend": overall_total_spend
        },
        "customers": customers_list
    }))
}

pub async fn get_admin_sellers(Extension(db): Extension<Database>) -> Json<Value> {
    let users_col: Collection<Document> = db.collection("users");
    let products_col: Collection<Document> = db.collection("products");

    let mut user_cursor = match users_col.find(doc!{}).await {
        Ok(c) => c,
        Err(_) => return Json(json!({"success": false, "error": "Failed to fetch users"}))
    };

    let mut sellers_list = Vec::new();
    while let Some(Ok(user)) = user_cursor.next().await {
        let role = user.get_str("role").unwrap_or("customer").to_lowercase();
        let email = user.get_str("email").unwrap_or_default().to_lowercase();
        
        if role == "seller" || role == "vendor" || email.contains("seller") || email.contains("vendor") {
            let user_id = if let Ok(oid) = user.get_object_id("_id") {
                oid.to_hex()
            } else if let Ok(s) = user.get_str("_id") {
                s.to_string()
            } else {
                user.get("_id").map(|v| v.to_string()).unwrap_or_default()
            };

            let store_name = user.get_str("storeName").or_else(|_| user.get_str("store_name")).unwrap_or_else(|_| user.get_str("name").unwrap_or("Archive Store"));
            let owner_name = user.get_str("name").unwrap_or("Store Owner");
            let status = user.get_str("status").unwrap_or("pending");

            let product_count = products_col.count_documents(doc!{ 
                "$or": [
                    { "vendor": owner_name },
                    { "store": store_name },
                    { "email": &email }
                ]
            }).await.unwrap_or(0);

            sellers_list.push(json!({
                "id": user_id,
                "_id": user_id,
                "store": store_name,
                "owner": owner_name,
                "email": email,
                "products": product_count,
                "earnings": 0.0,
                "status": status,
                "comm": 15
            }));
        }
    }

    Json(json!({
        "success": true,
        "sellers": sellers_list
    }))
}

pub async fn update_vendor_status(
    Extension(db): Extension<Database>,
    Path(id): Path<String>,
    Json(payload): Json<StatusBody>,
) -> Json<Value> {
    let users_col: Collection<Document> = db.collection("users");
    let new_status = &payload.status;

    let filter = if let Ok(oid) = ObjectId::parse_str(&id) {
        doc! { "$or": [{ "_id": oid }, { "_id": id.clone() }] }
    } else {
        doc! { "_id": id.clone() }
    };

    let update = doc! { "$set": { "status": new_status } };

    match users_col.update_one(filter, update).await {
        Ok(r) if r.matched_count > 0 => Json(json!({"success": true})),
        Ok(_) => {
            if let Some(ref email) = payload.email {
                let email_filter = doc! { "email": email.to_lowercase().trim() };
                if let Ok(res) = users_col.update_one(email_filter, doc! { "$set": { "status": new_status } }).await {
                    if res.matched_count > 0 {
                        return Json(json!({"success": true}));
                    }
                }
            }
            Json(json!({"success": false, "error": "Vendor not found or status unchanged"}))
        },
        Err(e) => Json(json!({"success": false, "error": e.to_string()}))
    }
}