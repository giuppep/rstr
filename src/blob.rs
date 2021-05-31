use crate::error::{BlobError, BlobErrorKind, Result};
use chrono::{offset::Utc, DateTime};
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
    value: String,
}

#[derive(Debug)]
pub struct BlobMetadata {
    pub filename: String,
    pub mime_type: String,
    pub size: u64,
    pub created: DateTime<Utc>,
}

impl BlobRef {
    pub fn new(value: &str) -> Result<BlobRef> {
        match value.len() == 64 {
            true => Ok(BlobRef {
                value: String::from(value),
            }),
            false => Err(BlobError::Blob(BlobErrorKind::InvalidRefLength)),
        }
    }

    pub fn from_hasher(hasher: Sha256) -> BlobRef {
        BlobRef::new(&format!("{:x}", hasher.finalize())[..]).unwrap()
    }
    pub fn from_path(path: &Path) -> Result<BlobRef> {
        let mut file = File::open(path)?;
        let mut hasher = BlobRef::hasher();

        io::copy(&mut file, &mut hasher)?;
        Ok(BlobRef::from_hasher(hasher))
    }

    pub fn hasher() -> Sha256 {
        Sha256::new()
    }

    pub fn to_path(&self) -> PathBuf {
        let base_path =
            env::var("RUSTORE_DATA_PATH").unwrap_or_else(|_| String::from(RUSTORE_DATA_PATH));
        let path = Path::new(&base_path)
            .join(&self.value[0..2])
            .join(&self.value[2..4])
            .join(&self.value[4..6])
            .join(&self.value[6..]);

        path
    }

    pub fn exists(&self) -> bool {
        let dir = self.to_path();
        dir.exists() && dir.read_dir().unwrap().next().is_some()
    }

    pub fn delete(&self) -> io::Result<()> {
        fs::remove_dir_all(self.to_path())
    }

    fn file_path(&self) -> Result<PathBuf> {
        // Get the full path to the file, including the filename
        let mut entries = self.to_path().read_dir().map_err(BlobError::IO)?;
        if let Some(Ok(entry)) = entries.next() {
            return Ok(entry.path());
        };
        Err(BlobError::Blob(BlobErrorKind::NotFound))
    }
    pub fn mime(&self) -> Result<&str> {
        match infer::get_from_path(self.file_path()?).map_err(BlobError::IO)? {
            Some(mime) => Ok(mime.mime_type()),
            _ => Ok("application/octet-stream"),
        }
    }
    pub fn content(&self) -> Result<Vec<u8>> {
        fs::read(&self.file_path()?).map_err(BlobError::IO)
    }

    pub fn metadata(&self) -> Result<BlobMetadata> {
        let file_path = self.file_path()?;
        let filename = file_path.file_name().unwrap().to_str().unwrap().to_string();

        let metadata = fs::metadata(file_path).unwrap();
        Ok(BlobMetadata {
            mime_type: String::from(self.mime()?),
            filename,
            size: metadata.len(),
            created: metadata.created().unwrap().into(),
        })
    }
    pub fn reference(&self) -> &str {
        &self.value
    }
}

impl std::fmt::Display for BlobRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlobRef({})", &self.value[..10])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const TEST_DATA_PATH: &str = "test/";
    const TEST_FILE: &str = "test/test_file.txt";
    const TEST_FILE_HASH: &str = "f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de";
    const TEST_FILE_PATH: &str =
        "f2/9b/c6/4a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de";

    #[test]
    fn test_hashing() {
        let path = Path::new(TEST_FILE);
        let blob_ref = BlobRef::from_path(&path).unwrap();
        assert_eq!(blob_ref.reference(), TEST_FILE_HASH)
    }

    #[test]
    fn test_create_blob_ref() {
        let valid_hash = TEST_FILE_HASH;
        let invalid_hash = "this_is_too_short";

        assert!(BlobRef::new(valid_hash).is_ok());
        assert!(BlobRef::new(invalid_hash).is_err())
    }

    #[test]
    fn test_get_dir() {
        env::set_var("RUSTORE_DATA_PATH", TEST_DATA_PATH);

        let hash = TEST_FILE_HASH;
        let blob_ref = BlobRef::new(hash).unwrap();
        let dir = blob_ref.to_path();

        assert_eq!(
            dir.to_str().unwrap(),
            format!("{}{}", TEST_DATA_PATH, TEST_FILE_PATH)
        )
    }
}
