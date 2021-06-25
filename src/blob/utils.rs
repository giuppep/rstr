use crate::blob::{BlobRef, BlobStore, Error, Result};
use ignore::{WalkBuilder, WalkState};
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

type BlobRefAndPath = (PathBuf, BlobRef);

// /// Function to add a file from disk to the blob store
// fn add_file(path: &Path) -> Result<BlobRef> {
//     if !path.is_file() {
//         return Err(io::Error::from(io::ErrorKind::InvalidInput).into());
//     }

//     let blob_ref = BlobRef::from_path(path)?;
//     if !blob_ref.exists() {
//         let save_path = &blob_ref.to_path();
//         fs::create_dir_all(save_path)?;
//         let filename = path.file_name().unwrap();
//         fs::copy(path, save_path.join(&filename))?;
//     }

//     Ok(blob_ref)
// }

/// Given a path to a directory it recursively walks all its children in parallel
/// and returns a list of paths to files.
fn collect_file_paths(path: &Path) -> Vec<PathBuf> {
    if path.is_file() {
        return vec![path.to_path_buf()];
    }

    let walker = WalkBuilder::new(path);
    let (tx, rx) = mpsc::channel();
    walker.build_parallel().run(|| {
        let tx = tx.clone();
        Box::new(move |entry| match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    tx.send(path.into()).expect("Err")
                }
                WalkState::Continue
            }
            Err(_) => WalkState::Continue,
        })
    });

    drop(tx);
    let mut paths = vec![];
    for path in rx.iter() {
        paths.push(path)
    }
    paths
}

fn progress_bar(length: u64) -> ProgressBar {
    let pb = ProgressBar::new(length);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})\n{msg}"),
    );
    pb
}

/// Given a list of paths to files/directories it adds them to the blob store. In the case
/// of a directory it adds all the files in its children recursively.
///
/// The function iterates over all paths in parallel and adds each file to the blob store.
///
/// It returns two vectors: one containing the paths to the files that were successfully
/// added together with their generated `BlobRef` and the other containing the list of
/// paths that errored together with the error.
///
/// # Examples
///
/// ```no_run
/// # use std::path::Path;
/// # use rustore::{add_files, BlobRef};
/// let paths = [Path::new("/path/to/my/files/")];
/// let threads: u8 = 8;
/// let (blob_refs_with_paths, errors) = add_files(&paths[..], threads);
/// let blob_refs: Vec<BlobRef> = blob_refs_with_paths.into_iter().map(|(_, b)| b).collect();
/// ```
pub fn add_files<P: AsRef<Path>>(
    paths: &[P],
    threads: u8,
) -> (Vec<BlobRefAndPath>, Vec<(PathBuf, Error)>) {
    let paths: Vec<PathBuf> = paths
        .iter()
        .flat_map(|p| collect_file_paths(p.as_ref()))
        .collect();

    let (tx, rx) = mpsc::channel();

    let chunk_size = std::cmp::max(paths.len() / threads as usize, 1_usize);
    let chunks = paths.chunks(chunk_size);

    let blob_store = BlobStore::new(std::env::var("RUSTORE_DATA_PATH").unwrap()).unwrap();

    for chunk in chunks {
        let tx = tx.clone();
        let chunk = chunk.to_owned();
        let blob_store = blob_store.clone();
        thread::spawn(move || {
            for path in chunk {
                let blob_ref = blob_store.add(&path);
                tx.send((path, blob_ref)).expect("err")
            }
        });
    }

    drop(tx);

    let pb = progress_bar(paths.len() as u64);
    let (success, errors): (Vec<_>, Vec<_>) =
        rx.iter().progress_with(pb).partition(|(_, b)| b.is_ok());

    let success = success.into_iter().map(|(p, b)| (p, b.unwrap())).collect();
    let errors = errors
        .into_iter()
        .map(|(p, b)| (p, b.unwrap_err()))
        .collect();
    (success, errors)
}
