#![deny(clippy::all, clippy::cargo)]
#![warn(clippy::pedantic, clippy::missing_panics_doc)]
#![allow(clippy::multiple_crate_versions, clippy::must_use_candidate)]
//! `rustore` is a library for managing a content-addressable blob store.
//!
//! An entry in the blob store is represented by an instance of the struct [`BlobRef`].
//!
//! # Examples
//!
//! Before interacting with the blob store you must specify its path
//!
//! ```no_run
//! std::env::set_var("RUSTORE_DATA_PATH", "path/to/blob/store");
//! ```
//!
//! Add files or directories to the blob store:
//! ```no_run
//! use rustore::{BlobStore,BlobRef};
//! use std::path::{Path, PathBuf};
//!
//! let blob_store = BlobStore::new("tests/test_data_store").unwrap();
//! let n_threads: u8 = 8;
//! let (blob_refs_with_paths, _): (Vec<(PathBuf, BlobRef)>, _) = blob_store.add_files(
//!     &[
//!         Path::new("/path/to/a/file.pdf"),
//!         Path::new("/path/to/a/directory/"),
//!     ],
//!     n_threads,
//! );
//! let blob_refs: Vec<BlobRef> = blob_refs_with_paths.into_iter().map(|(_, b)| b).collect();
//! ```
//!
//! Retrieve a blob from the blob store
//!
//! ```
//! use rustore::{BlobStore, BlobRef};
//!
//! let blob_store = BlobStore::new("tests/test_data_store").unwrap();
//!
//! // Retrieve a blob from the blob store
//! let reference = "f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de";
//! let blob_ref = BlobRef::new(reference).unwrap();
//!
//! assert!(blob_store.exists(&blob_ref));
//!
//! // Get the blob's content
//! let content = blob_store.get(&blob_ref).unwrap();
//!
//! // Get the blob's metadata
//! let metadata = blob_store.metadata(&blob_ref).unwrap();
//! assert_eq!(metadata.filename, "test_file.txt");
//! assert_eq!(metadata.mime_type, "text/plain");
//! ```

mod blob;
pub use blob::{BlobMetadata, BlobRef, BlobStore, Error, Result};
