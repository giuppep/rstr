use crate::blob::BlobRef;
use ignore::{WalkBuilder, WalkState};
use std::fs;
use std::path::Path;
use std::sync::mpsc;

pub fn add_file(path: &Path, verbose: bool) -> BlobRef {
    assert!(path.is_file());

    let blob_ref = BlobRef::from_path(path).unwrap();
    if !blob_ref.exists() {
        let save_path = &blob_ref.to_path();
        fs::create_dir_all(save_path).expect("Could not create save directory");
        let filename = path.file_name().unwrap();
        fs::copy(path, save_path.join(&filename)).expect("Could not copy");
    }
    if verbose {
        println!("{}\t{}", blob_ref.reference(), path.to_str().unwrap());
    }
    blob_ref
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
                    let blob_ref = add_file(path, verbose);
                    tx.send(blob_ref).expect("Err");
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
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    let blob_ref = add_file(path, verbose);
                    blob_refs.push(blob_ref);
                }
            }
            Err(_) => (),
        }
    }
    blob_refs
}
pub fn add_folder(path: &Path, parallel: bool, verbose: bool) -> Vec<BlobRef> {
    assert!(path.is_dir());

    let blob_refs = match parallel {
        true => add_folder_multi_threaded(path, verbose),
        false => add_folder_single_threaded(path, verbose),
    };
    if verbose {
        println!("Imported {} files", blob_refs.len());
    }
    blob_refs
}
