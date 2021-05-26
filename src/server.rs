use crate::blob;
use actix_multipart::Multipart;
use actix_web::middleware::Logger;
use actix_web::{delete, get, post, route, web, App, HttpResponse, HttpServer, Responder};
use blob::BlobRef;
use env_logger::Env;
use log;
use sha2::{Digest, Sha256};
use std::io::Write;
use tempfile::NamedTempFile;

use futures::{StreamExt, TryStreamExt};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[route("/blobs/{hash}", method = "GET", method = "HEAD")]
async fn get_blob(web::Path((hash,)): web::Path<(String,)>) -> impl Responder {
    if hash.len() != 64 {
        return HttpResponse::BadRequest().body("Invalid blob reference. Must have 64 chars.");
    }
    let blob_ref = BlobRef::new(&hash);
    if !blob_ref.exists() {
        return HttpResponse::NotFound()
            .body(format!("Could not find blob corresponding to {}", &hash));
    }

    let metadata = blob_ref.metadata().unwrap();
    // TODO: change to stream?
    match blob_ref.content() {
        Ok(content) => HttpResponse::Ok()
            .content_type(metadata.mime_type)
            .header("filename", metadata.filename)
            .body(content),
        Err(_) => HttpResponse::InternalServerError().body("Cannot open file"),
    }
}

#[delete("/blobs/{hash}")]
async fn delete_blob(web::Path((hash,)): web::Path<(String,)>) -> impl Responder {
    if hash.len() != 64 {
        return HttpResponse::BadRequest().body("Invalid blob reference. Must have 64 chars.");
    }
    let blob_ref = BlobRef::new(&hash);
    if !blob_ref.exists() {
        return HttpResponse::NotFound()
            .body(format!("Could not find blob corresponding to {}", &hash));
    }

    match blob_ref.delete() {
        Ok(_) => HttpResponse::Ok().body(""),
        Err(_) => HttpResponse::InternalServerError().body("Cannot delete file"),
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

        let mut tmp_file = web::block(|| NamedTempFile::new_in("/tmp/rustore/.tmp/"))
            .await
            .unwrap();

        let mut hasher = Sha256::new();
        while let Some(Ok(chunk)) = field.next().await {
            match content_type.get_name().unwrap() {
                "file" => {
                    hasher.update(&chunk);
                    tmp_file = web::block(move || tmp_file.write_all(&chunk).map(|_| tmp_file))
                        .await
                        .unwrap();
                }
                _ => (),
            }
        }
        let blob_ref = BlobRef::new(&format!("{:x}", hasher.finalize())[..]);

        let save_path = blob_ref.to_path();
        web::block(move || {
            std::fs::create_dir_all(&save_path).unwrap();
            tmp_file.persist(&save_path.join(&filename))
        })
        .await
        .unwrap();

        log::info!("{} has been created", blob_ref);
        blobs.push(blob_ref)
    }
    let hashes: Vec<&str> = blobs.iter().map(|b| &b.hash[..]).collect();
    HttpResponse::Ok().json(hashes)
}

fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_blob);
    cfg.service(upload_blobs);
    cfg.service(delete_blob);
}

#[actix_web::main]
pub async fn start_server(port: String) -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info,actix_web=debug");
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    HttpServer::new(|| {
        App::new()
            .configure(init_routes)
            .wrap(Logger::new("%r %s %b bytes %D msecs"))
    })
    .bind(format!("127.0.0.1:{}", port))?
    .run()
    .await
}
