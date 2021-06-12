use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::path::Path;
use uuid::Uuid;

const TOKEN_STORE_PATH: &str = "/tmp/rustore/.tokens";

/// Generates a new API token and appends it to the list of valid tokens.
pub fn generate_token() -> String {
    let token = Uuid::new_v4()
        .to_simple()
        .encode_upper(&mut Uuid::encode_buffer())
        .to_string();
    save_token(&token, TOKEN_STORE_PATH);
    token
}

/// Appends a new token to the file containing the tokens.
fn save_token<P: AsRef<Path>>(token: &str, token_store_path: P) {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(token_store_path)
        .expect("Can't open file.");
    writeln!(&mut file, "{}", token).unwrap();
}

/// Validates a token against a file containing a list of valid tokens.
pub fn validate_token(token: &str) -> bool {
    let file = match File::open(TOKEN_STORE_PATH) {
        Ok(file) => file,
        Err(_) => return false,
    };

    let reader = std::io::BufReader::new(file);
    for line in reader.lines() {
        if token == line.unwrap() {
            return true;
        }
    }
    false
}
