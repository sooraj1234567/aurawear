use serde::Deserialize;
#[derive(Deserialize)] pub struct RegisterBody { pub email: String, pub password: String, pub name: String, pub phone: Option<String> }
#[derive(Deserialize)] pub struct LoginBody { pub email: String, pub password: String }