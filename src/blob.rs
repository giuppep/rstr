use sha2::{Digest, Sha256};
use std::{fs, io, path::Path, path::PathBuf};

const DATA_LOC: &str = "/tmp/rustore/";

#[derive(Debug)]
pub struct Blob {
    pub filename: String,
    pub content: Vec<u8>,
    hash: BlobHash,
}

#[derive(Debug)]
pub struct BlobHash {
    pub hash: String,
    algorithm: String, // TODO: make enum
}

impl BlobHash {
    pub fn new(hash: &str) -> BlobHash {
        BlobHash {
            hash: String::from(hash),
            algorithm: String::from("sha256"),
        }
    }

    pub fn to_path(&self) -> PathBuf {
        let path = Path::new(DATA_LOC)
            .join(&self.hash[0..2])
            .join(&self.hash[2..4])
            .join(&self.hash[4..6])
            .join(&self.hash[6..]);

        path
    }

    pub fn compute(content: &[u8]) -> BlobHash {
        let hash = format!("{:x}", Sha256::digest(content));
        BlobHash {
            hash,
            algorithm: String::from("sha256"),
        }
    }
}

impl Blob {
    pub fn from_path(path: &Path) -> Blob {
        let filename = path
            .file_name()
            .expect("Something went wrong extracting file name")
            .to_str()
            .expect("Could not convert filename to string")
            .to_string();
        let content = fs::read(path).unwrap();
        let hash = BlobHash::compute(&content);
        Blob {
            filename,
            content,
            hash,
        }
    }

    pub fn from_content(content: Vec<u8>, filename: &str) -> Blob {
        let hash = BlobHash::compute(&content);
        Blob {
            filename: String::from(filename),
            hash,
            content,
        }
    }

    pub fn from_hash(hash: BlobHash) -> Result<Blob, io::Error> {
        let entries = fs::read_dir(hash.to_path())?;

        for entry in entries {
            let path = entry.unwrap().path();
            let content = fs::read(&path)?;
            let filename = path
                .file_name()
                .expect("Something went wrong extracting file name")
                .to_str()
                .unwrap();
            let blob = Blob {
                filename: String::from(filename),
                content,
                hash,
            };
            return Ok(blob);
        }
        Err(io::Error::new(io::ErrorKind::NotFound, "File not found"))
    }

    pub fn save(&self) -> Result<PathBuf, io::Error> {
        let mut path = self.hash.to_path();
        fs::create_dir_all(&path)?;
        path = path.join(&self.filename);
        fs::write(&path, &self.content)?;
        Ok(path)
    }

    pub fn get_hash(&self) -> &str {
        &self.hash.hash[..]
    }
}

impl std::fmt::Display for Blob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Blob({}, {})", self.filename, &self.get_hash()[..10])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashing() {
        let blob = Blob::from_path(Path::new("test/test_file.txt"));
        assert_eq!(
            blob.get_hash(),
            "f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de"
        )
    }

    #[test]
    fn test_get_dir() {
        let hash =
            BlobHash::new("f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de");
        let dir = hash.to_path();
        assert_eq!(
            dir.to_str().unwrap(),
            "/tmp/rustore/f2/9b/c6/4a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de"
        )
    }
}
