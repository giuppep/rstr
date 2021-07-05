use std::fs::{File, OpenOptions};
use std::io::{prelude::*, BufReader};
use std::path::Path;
use uuid::Uuid;

/// Generates a new API token and appends it to the list of valid tokens.
pub fn generate_token(token_store_path: &Path) -> std::io::Result<String> {
    let token = (*Uuid::new_v4()
        .to_simple()
        .encode_upper(&mut Uuid::encode_buffer()))
    .to_string();
    save_token(&token, token_store_path)?;
    Ok(token)
}

/// Appends a new token to the file containing the tokens.
fn save_token(token: &str, token_store_path: &Path) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(token_store_path)?;
    writeln!(&mut file, "{}", token).unwrap();
    Ok(())
}

/// Validates a token against a file containing a list of valid tokens.
pub fn validate_token(token: &str, token_store_path: &Path) -> bool {
    let file = match File::open(token_store_path) {
        Ok(file) => file,
        Err(_) => return false,
    };

    let reader = BufReader::new(file);
    for line in reader.lines() {
        if token == line.unwrap() {
            return true;
        }
    }
    false
}
