use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
mod blob;
use blob::Blob;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/blob/{hash}")]
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hello).service(get_blob))
        .bind("127.0.0.1:3123")?
        .run()
        .await
}
