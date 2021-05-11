use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
mod blob;
use actix_multipart::Multipart;
use blob::{Blob, BlobRef};
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
    let blob = Blob::from_hash(blob_ref);

    match blob {
        Ok(blob) => HttpResponse::Ok()
            .content_type(blob.get_mime())
            .body(blob.content),
        Err(_) => {
            HttpResponse::Ok().body(format!("Could not find blob corresponding to {}", &hash))
        }
    }
}

#[post("/blobs")]
async fn upload_blobs(mut payload: Multipart) -> impl Responder {
    // TODO: handle errors
    let mut blobs: Vec<BlobRef> = Vec::new();
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();

        let filename = content_type.get_filename().unwrap();
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(get_blob)
            .service(upload_blobs)
    })
    .bind("127.0.0.1:3123")?
    .run()
    .await
}
