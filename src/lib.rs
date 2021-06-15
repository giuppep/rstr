#![deny(clippy::all, clippy::cargo)]
#![warn(clippy::pedantic, clippy::missing_panics_doc)]
#![allow(clippy::multiple_crate_versions, clippy::must_use_candidate)]
//! `rustore` is a library for managing a content-addressable blob store.
//!
//! An entry in the blob store is represented by an instance of the struct [`BlobRef`].
//!
//! [`BlobRef`]: blob::BlobRef.
pub mod blob;
