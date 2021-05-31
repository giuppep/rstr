use std::{fmt, io};

#[derive(Debug)]
pub enum BlobError {
    IO(io::Error),
    Regular(BlobErrorKind),
}
#[derive(Debug)]
pub enum BlobErrorKind {
    InvalidRefLength,
    NotFound,
}

impl BlobErrorKind {
    fn as_str(&self) -> &str {
        match *self {
            BlobErrorKind::NotFound => "File not found",
            BlobErrorKind::InvalidRefLength => {
                "Invalid refererence length. Reference must have 64 characters."
            }
        }
    }
}

impl fmt::Display for BlobError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BlobError::IO(ref err) => err.fmt(f),
            BlobError::Regular(ref err) => write!(f, "Error: {}", err.as_str()),
        }
    }
}

impl From<io::Error> for BlobError {
    fn from(err: io::Error) -> BlobError {
        BlobError::IO(err)
    }
}
