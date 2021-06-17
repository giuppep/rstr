use crate::blob::{BlobRef, Error, Result};
use ignore::{WalkBuilder, WalkState};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc;
use std::{fs, io, thread};

/// Function to add a file from disk to the blob store
fn add_file(path: &Path) -> Result<BlobRef> {
    if !path.is_file() {
        return Err(Error::Io(io::Error::from(io::ErrorKind::InvalidInput)));
    }

    let blob_ref = BlobRef::from_path(path)?;
    if !blob_ref.exists() {
        let save_path = &blob_ref.to_path();
        fs::create_dir_all(save_path)?;
        let filename = path.file_name().unwrap();
        fs::copy(path, save_path.join(&filename))?;
    }

    Ok(blob_ref)
}

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

/// Given a list of paths to files/directories it adds them to the blob store. In the case
/// of a directory it adds all the files in its children recursively.
///
/// The function iterates over all paths in parallel and adds each file to the blob store.
///
/// If `verbose` is set to `true` it will print to stdout the reference for the file and
/// its original path, e.g.:
/// ```text
/// f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de        test/test_file.txt
/// ```
/// At the end of the process it will print to `stderr` all the errors.
///
/// # Examples
///
/// ```no_run
/// # use std::path::PathBuf;
/// # use rustore::blob::add_files;
/// let paths = [PathBuf::from("/path/to/my/files/")];
/// let threads: u8 = 8;
/// let blob_refs = add_files(&paths[..], threads, false);
/// ```
pub fn add_files(paths: &[PathBuf], threads: u8, verbose: bool) -> Vec<BlobRef> {
    let paths: Vec<PathBuf> = paths.iter().flat_map(|p| collect_file_paths(p)).collect();

    let (tx, rx) = mpsc::channel();

    let chunk_size = (paths.len()) / threads as usize;
    let chunks = paths.chunks(chunk_size);

    for chunk in chunks {
        let tx = tx.clone();
        let chunk = chunk.to_owned();
        thread::spawn(move || {
            for path in chunk {
                let blob_ref = add_file(&path);
                tx.send((path, blob_ref)).expect("err")
            }
        });
    }

    drop(tx);

    let pb = ProgressBar::new(paths.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})\n{msg}"),
    );

    let mut blob_refs = vec![];
    for (path, blob_ref) in rx.iter() {
        if let Ok(blob_ref) = blob_ref {
            if verbose {
                pb.println(format!(
                    "{}\t\t{}",
                    &blob_ref.reference(),
                    path.to_string_lossy()
                ));
            }
            blob_refs.push(blob_ref)
        } else {
            eprintln!("ERROR\t\t{}", path.to_string_lossy())
        }
        pb.inc(1);
    }
    pb.finish_with_message(format!("Successfully added {} blobs!", blob_refs.len()));
    blob_refs
}
