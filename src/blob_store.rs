use crate::blob::BlobRef;
use ignore::{WalkBuilder, WalkState};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::mpsc;

pub fn add_file(path: &Path) -> BlobRef {
    assert!(path.is_file());

    let blob_ref = BlobRef::from_path(&path).unwrap();
    if !blob_ref.exists() {
        let save_path = &blob_ref.to_path();
        fs::create_dir_all(save_path).expect("Could not create save directory");
        let filename = path.file_name().unwrap();
        fs::copy(path, save_path.join(&filename)).expect("Could not copy");
    }
    blob_ref
}

fn add_folder_multi_threaded(path: &Path) -> HashMap<String, String> {
    let walker = WalkBuilder::new(path);
    let (tx, rx) = mpsc::channel();
    walker.build_parallel().run(|| {
        let tx = tx.clone();
        Box::new(move |entry| match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    let blob_ref = add_file(path);
                    tx.send((String::from(path.to_str().unwrap()), blob_ref))
                        .expect("Err");
                }
                WalkState::Continue
            }
            Err(_) => WalkState::Continue,
        })
    });

    drop(tx);
    let mut output = HashMap::new();
    for (path, blob_ref) in rx.iter() {
        output.insert(path, blob_ref.hash);
    }
    output
}

fn add_folder_single_threaded(path: &Path) -> HashMap<String, String> {
    let walker = WalkBuilder::new(path);
    let mut output = HashMap::new();
    for entry in walker.build() {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    let blob_ref = add_file(path);
                    output.insert(String::from(path.to_str().unwrap()), blob_ref.hash);
                }
            }
            Err(_) => (),
        }
    }
    output
}
pub fn add_folder(path: &Path, parallel: bool) -> HashMap<String, String> {
    assert!(path.is_dir());

    let blob_refs = match parallel {
        true => add_folder_multi_threaded(path),
        false => add_folder_single_threaded(path),
    };

    println!("Imported {} files", blob_refs.len());
    blob_refs
}
