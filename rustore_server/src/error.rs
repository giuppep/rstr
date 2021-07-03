use actix_web::HttpResponse;
use rustore::Error;
use serde::Serialize;
/// Struct representing the json payload returned to the user upon error
#[derive(Serialize)]
pub struct ErrorResponse {
    /// The name of the error, can be used to match to error classes in client.
    error: String,
    /// A message providing more detail on the error.
    message: String,
    /// The HTTP status code of the error. Used for converting to a response.
    #[serde(skip_serializing)]
    status_code: u16,
}

impl ErrorResponse {
    pub fn new(error: &str, message: &str, code: u16) -> Self {
        ErrorResponse {
            error: error.into(),
            message: message.into(),
            status_code: code,
        }
    }
}

impl From<Error> for ErrorResponse {
    fn from(err: Error) -> ErrorResponse {
        match err {
            Error::BlobNotFound => ErrorResponse::new("BlobNotFound", &err.to_string(), 404),
            Error::InvalidRef => ErrorResponse::new("InvalidReference", &err.to_string(), 400),
            Error::Io(_) => ErrorResponse::new("IO", &err.to_string(), 500),
        }
    }
}

impl From<ErrorResponse> for HttpResponse {
    fn from(err: ErrorResponse) -> Self {
        match err.status_code {
            404 => HttpResponse::NotFound().json(err),
            400 => HttpResponse::BadRequest().json(err),
            401 => HttpResponse::Unauthorized().json(err),
            _ => HttpResponse::InternalServerError().json(err),
        }
    }
}
