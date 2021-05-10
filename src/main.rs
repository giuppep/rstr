use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
mod blob;
use actix_multipart::Multipart;
use blob::Blob;

use futures::{StreamExt, TryStreamExt};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/blobs/{hash}")]
async fn get_blob(path: web::Path<(String,)>) -> impl Responder {
    let hash = path.into_inner().0;
    let blob = Blob::from_hash(&hash);

    match blob {
        Ok(blob) => HttpResponse::Ok().body(format!("Retrieved {}", blob)),
        Err(_) => {
            HttpResponse::Ok().body(format!("Could not find blob corresponding to {}", &hash))
        }
    }
}

#[post("/blobs")]
async fn upload_blobs(mut payload: Multipart) -> impl Responder {
    let mut blobs: Vec<Blob> = Vec::new();
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();

        let filename = content_type.get_filename().unwrap();
        let filename = sanitize_filename::sanitize(filename);

        let mut contents: Vec<u8> = Vec::new();

        while let Some(Ok(chunk)) = field.next().await {
            // let content_disp = field.content_disposition().unwrap();
            match content_type.get_name().unwrap() {
                "file" => contents.extend_from_slice(&chunk.to_vec()),
                _ => (),
            }
        }
        let blob = Blob::from_content(contents, &filename);
        println!("{} has been created", blob);
        blob.save()
            .expect("something went wrong when saving the blob");
        blobs.push(blob)
    }
    let hashes: Vec<&str> = blobs.iter().map(|b| &b.hash[..]).collect();
    println!("{:?}", hashes);
    HttpResponse::Ok().body(hashes.join("\n"))
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
