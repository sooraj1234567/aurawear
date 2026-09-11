use serde::Deserialize;

#[derive(Deserialize)]
pub struct RegisterBody {
    pub email: String,
    pub password: String,
    pub name: String,
    pub phone: Option<String>,
    pub role: Option<String>, // e.g., "customer", "vendor", or "admin"
}

#[derive(Deserialize)]
pub struct LoginBody {
    pub email: String,
    pub password: String,
}