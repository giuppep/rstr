# rstr
`rstr` is a library for managing a content-addressable blob store.

# Examples
Add files or directories to the blob store:

```rust
use rstr::{BlobStore,BlobRef};
use std::path::{Path, PathBuf};
let blob_store = BlobStore::new("../tests/test_data_store").unwrap();
let n_threads: u8 = 8;
let (blob_refs_with_paths, _): (Vec<(PathBuf, BlobRef)>, _) = blob_store.add_files(
    &[
        // Can add files
        Path::new("../tests/test_file.txt"),
        // or directories
        Path::new("../tests/"),
    ],
    n_threads,
);
let blob_refs: Vec<BlobRef> = blob_refs_with_paths.into_iter().map(|(_, b)| b).collect();
```

Retrieve a blob from the blob store
```rust
use rstr::{BlobStore, BlobRef};

let blob_store = BlobStore::new("../tests/test_data_store").unwrap();

// Retrieve a blob from the blob store
let reference = "f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de";
let blob_ref = BlobRef::new(reference).unwrap();

assert!(blob_store.exists(&blob_ref));

// Get the blob's content
let content = blob_store.get(&blob_ref).unwrap();

// Get the blob's metadata
let metadata = blob_store.metadata(&blob_ref).unwrap();
assert_eq!(metadata.filename, "test_file.txt");
assert_eq!(metadata.mime_type, "text/plain");
```