use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    env,
    fs::{self, File},
    io,
    path::Path,
    path::PathBuf,
};

const RUSTORE_DATA_PATH: &str = "/tmp/rustore/";

#[derive(Debug, Clone)]
pub struct BlobRef {
    pub hash: String,
    algorithm: String, // TODO: make enum
}

#[derive(Debug, Serialize)]
pub struct BlobMetadata {
    filename: String,
    mime_type: String,
}

impl BlobRef {
    pub fn new(hash: &str) -> BlobRef {
        BlobRef {
            hash: String::from(hash),
            algorithm: String::from("sha256"),
        }
    }

    pub fn to_path(&self) -> PathBuf {
        let base_path = env::var("RUSTORE_DATA_PATH").unwrap_or(String::from(RUSTORE_DATA_PATH));
        let path = Path::new(&base_path)
            .join(&self.hash[0..2])
            .join(&self.hash[2..4])
            .join(&self.hash[4..6])
            .join(&self.hash[6..]);

        path
    }

    pub fn exists(&self) -> bool {
        let dir = self.to_path();
        if dir.exists() {
            return !dir.read_dir().unwrap().next().is_none();
        }
        false
    }

    fn get_file_path(&self) -> Result<PathBuf, &'static str> {
        // Get the full path to the file, including the filename
        match self.to_path().read_dir() {
            Ok(entries) => {
                for entry in entries {
                    return Ok(entry.unwrap().path());
                }
                return Err("Directory is empty");
            }
            Err(_) => Err("Directory not found"),
        }
    }
    pub fn get_mime(&self) -> Result<&str, &'static str> {
        match infer::get_from_path(self.get_file_path()?).expect("could not read file") {
            Some(mime) => return Ok(mime.mime_type()),
            _ => return Ok("application/octet-stream"),
        }
    }
    pub fn get_content(&self) -> Result<Vec<u8>, &'static str> {
        match fs::read(&self.get_file_path()?) {
            Ok(f) => Ok(f),
            Err(_) => Err("Cannot open the file"),
        }
    }

    pub fn get_metadata(&self) -> Result<BlobMetadata, &'static str> {
        if !self.exists() {
            return Err("File not found");
        }

        let filename = self
            .get_file_path()?
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        Ok(BlobMetadata {
            mime_type: String::from(self.get_mime()?),
            filename,
        })
    }

    pub fn from_path(path: &Path) -> Result<BlobRef, std::io::Error> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();

        io::copy(&mut file, &mut hasher)?;
        Ok(BlobRef::new(&format!("{:x}", hasher.finalize())))
    }
}

impl std::fmt::Display for BlobRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlobRef({}:{})", &self.algorithm, &self.hash[..10])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashing() {
        let path = Path::new("test/test_file.txt");
        let blob_ref = BlobRef::from_path(&path).unwrap();
        assert_eq!(
            blob_ref.hash,
            "f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de"
        )
    }

    #[test]
    fn test_get_dir() {
        let hash = BlobRef::new("f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de");
        let dir = hash.to_path();
        assert_eq!(
            dir.to_str().unwrap(),
            "/tmp/rustore/f2/9b/c6/4a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de"
        )
    }
}
