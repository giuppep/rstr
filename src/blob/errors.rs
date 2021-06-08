use std::{fmt, io};

/// Error raised by the blob store
#[derive(Debug)]
pub enum BlobError {
    Io(io::Error),
    Blob(BlobErrorKind),
}

/// Error kind raised by the blob store
#[derive(Debug)]
pub enum BlobErrorKind {
    /// Occurs when trying to instantiate a `BlobRef` with an invalid string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rustore::blob::{BlobRef, BlobErrorKind};
    /// let err = BlobRef::new("invalid").unwrap_err();
    /// assert_eq!(err, BlobErrorKind::InvalidRef)
    /// ```
    InvalidRef,
}

impl BlobErrorKind {
    fn as_str(&self) -> &str {
        match *self {
            BlobErrorKind::InvalidRef => {
                "Invalid refererence. Reference must have 64 alphanumerical characters."
            }
        }
    }
}

impl fmt::Display for BlobError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BlobError::Io(ref err) => err.fmt(f),
            BlobError::Blob(ref err) => write!(f, "Error: {}", err.as_str()),
        }
    }
}

impl From<io::Error> for BlobError {
    fn from(err: io::Error) -> BlobError {
        BlobError::Io(err)
    }
}

/// Shorthand for [`Result`] type
///
/// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
pub type Result<T> = std::result::Result<T, BlobError>;
