use super::error::{Error, Result};
use chrono::{offset::Utc, DateTime};
use lazy_static::lazy_static;
use regex::Regex;
use sha2::{Digest, Sha256};
use std::{
    env,
    fs::{self, File},
    io,
    path::Path,
    path::PathBuf,
};
use tree_magic_mini as magic;

/// Struct representing a reference to an entry in the blob store
#[derive(Debug, Clone)]
pub struct BlobRef {
    /// The value of the reference, i.e. the sha256 hash of the blob
    value: String,
}

/// Struct representing the metadata associated to a blob
#[derive(Debug)]
pub struct BlobMetadata {
    /// The filename of the blob
    pub filename: String,
    /// The mime-type of the blob (e.g. `image/png`)
    pub mime_type: String,
    /// The size of the blob in bytes
    pub size: u64,
    /// The creation timestamp of the blob
    pub created: DateTime<Utc>,
}

/// Returns a `BlobRef` instance from a hasher
///
/// # Examples
///
/// ```
/// # use sha2::{Digest, Sha256};
/// # use rustore::BlobRef;
/// let mut hasher = Sha256::new();
/// hasher.update(b"hello world");
/// let blob_ref = BlobRef::from(hasher);
/// assert_eq!(blob_ref.reference(), "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
/// ```
impl From<Sha256> for BlobRef {
    fn from(hasher: Sha256) -> Self {
        BlobRef::new(&format!("{:x}", hasher.finalize())[..]).unwrap()
    }
}

impl BlobRef {
    /// Creates a new `BlobRef` from a valid hex representation of the sha256 hash.
    ///
    /// # Errors
    ///
    /// The method will return a [`Error::InvalidRef`] if the input string
    /// - has `len() != 64`
    /// - contains any char except lowercase letters and digits
    /// # Examples
    /// ```
    /// # use rustore::BlobRef;
    /// let blob_ref = BlobRef::new("f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de");
    /// assert!(blob_ref.is_ok())
    /// ```
    /// ```
    /// # use rustore::BlobRef;
    /// let blob_ref = BlobRef::new("a_short_hash");
    /// assert!(blob_ref.is_err());
    /// let blob_ref = BlobRef::new("....aninvalidhash.29bc64a9d3732b4b9035125fdb3285f5b6455778edca7");
    /// assert!(blob_ref.is_err());
    /// ```

    pub fn new(value: &str) -> Result<BlobRef> {
        lazy_static! {
            static ref VALID_HASH_REGEX: Regex = Regex::new(r"^[a-z0-9]{64}$").unwrap();
        }

        if VALID_HASH_REGEX.is_match(value) {
            Ok(BlobRef {
                value: String::from(value),
            })
        } else {
            Err(Error::InvalidRef)
        }
    }

    /// Creates a `BlobRef` from a document on disk.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::Path;
    /// # use rustore::BlobRef;
    /// let path = Path::new("tests/test_file.txt");
    /// let blob_ref = BlobRef::from_path(&path);
    /// assert!(blob_ref.is_ok());
    /// assert_eq!(blob_ref.unwrap().reference(), "f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de")
    /// ```
    /// Note that this *does not* add the file to the blob store, the user will have to
    /// manually save it to `blob_ref.to_path()`.
    ///
    /// # Errors
    ///
    /// See [`std::fs::File::open`] and [`io::copy`]
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<BlobRef> {
        let mut file = File::open(path)?;
        let mut hasher = BlobRef::hasher();

        io::copy(&mut file, &mut hasher)?;
        Ok(BlobRef::from(hasher))
    }

    /// Returns an instance of the hasher used to compute the blob reference for a file
    ///
    /// # Examples
    ///
    /// ```
    /// # use rustore::BlobRef;
    /// # use sha2::{Digest, Sha256};
    /// let mut hasher = BlobRef::hasher();
    /// hasher.update(b"hello world");
    /// let result = hasher.finalize();
    /// assert_eq!(format!("{:x}", result), "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9")
    /// ```
    pub fn hasher() -> Sha256 {
        Sha256::new()
    }

    /// Converts the blob's reference into a path.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rustore::BlobRef;
    /// std::env::set_var("RUSTORE_DATA_PATH", "/tmp/rustore");
    /// let hash = "f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de";
    /// let blob_ref = BlobRef::new(hash).unwrap();
    /// assert_eq!(blob_ref.to_path().to_str().unwrap(), "/tmp/rustore/f2/9b/c6/4a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de")
    /// ```
    ///
    /// # Panics
    ///
    /// This function assumes that the `RUSTORE_DATA_PATH` environment variable has been
    /// set to a valid path and panics otherwise.
    pub fn to_path(&self) -> PathBuf {
        let base_path: PathBuf = env::var("RUSTORE_DATA_PATH").unwrap().into();
        base_path
            .join(&self.value[0..2])
            .join(&self.value[2..4])
            .join(&self.value[4..6])
            .join(&self.value[6..])
    }

    /// Returns `true` if there is a file associated with the reference is in the blob store
    pub fn exists(&self) -> bool {
        let dir = self.to_path();
        dir.exists() && dir.read_dir().unwrap().next().is_some()
    }

    /// Deletes the file referenced by the `BlobRef`.
    ///
    /// # Errors
    ///
    /// See [`fs::remove_dir_all`].
    pub fn delete(&self) -> Result<()> {
        Ok(fs::remove_dir_all(self.to_path())?)
    }

    /// Get the full path to the file, including the filename
    ///
    /// # Errors
    ///
    /// Will return an error if
    /// - the directory cannot be read
    /// - the directory is empty
    fn file_path(&self) -> Result<PathBuf> {
        let mut entries = self.to_path().read_dir()?;
        if let Some(Ok(entry)) = entries.next() {
            return Ok(entry.path());
        };
        Err(Error::Io(io::Error::from(io::ErrorKind::NotFound)))
    }

    /// Returns the mime type inferred from the file's magic number as a string.
    /// It defaults to "application/octet-stream" if it cannot determine the type.
    /// We use the [`tree_magic_mini`] crate to infer the mime type.
    ///
    /// # Errors
    ///
    /// It will return an error if the file cannot be read or the mime type cannot be inferred.
    pub fn mime(&self) -> Result<&str> {
        match magic::from_filepath(&self.file_path()?) {
            Some(mime) => Ok(mime),
            None => Ok("application/octet-stream"),
        }
    }

    /// Get the content of the referenced file as a byte array.
    ///
    /// # Errors
    ///
    /// It wil return an error if the file is not present or cannot be read.
    pub fn content(&self) -> Result<Vec<u8>> {
        Ok(fs::read(&self.file_path()?)?)
    }

    /// Returns some metadata for the file referenced to.
    ///
    /// # Errors
    ///
    /// Will return an error if the file cannot be found/opened or if [`std::fs::metadata`]
    /// errors.
    pub fn metadata(&self) -> Result<BlobMetadata> {
        let file_path = self.file_path()?;
        let filename = file_path.file_name().unwrap().to_str().unwrap().to_string();

        let metadata = fs::metadata(file_path)?;
        Ok(BlobMetadata {
            mime_type: String::from(self.mime()?),
            filename,
            size: metadata.len(),
            created: metadata.created()?.into(),
        })
    }

    /// Returns a string reference (hex representation of Sha256 hash) for the blob
    pub fn reference(&self) -> &str {
        &self.value
    }
}

impl std::fmt::Display for BlobRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlobRef({})", &self.value[..10])
    }
}
