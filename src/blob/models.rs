use super::error::{Error, Result};
use chrono::{offset::Utc, DateTime};
use lazy_static::lazy_static;
use regex::Regex;
use sha2::{Digest, Sha256};
use std::{
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

#[derive(Clone, Debug)]
pub struct BlobStore {
    root: PathBuf,
}

impl BlobStore {
    /// Creates a new instance of the `BlobStore` struct used to interact with the blob
    /// store. If the specified blob store root path does not exists, it tries to create
    /// it.
    ///
    /// # Errors
    ///
    /// It errors if the specified path is not a directory or if it does not exist and
    /// cannot be created.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustore::BlobStore;
    ///
    /// let blob_store = BlobStore::new("tests/test_data_store");
    /// assert!(blob_store.is_ok());
    ///
    /// let blob_store = BlobStore::new("tests/test_file.txt");
    /// assert!(blob_store.is_err());
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> Result<BlobStore> {
        let path = path.as_ref();
        if !path.exists() {
            fs::create_dir_all(path)?
        } else if !path.is_dir() {
            // TODO: return proper error
            return Err(io::Error::from(io::ErrorKind::Other).into());
        }
        Ok(BlobStore { root: path.into() })
    }

    /// Returns an instance of the hasher used to compute the blob reference for a file
    ///
    /// # Examples
    ///
    /// ```
    /// # use rustore::BlobStore;
    /// # use sha2::{Digest, Sha256};
    /// let mut hasher = BlobStore::hasher();
    /// hasher.update(b"hello world");
    /// let result = hasher.finalize();
    /// assert_eq!(format!("{:x}", result), "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9")
    /// ```
    pub fn hasher() -> Sha256 {
        Sha256::new()
    }

    /// Given a `BlobRef` it returns it's path inside the blob store
    fn get_blob_path(&self, blob_ref: &BlobRef) -> PathBuf {
        self.root.join(blob_ref.to_path())
    }

    /// Given a `BlobRef` it returns it's path inside the blob store, including the filename
    ///
    /// # Errors
    ///
    /// It will error if the directory is not present/cannot be read or there is no file.
    fn get_blob_file_path(&self, blob_ref: &BlobRef) -> Result<PathBuf> {
        let mut entries = self.get_blob_path(blob_ref).read_dir()?;
        if let Some(Ok(entry)) = entries.next() {
            return Ok(entry.path());
        };
        Err(Error::BlobNotFound)
    }

    /// Add a file to the blob store given a path.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustore::{BlobStore, BlobRef};
    /// use std::path::PathBuf;
    /// let blob_store = BlobStore::new("tests/test_data_store/").unwrap();
    ///
    /// let blob_ref: BlobRef = blob_store.add("tests/test_file.txt").unwrap();
    /// assert!(blob_store.exists(&blob_ref));
    /// assert_eq!(blob_ref.reference(), "f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de");
    /// ```
    pub fn add<P: AsRef<Path>>(&self, path: P) -> Result<BlobRef> {
        let mut file = File::open(&path)?;
        let mut hasher = BlobStore::hasher();

        io::copy(&mut file, &mut hasher)?;
        let blob_ref = BlobRef::from(hasher);

        if !self.exists(&blob_ref) {
            let save_path = self.get_blob_path(&blob_ref);
            fs::create_dir_all(&save_path)?;

            let filename = path.as_ref().file_name().unwrap();
            let save_path = save_path.join(&filename);
            fs::copy(path, save_path)?;
        };

        Ok(blob_ref)
    }

    pub fn get(&self, blob_ref: &BlobRef) -> Result<Vec<u8>> {
        Ok(fs::read(&self.get_blob_file_path(blob_ref)?)?)
    }

    /// Returns `true` if there is a file associated with the `BlobRef` in the blob store
    ///
    /// # Examples
    ///
    /// ```
    /// use rustore::{BlobStore, BlobRef};
    ///
    /// let blob_store = BlobStore::new("tests/test_data_store/").unwrap();
    /// let blob_ref = BlobRef::new("f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de").unwrap();
    ///
    /// assert!(blob_store.exists(&blob_ref))
    /// ```
    pub fn exists(&self, blob_ref: &BlobRef) -> bool {
        let dir = self.get_blob_path(blob_ref);
        dir.exists() && dir.read_dir().unwrap().next().is_some()
    }

    /// Given a `BlobRef` it deletes the corresponding blob from the blob store
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rustore::{BlobStore, BlobRef};
    /// let blob_store = BlobStore::new("/path/to/blob/store").unwrap();
    ///
    /// let blob_ref = BlobRef::new("b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9").unwrap();
    /// assert!(blob_store.exists(&blob_ref));
    ///
    /// blob_store.delete(&blob_ref);
    /// assert!(!blob_store.exists(&blob_ref));
    /// ```
    /// # Errors
    ///
    /// See [`fs::remove_dir_all`].
    pub fn delete(&self, blob_ref: &BlobRef) -> Result<()> {
        Ok(fs::remove_dir_all(self.get_blob_path(blob_ref))?)
    }

    /// Given a `BlobRef` returns the metadata relative to the referenced blob. For more
    /// details on the metadata returned see `BlobMetadata`.
    ///
    /// The mime type is inferred from the file's magic number as a string.
    /// It defaults to "application/octet-stream" if it cannot determine the type.
    /// We use the [`tree_magic_mini`] crate to infer the mime type.
    ///
    /// # Errors
    ///
    /// Will return an error if the file cannot be found/opened or if [`std::fs::metadata`]
    /// fails.
    pub fn metadata(&self, blob_ref: &BlobRef) -> Result<BlobMetadata> {
        let file_path = self.get_blob_path(blob_ref);
        let filename = file_path.file_name().unwrap().to_str().unwrap().to_string();

        let mime = magic::from_filepath(&self.get_blob_file_path(blob_ref)?)
            .unwrap_or("application/octet-stream");

        let metadata = fs::metadata(file_path)?;
        Ok(BlobMetadata {
            mime_type: String::from(mime),
            filename,
            size: metadata.len(),
            created: metadata.created()?.into(),
        })
    }
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
    ///
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

    /// Converts the blob's reference into a path relative to the root of the blob store.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rustore::BlobRef;
    /// let hash = "f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de";
    /// let blob_ref = BlobRef::new(hash).unwrap();
    /// assert_eq!(blob_ref.to_path().to_str().unwrap(), "f2/9b/c6/4a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de")
    /// ```
    ///
    /// # Panics
    ///
    /// This function assumes that the `RUSTORE_DATA_PATH` environment variable has been
    /// set to a valid path and panics otherwise.
    pub fn to_path(&self) -> PathBuf {
        PathBuf::from(&self.value[0..2])
            .join(&self.value[2..4])
            .join(&self.value[4..6])
            .join(&self.value[6..])
    }

    /// Returns a string reference (hex representation of Sha256 hash) for the blob
    ///
    /// # Examples
    ///
    /// ```
    /// # use rustore::BlobRef;
    /// let hash = "f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de";
    /// let blob_ref = BlobRef::new(hash).unwrap();
    ///
    /// assert_eq!(blob_ref.reference(), hash);
    /// ```
    pub fn reference(&self) -> &str {
        &self.value
    }
}

impl std::fmt::Display for BlobRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlobRef({})", &self.value[..10])
    }
}
