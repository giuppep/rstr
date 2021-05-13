use super::blob::BlobRef;
use actix_multipart::Multipart;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use sha2::{Digest, Sha256};
use std::io::Write;
use tempfile::NamedTempFile;

use futures::{StreamExt, TryStreamExt};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/blobs/{hash}")]
async fn get_blob(web::Path((hash,)): web::Path<(String,)>) -> impl Responder {
    let blob_ref = BlobRef::new(&hash);
    if !blob_ref.exists() {
        return HttpResponse::NotFound()
            .body(format!("Could not find blob corresponding to {}", &hash));
    }

    let mimetype = blob_ref.get_mime().unwrap();
    // TODO: change to stream?
    match blob_ref.get_content() {
        Ok(content) => HttpResponse::Ok().content_type(mimetype).body(content),
        Err(_) => HttpResponse::InternalServerError().body("Cannot open file"),
    }
}

#[get("/blobs/{hash}/metadata")]
async fn get_blob_metadata(web::Path((hash,)): web::Path<(String,)>) -> impl Responder {
    let blob_ref = BlobRef::new(&hash);
    if !blob_ref.exists() {
        return HttpResponse::NotFound()
            .body(format!("Could not find blob corresponding to {}", &hash));
    }

    let metadata = blob_ref.get_metadata();
    match metadata {
        Ok(metadata) => HttpResponse::Ok().json(metadata),
        Err(_) => HttpResponse::InternalServerError().body("Cannot retrieve metadata"),
    }
}

#[post("/blobs")]
async fn upload_blobs(mut payload: Multipart) -> impl Responder {
    // TODO: handle errors
    let mut blobs: Vec<BlobRef> = Vec::new();
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();

        let filename = content_type.get_filename().unwrap_or("file");
        let filename = sanitize_filename::sanitize(filename);

        let mut tmp_file = NamedTempFile::new_in("/tmp/rustore/.tmp/").unwrap();
        let mut hasher = Sha256::new();
        while let Some(Ok(chunk)) = field.next().await {
            match content_type.get_name().unwrap() {
                "file" => {
                    tmp_file.write_all(&chunk).unwrap();
                    hasher.update(&chunk)
                }
                _ => (),
            }
        }
        let blob_ref = BlobRef::new(&format!("{:x}", hasher.finalize())[..]);
        println!("{} has been created", blob_ref);

        std::fs::create_dir_all(&blob_ref.to_path()).unwrap();
        tmp_file
            .persist(blob_ref.to_path().join(&filename))
            .unwrap();
        blobs.push(blob_ref)
    }
    let hashes: Vec<&str> = blobs.iter().map(|b| &b.hash[..]).collect();
    HttpResponse::Ok().json(hashes)
}

fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_blob);
    cfg.service(get_blob_metadata);
    cfg.service(upload_blobs);
}

#[actix_web::main]
pub async fn start_server(port: String) -> std::io::Result<()> {
    HttpServer::new(|| App::new().configure(init_routes))
        .bind(format!("127.0.0.1:{}", port))?
        .run()
        .await
}
