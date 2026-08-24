use axum::{Extension, extract::{Path, Multipart}, Json};
use mongodb::{bson::{doc, oid::ObjectId, Document, Bson, Binary, spec::BinarySubtype}, Collection, Database};
use serde_json::{json, Value};

pub async fn upload_image(Extension(db): Extension<Database>, mut multipart: Multipart) -> Json<Value> {
    while let Ok(Some(field)) = multipart.next_field().await {
        if let Some(name) = field.file_name().map(|s| s.to_string()) {
            let data = field.bytes().await.unwrap();
            let id = ObjectId::new();
            let filename = format!("{}_{}", id, name.replace(' ', "_"));
            let _ = std::fs::create_dir_all("public/uploads");
            let _ = std::fs::write(format!("public/uploads/{}", filename), &data);
            let col: Collection<Document> = db.collection("uploads");
            let bin = Binary { subtype: BinarySubtype::Generic, bytes: data.to_vec() };
            let _ = col.insert_one(doc!{"_id": id, "filename": filename.clone(), "data": bin}).await;
            return Json(json!({"success":true, "path": format!("/uploads/{}", filename)}));
        }
    }
    Json(json!({"success":false}))
}

pub async fn get_file(Extension(db): Extension<Database>, Path(id): Path<String>) -> axum::response::Response {
    use axum::{body::Body, http::{header, StatusCode}};
    let col: Collection<Document> = db.collection("uploads");
    
    if let Ok(bytes) = std::fs::read(format!("public/uploads/{}", id)) {
        return axum::response::Response::builder().status(StatusCode::OK).header(header::CONTENT_TYPE, "image/jpeg").body(Body::from(bytes)).unwrap();
    }
    if let Ok(oid) = ObjectId::parse_str(&id) {
        if let Ok(Some(doc)) = col.find_one(doc!{"_id": oid}).await {
            if let Some(Bson::Binary(b)) = doc.get("data") {
                return axum::response::Response::builder().status(StatusCode::OK).header(header::CONTENT_TYPE, "image/jpeg").body(Body::from(b.bytes.clone())).unwrap();
            }
            if let Ok(fname) = doc.get_str("filename") {
                if let Ok(b) = std::fs::read(format!("public/uploads/{}", fname)) {
                    return axum::response::Response::builder().status(StatusCode::OK).header(header::CONTENT_TYPE, "image/jpeg").body(Body::from(b)).unwrap();
                }
            }
        }
    }
    axum::response::Response::builder().status(StatusCode::NOT_FOUND).body(Body::from("not found")).unwrap()
}

pub async fn upload_hero(Extension(db): Extension<Database>, mut multipart: Multipart) -> Json<Value> {
    while let Ok(Some(field)) = multipart.next_field().await {
        let data = field.bytes().await.unwrap();
        let _ = std::fs::create_dir_all("public/uploads");
        let _ = std::fs::write("public/uploads/hero.jpg", &data);
        let col: Collection<Document> = db.collection("hero");
        let _ = col.delete_many(doc!{}).await;
        let bin = Binary { subtype: BinarySubtype::Generic, bytes: data.to_vec() };
        let _ = col.insert_one(doc!{"image": "/uploads/hero.jpg", "data": bin}).await;
        return Json(json!({"success":true, "path": "/uploads/hero.jpg"}));
    }
    Json(json!({"success":false}))
}