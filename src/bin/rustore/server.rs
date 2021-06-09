use actix_multipart::Multipart;
use actix_web::middleware::Logger;
use actix_web::{delete, get, post, route, web, App, HttpResponse, HttpServer, Responder};
use env_logger::Env;
use futures::{StreamExt, TryStreamExt};
use log;
use rustore::blob::BlobRef;
use sha2::Digest;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

pub struct Config {
    port: u16,
    log_level: log::Level,
    tmp_folder: PathBuf,
}

impl Config {
    pub fn new(port: u16, log_level: log::Level, tmp_folder: PathBuf) -> Config {
        Config {
            port,
            log_level,
            tmp_folder,
        }
    }
}

#[get("/status")]
async fn app_status() -> impl Responder {
    HttpResponse::Ok()
}

#[route("/blobs/{hash}", method = "GET", method = "HEAD")]
async fn get_blob(web::Path((hash,)): web::Path<(String,)>) -> impl Responder {
    let blob_ref = match BlobRef::new(&hash) {
        Ok(blob_ref) if !blob_ref.exists() => {
            return HttpResponse::NotFound()
                .body(format!("Could not find blob corresponding to {}", &hash));
        }
        Ok(blob_ref) => blob_ref,
        Err(e) => {
            return HttpResponse::BadRequest().body(e.to_string());
        }
    };

    let metadata = blob_ref.metadata().unwrap();
    // TODO: change to stream?
    match blob_ref.content() {
        Ok(content) => HttpResponse::Ok()
            .content_type(metadata.mime_type)
            .header("filename", metadata.filename)
            .header("created", metadata.created.to_rfc3339())
            .body(content),
        Err(_) => HttpResponse::InternalServerError().body("Cannot open file"),
    }
}

#[delete("/blobs/{hash}")]
async fn delete_blob(web::Path((hash,)): web::Path<(String,)>) -> impl Responder {
    let blob_ref = match BlobRef::new(&hash) {
        Ok(blob_ref) if !blob_ref.exists() => {
            return HttpResponse::NotFound()
                .body(format!("Could not find blob corresponding to {}", &hash));
        }
        Ok(blob_ref) => blob_ref,
        Err(e) => {
            return HttpResponse::BadRequest().body(e.to_string());
        }
    };

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

        let mut tmp_file =
            web::block(|| NamedTempFile::new_in(std::env::var("RUSTORE_TMP_FOLDER").unwrap()))
                .await
                .unwrap();

        let mut hasher = BlobRef::hasher();
        while let Some(Ok(chunk)) = field.next().await {
            if content_type.get_name().unwrap() == "file" {
                hasher.update(&chunk);
                tmp_file = web::block(move || tmp_file.write_all(&chunk).map(|_| tmp_file))
                    .await
                    .unwrap();
            }
        }
        let blob_ref = BlobRef::from_hasher(hasher);

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
    let hashes: Vec<&str> = blobs.iter().map(|b| b.reference()).collect();
    HttpResponse::Ok().json(hashes)
}

fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(app_status);
    cfg.service(get_blob);
    cfg.service(upload_blobs);
    cfg.service(delete_blob);
}

#[actix_web::main]
pub async fn start_server(config: Config) -> std::io::Result<()> {
    std::env::set_var(
        "RUST_LOG",
        format!("{},actix_web={}", config.log_level, config.log_level),
    );
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    std::env::set_var("RUSTORE_TMP_FOLDER", &config.tmp_folder);
    std::fs::create_dir_all(&config.tmp_folder).unwrap();

    HttpServer::new(|| {
        App::new()
            .configure(init_routes)
            .wrap(Logger::new("%r %s %b bytes %D msecs"))
    })
    .bind(format!("127.0.0.1:{}", config.port.to_string()))?
    .run()
    .await
}
