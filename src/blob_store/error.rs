use std::{error, fmt, io};

/// Error raised by the blob store
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    /// Occurs when trying to instantiate a `BlobRef` with an invalid string.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustore::BlobRef;
    /// let err = BlobRef::new("invalid").unwrap_err();
    /// # // io::Error does not implement PartialEq
    /// // err == Error::InvalidRef
    /// assert_eq!(format!("{}", err), "Error: Invalid reference. Reference must have 64 alphanumerical characters.");
    /// ```
    InvalidRef,

    /// Occurs when trying to perfom some action on a blob that is not present in the
    /// blob store.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustore::{BlobRef, BlobStore, Error};
    ///
    /// let blob_store = BlobStore::new("tests/test_data_store").unwrap();
    /// let blob_ref = BlobRef::new("b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9").unwrap();
    ///
    /// assert!(!blob_store.exists(&blob_ref));
    /// let err = blob_store.get(&blob_ref).unwrap_err();
    /// # // io::Error does not implement PartialEq
    /// // err == Error::BlobNotFound
    /// assert_eq!(format!("{}", err), "Error: The requested blob was not found in the blob store.");
    ///
    /// let err = blob_store.delete(&blob_ref).unwrap_err();
    /// assert_eq!(format!("{}", err), "Error: The requested blob was not found in the blob store.");
    ///
    /// let err = blob_store.metadata(&blob_ref).unwrap_err();
    /// assert_eq!(format!("{}", err), "Error: The requested blob was not found in the blob store.");
    /// ```
    BlobNotFound,
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::Io(ref err) => Some(err),
            Error::BlobNotFound | Error::InvalidRef => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => err.fmt(f),
            Error::InvalidRef => write!(
                f,
                "Error: Invalid reference. Reference must have 64 alphanumerical characters."
            ),
            Error::BlobNotFound => write!(
                f,
                "Error: The requested blob was not found in the blob store."
            ),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        match err.kind() {
            // TODO: make sure this doesn't swallow some other NotFound error.
            io::ErrorKind::NotFound => Error::BlobNotFound,
            _ => Error::Io(err),
        }
    }
}

/// Shorthand for [`Result`] type
///
/// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
pub type Result<T> = std::result::Result<T, Error>;
