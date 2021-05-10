use sha2::{Digest, Sha256};
use std::{fs, io, path::Path, path::PathBuf};

const DATA_LOC: &str = "/tmp/rustore/";

#[derive(Debug)]
pub struct Blob {
    pub filename: String,
    pub hash: String,
    content: Vec<u8>,
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
        let hash = hash_content(&content);
        Blob {
            filename,
            content,
            hash,
        }
    }

    pub fn from_content(content: Vec<u8>, filename: &str) -> Blob {
        let hash = hash_content(&content);
        Blob {
            filename: String::from(filename),
            hash,
            content,
        }
    }

    pub fn from_hash(hash: &str) -> Result<Blob, io::Error> {
        let dir = hash_to_path(hash);
        let entries = fs::read_dir(dir)?;

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
                hash: String::from(hash),
                content,
            };
            return Ok(blob);
        }
        Err(io::Error::new(io::ErrorKind::NotFound, "File not found"))
    }

    fn get_dir(&self) -> PathBuf {
        hash_to_path(&self.hash)
    }

    pub fn save(&self) -> Result<PathBuf, io::Error> {
        let mut path = self.get_dir();
        fs::create_dir_all(&path)?;
        path = path.join(&self.filename);
        fs::write(&path, &self.content)?;
        Ok(path)
    }
}

impl std::fmt::Display for Blob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Blob({}, {})", self.filename, &self.hash[..10])
    }
}

fn hash_content(content: &Vec<u8>) -> String {
    format!("{:x}", Sha256::digest(content))
}

fn hash_to_path(hash: &str) -> PathBuf {
    let path = Path::new(DATA_LOC)
        .join(&hash[0..2])
        .join(&hash[2..4])
        .join(&hash[4..6])
        .join(&hash[6..]);

    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashing() {
        let blob = Blob::from_path(Path::new("test/test_file.txt"));
        assert_eq!(
            blob.hash,
            "f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de"
        )
    }

    #[test]
    fn test_get_dir() {
        let hash = "f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de";
        let dir = hash_to_path(hash);
        assert_eq!(
            dir.to_str().unwrap(),
            "/tmp/rustore/f2/9b/c6/4a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de"
        )
    }
}
