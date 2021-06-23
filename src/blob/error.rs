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
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::Io(ref err) => Some(err),
            Error::InvalidRef => None,
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
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

/// Shorthand for [`Result`] type
///
/// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
pub type Result<T> = std::result::Result<T, Error>;
