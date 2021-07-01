use serde::Serialize;

/// Struct representing the json payload returned to the user upon error
#[derive(Serialize)]
pub struct ErrorResponse {
    /// The name of the error, can be used to match to error classes in client.
    error: String,
    /// A message providing more detail on the error.
    message: String,
}

impl ErrorResponse {
    pub fn new(error: &str, message: &str) -> Self {
        ErrorResponse {
            error: String::from(error),
            message: String::from(message),
        }
    }
}
