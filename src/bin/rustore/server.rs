use crate::security::validate_token;
use crate::settings::ServerSettings;
use actix_multipart::Multipart;
use actix_service::Service;
use actix_web::middleware::Logger;
use actix_web::{delete, get, post, route, web, App, HttpResponse, HttpServer, Responder};
use env_logger::Env;
use futures::future::{ok, Either};
use futures::{StreamExt, TryStreamExt};
use rustore::BlobRef;
use serde::Serialize;
use sha2::Digest;
use std::io::Write;
use tempfile::NamedTempFile;

/// Struct representing the json payload returned to the user upon error
#[derive(Serialize)]
struct ErrorResponse {
    /// The name of the error, can be used to match to error classes in client.
    name: String,
    /// A message providing more detail on the error.
    message: String,
}

impl ErrorResponse {
    fn new(name: &str, message: &str) -> Self {
        ErrorResponse {
            name: String::from(name),
            message: String::from(message),
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
            return HttpResponse::NotFound().json(ErrorResponse::new(
                "BlobNotFound",
                &format!("Could not find blob corresponding to {}", &hash),
            ))
        }
        Ok(blob_ref) => blob_ref,
        Err(e) => {
            return HttpResponse::BadRequest()
                .json(ErrorResponse::new("InvalidReference", &e.to_string()));
        }
    };

    let metadata = blob_ref.metadata().unwrap();
    // TODO: change to stream?
    match blob_ref.content() {
        Ok(content) => HttpResponse::Ok()
            .content_type(metadata.mime_type)
            .header("filename", metadata.filename)
            .header(
                "created",
                metadata
                    .created
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, false),
            )
            .header("content-disposition", "attachment")
            .body(content),
        Err(_) => HttpResponse::InternalServerError()
            .json(ErrorResponse::new("ServerError", "Cannot open the file")),
    }
}

#[delete("/blobs/{hash}")]
async fn delete_blob(web::Path((hash,)): web::Path<(String,)>) -> impl Responder {
    let blob_ref = match BlobRef::new(&hash) {
        Ok(blob_ref) if !blob_ref.exists() => {
            return HttpResponse::NotFound().json(ErrorResponse::new(
                "BlobNotFound",
                &format!("Could not find blob corresponding to {}", &hash),
            ));
        }
        Ok(blob_ref) => blob_ref,
        Err(e) => {
            return HttpResponse::BadRequest()
                .json(ErrorResponse::new("InvalidReference", &e.to_string()));
        }
    };

    match blob_ref.delete() {
        Ok(_) => HttpResponse::Ok().body(""),
        Err(_) => HttpResponse::InternalServerError()
            .json(ErrorResponse::new("ServerError", "Cannot delete the file")),
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
        let blob_ref = BlobRef::from(hasher);

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
    let hashes: Vec<&str> = blobs.iter().map(BlobRef::reference).collect();
    HttpResponse::Ok().json(hashes)
}

fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(app_status);
    cfg.service(get_blob);
    cfg.service(upload_blobs);
    cfg.service(delete_blob);
}

#[actix_web::main]
pub async fn start_server(settings: ServerSettings) -> std::io::Result<()> {
    std::env::set_var(
        "RUST_LOG",
        format!("{},actix_web={}", settings.log_level, settings.log_level),
    );
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    settings.set_env_vars();
    settings.create_dirs();

    HttpServer::new(|| {
        App::new()
            .wrap_fn(|req, srv| {
                let auth_token = req.headers().get("X-Auth-Token");
                match auth_token {
                    Some(auth_token) if validate_token(auth_token.to_str().unwrap()) => {
                        Either::Left(srv.call(req))
                    }
                    _ => Either::Right(ok(req.into_response(HttpResponse::Unauthorized().json(
                        ErrorResponse::new("InvalidToken","Unauthorized: the provided authentication token does not match our records."),
                    )))),
                }
            })
            .configure(init_routes)
            .wrap(Logger::new("%r %s %b bytes %D msecs"))
    })
    .bind(format!("127.0.0.1:{}", settings.port.to_string()))?
    .run()
    .await
}
