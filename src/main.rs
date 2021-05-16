use clap::{App, Arg, SubCommand};

use std::fs;
use std::path::Path;
mod blob;
mod server;
use blob::BlobRef;
use ignore::{WalkBuilder, WalkState};
use std::fs::File;
use std::io::Write;
use std::sync::mpsc;

fn add_file(path: &Path) -> BlobRef {
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

fn add_folder_multi_threaded(path: &Path) -> Vec<BlobRef> {
    let walker = WalkBuilder::new(path);
    let (tx, rx) = mpsc::channel();
    walker.build_parallel().run(|| {
        let tx = tx.clone();
        Box::new(move |entry| match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    let blob_ref = add_file(path);
                    tx.send(blob_ref).expect("Err");
                }
                WalkState::Continue
            }
            Err(_) => WalkState::Continue,
        })
    });

    drop(tx);
    let mut blob_refs: Vec<BlobRef> = vec![];
    for blob_ref in rx.iter() {
        blob_refs.push(blob_ref);
    }
    blob_refs
}

fn add_folder_single_threaded(path: &Path) -> Vec<BlobRef> {
    let walker = WalkBuilder::new(path);
    let mut blob_refs: Vec<BlobRef> = vec![];
    for entry in walker.build() {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    blob_refs.push(add_file(path));
                }
            }
            Err(_) => (),
        }
    }
    blob_refs
}
fn add_folder(path: &Path, parallel: bool) -> Vec<BlobRef> {
    assert!(path.is_dir());
    println!("Parallel {}", &parallel);

    let blob_refs = match parallel {
        true => add_folder_multi_threaded(path),
        false => add_folder_single_threaded(path),
    };

    println!("Imported {} files", blob_refs.len());
    blob_refs
}

fn main() {
    let clap_matches = App::new("rustore")
        .version("0.1.0")
        .author("Giuseppe Papallo <giuseppe@papallo.it>")
        .about("Simmple content addressable blob store")
        .subcommand(
            SubCommand::with_name("add")
                .about("Adds a new file to the blob store")
                .arg(
                    Arg::with_name("file")
                        .required(true)
                        .index(1)
                        .help("Path to the file to add"),
                ),
        )
        .subcommand(
            SubCommand::with_name("import")
                .about("Recursively imports files from a directory into the blob store")
                .arg(
                    Arg::with_name("dir")
                        .required(true)
                        .index(1)
                        .help("Path to the file to add"),
                )
                .arg(
                    Arg::with_name("parallel")
                        .long("parallel")
                        .required(false)
                        .help("Whether to run in parallel"),
                ),
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("Retrieves a file from the blob store")
                .arg(
                    Arg::with_name("hash")
                        .required(true)
                        .index(1)
                        .help("The hash of the file to retrieve"),
                ),
        )
        .subcommand(
            SubCommand::with_name("start")
                .about("Starts the blob store server")
                .arg(
                    Arg::with_name("port")
                        .long("port")
                        .required(false)
                        .takes_value(true)
                        .help("The port on which to run"),
                ),
        )
        .get_matches();

    if let Some(clap_matches) = clap_matches.subcommand_matches("add") {
        let input_path = Path::new(clap_matches.value_of("file").unwrap());

        let blob_ref = add_file(input_path);
        println!("Blob reference {}", blob_ref)
    }

    if let Some(clap_matches) = clap_matches.subcommand_matches("get") {
        let hash = clap_matches.value_of("hash").unwrap();

        let blob_ref = BlobRef::new(&hash);
        if blob_ref.exists() {
            println!("Retrieved {}", blob_ref)
        } else {
            println!("No blob corresponding to {}", blob_ref)
        }
    }

    if let Some(clap_matches) = clap_matches.subcommand_matches("start") {
        let port = clap_matches.value_of("port").unwrap_or("3123");
        server::start_server(String::from(port)).unwrap()
    }

    if let Some(clap_matches) = clap_matches.subcommand_matches("import") {
        let input_path = Path::new(clap_matches.value_of("dir").unwrap());
        let parallel = clap_matches.is_present("parallel");

        let blob_refs = add_folder(input_path, parallel);
        for blob_ref in &blob_refs[..1] {
            println!("Blob reference {:?}", blob_ref)
        }
    }
}
