use crate::blob::BlobRef;
use crate::error::{BlobError, Result};
use ignore::{WalkBuilder, WalkState};
use std::path::Path;
use std::sync::mpsc;
use std::{fs, io};

pub fn add_file(path: &Path, verbose: bool) -> Result<BlobRef> {
    if !path.is_file() {
        return Err(BlobError::IO(io::Error::from(io::ErrorKind::InvalidInput)));
    }

    let blob_ref = BlobRef::from_path(path)?;
    if !blob_ref.exists() {
        let save_path = &blob_ref.to_path();
        fs::create_dir_all(save_path)?;
        let filename = path.file_name().unwrap();
        fs::copy(path, save_path.join(&filename))?;
    }
    if verbose {
        println!("{}\t{}", blob_ref.reference(), path.to_str().unwrap());
    }
    Ok(blob_ref)
}

fn add_folder_multi_threaded(path: &Path, verbose: bool) -> Vec<BlobRef> {
    let walker = WalkBuilder::new(path);
    let (tx, rx) = mpsc::channel();
    walker.build_parallel().run(|| {
        let tx = tx.clone();
        Box::new(move |entry| match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    match add_file(path, verbose) {
                        Ok(blob_ref) => tx.send(blob_ref).expect("Err"),
                        Err(e) => eprintln!("{}", e),
                    }
                }
                WalkState::Continue
            }
            Err(_) => WalkState::Continue,
        })
    });

    drop(tx);
    let mut blob_refs = vec![];
    for blob_ref in rx.iter() {
        blob_refs.push(blob_ref);
    }
    blob_refs
}

fn add_folder_single_threaded(path: &Path, verbose: bool) -> Vec<BlobRef> {
    let walker = WalkBuilder::new(path);
    let mut blob_refs = vec![];
    for entry in walker.build() {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() {
                match add_file(path, verbose) {
                    Ok(blob_ref) => blob_refs.push(blob_ref),
                    Err(e) => eprintln!("{}", e),
                }
            }
        }
    }
    blob_refs
}
pub fn add_folder(path: &Path, parallel: bool, verbose: bool) -> Result<Vec<BlobRef>> {
    if !path.is_dir() {
        return Err(BlobError::IO(io::Error::from(io::ErrorKind::InvalidInput)));
    }

    let blob_refs = match parallel {
        true => add_folder_multi_threaded(path, verbose),
        false => add_folder_single_threaded(path, verbose),
    };

    if verbose {
        println!("Imported {} files", blob_refs.len());
    }
    Ok(blob_refs)
}
