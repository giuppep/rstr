use crate::blob::{BlobError, BlobRef, Result};
use ignore::{WalkBuilder, WalkState};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc;
use std::{fs, io};

/// Function to add a file from disk to the blob store
///
/// If verbose is `true` it prints to stdout the reference for the file and it's original path.
/// ```ignore
/// f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de        test/test_file.txt
/// ```
fn add_file(path: &Path, verbose: bool) -> Result<BlobRef> {
    if !path.is_file() {
        return Err(BlobError::IO(io::Error::from(io::ErrorKind::InvalidInput)));
    }

    let blob_ref = BlobRef::from_path(path)?;
    if !blob_ref.exists() {
        let save_path = &blob_ref.to_path();
        fs::create_dir_all(save_path).map_err(BlobError::IO)?;
        let filename = path.file_name().unwrap();
        fs::copy(path, save_path.join(&filename)).map_err(BlobError::IO)?;
    }
    if verbose {
        println!("{}\t{}", blob_ref.reference(), path.to_str().unwrap());
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
///
/// The function calls [`add_file`] to import the single files. The argument `verbose`
/// is passed to the  [`add_file`] function. Any errors thrown by `add_file` are printed
/// to `stderr`.
///
/// # Examples
///
/// ```no_run
/// # use std::path::Path;
/// # use rustore::blob::add_files;
/// let paths = vec![Path::new("/path/to/my/files/")];
/// let blob_refs = add_files(paths, false).unwrap();
/// ```
pub fn add_files(paths: Vec<&Path>, verbose: bool) -> Result<Vec<BlobRef>> {
    let paths: Vec<PathBuf> = paths
        .par_iter()
        .flat_map(|p| collect_file_paths(p))
        .collect();

    let pb = ProgressBar::new(paths.len() as u64);
    pb.set_style(ProgressStyle::default_bar().template(
        "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})",
    ));

    let (blob_refs, errors): (Vec<_>, Vec<_>) = paths
        .par_iter()
        .progress_with(pb)
        .map(|p| add_file(p, verbose))
        .partition(Result::is_ok);

    let blob_refs = blob_refs.into_iter().map(Result::unwrap).collect();

    for error in errors.into_iter() {
        eprintln!("{}", error.unwrap_err().to_string());
    }

    Ok(blob_refs)
}
