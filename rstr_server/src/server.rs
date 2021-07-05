use crate::error::ErrorResponse;
use crate::security::validate_token;
use crate::settings::Settings;
use actix_multipart::Multipart;
use actix_web::dev::Service;
use actix_web::middleware::Logger;
use actix_web::{delete, get, post, route, web, App, HttpResponse, HttpServer, Responder};
use env_logger::Env;
use futures::future::{ok, Either};
use futures::{StreamExt, TryStreamExt};
use rstr::{BlobRef, BlobStore, Sha2Digest};
use std::io::Write;
use tempfile::NamedTempFile;

#[get("/status")]
async fn app_status() -> impl Responder {
    HttpResponse::Ok()
}

#[route("/blobs/{hash}", method = "GET", method = "HEAD")]
async fn get_blob(
    web::Path((hash,)): web::Path<(String,)>,
    data: web::Data<Settings>,
) -> impl Responder {
    let blob_ref = match BlobRef::new(&hash) {
        Ok(blob_ref) => blob_ref,
        Err(e) => return HttpResponse::from(ErrorResponse::from(e)),
    };

    let blob_store = BlobStore::new(&data.data_store_dir).unwrap();

    // TODO: change to stream?
    match blob_store.get(&blob_ref) {
        Ok(content) => {
            let metadata = blob_store.metadata(&blob_ref).unwrap();
            HttpResponse::Ok()
                .header("created", metadata.created_str())
                .header("filename", metadata.filename)
                .header("content-disposition", "attachment")
                .content_type(metadata.mime_type)
                .body(content)
        }
        Err(e) => HttpResponse::from(ErrorResponse::from(e)),
    }
}

#[delete("/blobs/{hash}")]
async fn delete_blob(
    web::Path((hash,)): web::Path<(String,)>,
    data: web::Data<Settings>,
) -> impl Responder {
    let blob_ref = match BlobRef::new(&hash) {
        Ok(blob_ref) => blob_ref,
        Err(e) => {
            return HttpResponse::from(ErrorResponse::from(e));
        }
    };

    let blob_store = BlobStore::new(&data.data_store_dir).unwrap();

    match blob_store.delete(&blob_ref) {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::from(ErrorResponse::from(e)),
    }
}

#[post("/blobs")]
async fn upload_blobs(mut payload: Multipart, data: web::Data<Settings>) -> impl Responder {
    // TODO: handle errors
    let mut blobs: Vec<BlobRef> = Vec::new();
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();

        let filename = content_type.get_filename().unwrap_or("file");
        let filename = sanitize_filename::sanitize(filename);

        let tmp_dir = data.server.tmp_directory.clone();
        let mut tmp_file = web::block(|| NamedTempFile::new_in(tmp_dir)).await.unwrap();

        let mut hasher = BlobStore::hasher();
        while let Some(Ok(chunk)) = field.next().await {
            if content_type.get_name().unwrap() == "file" {
                hasher.update(&chunk);
                tmp_file = web::block(move || tmp_file.write_all(&chunk).map(|_| tmp_file))
                    .await
                    .unwrap();
            }
        }
        let blob_ref = BlobRef::from(hasher);

        let save_path = std::path::PathBuf::from(&data.data_store_dir).join(blob_ref.to_path());
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
pub async fn start_server(settings: Settings) -> std::io::Result<()> {
    std::env::set_var(
        "RUST_LOG",
        format!(
            "{},actix_web={}",
            settings.server.log_level, settings.server.log_level
        ),
    );
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let addr = format!("127.0.0.1:{}", &settings.server.port.to_string());

    if let Err(e) = settings.server.create_dirs() {
        return Err(e);
    }

    HttpServer::new(move || {
        let settings = settings.clone();
        App::new()
            .data(settings.clone())
            .wrap_fn( move |req, srv| {
                let auth_token = req.headers().get("X-Auth-Token");
                match auth_token {
                    Some(auth_token) if validate_token(auth_token.to_str().unwrap(), &settings.server.token_store_path) => {
                        Either::Left(srv.call(req))
                    }
                    _ => Either::Right(ok(req.into_response(HttpResponse::from(
                        ErrorResponse::new("InvalidToken","Unauthorized: the provided authentication token does not match our records.", 401),
                    )))),
                }
            })

            .configure(init_routes)
            .wrap(Logger::new("%r %s %b bytes %D msecs"))
    })
    .bind(addr)?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http, test, web, App};

    #[actix_rt::test]
    async fn test_app_status() {
        let mut app = test::init_service(App::new().configure(init_routes)).await;
        let req = test::TestRequest::get().uri("/status").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success())
    }

    #[actix_rt::test]
    async fn test_get_blob() {
        let settings = Settings {
            data_store_dir: "../tests/test_data_store".into(),
            ..Settings::default()
        };
        let mut app = test::init_service(App::new().data(settings).configure(init_routes)).await;

        // Test getting the blob and its metadata
        let url = "/blobs/f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de";
        let req = test::TestRequest::get().uri(url).to_request();
        let mut resp = test::call_service(&mut app, req).await;

        assert_eq!(resp.status(), http::StatusCode::OK);

        assert_eq!(
            resp.headers().get(http::header::CONTENT_TYPE).unwrap(),
            http::HeaderValue::from_static("text/plain")
        );
        assert_eq!(
            resp.headers()
                .get(http::header::CONTENT_DISPOSITION)
                .unwrap(),
            http::HeaderValue::from_static("attachment")
        );
        assert_eq!(
            resp.headers().get("filename").unwrap(),
            http::HeaderValue::from_static("test_file.txt")
        );

        let (result, _) = resp.take_body().into_future().await;
        assert_eq!(
            result.unwrap().unwrap(),
            web::Bytes::from_static(b"This is a test file.")
        )
    }

    #[actix_rt::test]
    async fn test_get_errors() {
        let settings = Settings {
            data_store_dir: "../tests/test_data_store".into(),
            ..Settings::default()
        };
        let mut app = test::init_service(App::new().data(settings).configure(init_routes)).await;

        let missing_ref_url =
            "/blobs/f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0dx";
        let req = test::TestRequest::get().uri(missing_ref_url).to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);

        let invalid_ref_url = "/blobs/invalid.url";
        let req = test::TestRequest::get().uri(invalid_ref_url).to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    }

    // TODO: how can we test multipart blob upload?
    // TODO: test authentication
}
