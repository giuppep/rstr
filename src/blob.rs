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
    pub fn new(value: &str) -> Result<BlobRef, &'static str> {
        match value.len() == 64 {
            true => Ok(BlobRef {
                value: String::from(value),
            }),
            false => Err("Invalid length. Reference must have 64 characters."),
        }
    }

    pub fn from_hasher(hasher: Sha256) -> BlobRef {
        BlobRef::new(&format!("{:x}", hasher.finalize())[..]).unwrap()
    }
    pub fn from_path(path: &Path) -> Result<BlobRef, std::io::Error> {
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

    fn file_path(&self) -> Result<PathBuf, &'static str> {
        // Get the full path to the file, including the filename
        match self.to_path().read_dir() {
            Ok(mut entries) => {
                if let Some(Ok(entry)) = entries.next() {
                    return Ok(entry.path());
                };
                Err("Directory is empty")
            }
            Err(_) => Err("Directory not found"),
        }
    }
    pub fn mime(&self) -> Result<&str, &'static str> {
        match infer::get_from_path(self.file_path()?).expect("could not read file") {
            Some(mime) => Ok(mime.mime_type()),
            _ => Ok("application/octet-stream"),
        }
    }
    pub fn content(&self) -> Result<Vec<u8>, &'static str> {
        match fs::read(&self.file_path()?) {
            Ok(f) => Ok(f),
            Err(_) => Err("Cannot open the file"),
        }
    }

    pub fn metadata(&self) -> Result<BlobMetadata, &'static str> {
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

    #[test]
    fn test_hashing() {
        let path = Path::new("test/test_file.txt");
        let blob_ref = BlobRef::from_path(&path).unwrap();
        assert_eq!(
            blob_ref.reference(),
            "f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de"
        )
    }

    #[test]
    fn test_get_dir() {
        let hash = BlobRef::new("f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de")
            .unwrap();
        let dir = hash.to_path();
        assert_eq!(
            dir.to_str().unwrap(),
            "/tmp/rustore/f2/9b/c6/4a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de"
        )
    }
}
