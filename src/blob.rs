use sha2::{Digest, Sha256};
use std::{fs, io, path::Path, path::PathBuf};

const DATA_LOC: &str = "/tmp/rustore/";

#[derive(Debug)]
pub struct Blob {
    pub filename: String,
    pub content: Vec<u8>,
    reference: BlobRef,
}

#[derive(Debug)]
pub struct BlobRef {
    pub hash: String,
    algorithm: String, // TODO: make enum
}

impl BlobRef {
    pub fn new(hash: &str) -> BlobRef {
        BlobRef {
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

    pub fn compute(content: &[u8]) -> BlobRef {
        let hash = format!("{:x}", Sha256::digest(content));
        BlobRef {
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
        let hash = BlobRef::compute(&content);
        Blob {
            filename,
            content,
            reference: hash,
        }
    }

    pub fn from_content(content: Vec<u8>, filename: &str) -> Blob {
        let hash = BlobRef::compute(&content);
        Blob {
            filename: String::from(filename),
            reference: hash,
            content,
        }
    }

    pub fn from_hash(hash: BlobRef) -> Result<Blob, io::Error> {
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
                reference: hash,
            };
            return Ok(blob);
        }
        Err(io::Error::new(io::ErrorKind::NotFound, "File not found"))
    }

    pub fn save(&self) -> Result<PathBuf, io::Error> {
        let mut path = self.reference.to_path();
        fs::create_dir_all(&path)?;
        path = path.join(&self.filename);
        fs::write(&path, &self.content)?;
        Ok(path)
    }

    pub fn get_ref(&self) -> &str {
        &self.reference.hash[..]
    }

    pub fn get_mime(&self) -> &str {
        match infer::get(&self.content) {
            Some(mime) => mime.mime_type(),
            None => "application/octet-stream",
        }
    }
}

impl std::fmt::Display for Blob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Blob({}, {})", self.filename, &self.get_ref()[..10])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashing() {
        let blob = Blob::from_path(Path::new("test/test_file.txt"));
        assert_eq!(
            blob.get_ref(),
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
