use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::path::Path;
use uuid::Uuid;

pub fn generate_token() -> String {
    Uuid::new_v4()
        .to_simple()
        .encode_upper(&mut Uuid::encode_buffer())
        .to_string()
}

pub fn save_token<P: AsRef<Path>>(token: &str, token_store_path: P) {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(token_store_path)
        .expect("Can't open file.");
    writeln!(&mut file, "{}", token).unwrap();
}

pub fn validate_token(token: &str) -> bool {
    // TODO: handle errors
    let file = File::open("/tmp/rustore/.tokens").unwrap();
    let reader = std::io::BufReader::new(file);
    for line in reader.lines() {
        if token == line.unwrap() {
            return true;
        }
    }
    false
}
