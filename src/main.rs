use std::{io, io::Write, path::Path};
mod blob;
mod blob_store;
mod cli;
mod error;
mod server;
use blob::BlobRef;
use cli::app;

fn main() {
    let clap_matches = app().get_matches();

    if let Some(clap_matches) = clap_matches.subcommand_matches("add") {
        for input_path in clap_matches.values_of("files").unwrap() {
            let input_path = Path::new(input_path);
            if let Err(e) = blob_store::add_file(input_path, true) {
                eprintln!("Could not add {:?}: {}", input_path, e)
            }
        }
    }

    if let Some(clap_matches) = clap_matches.subcommand_matches("check") {
        let show_metadata = clap_matches.is_present("metadata");

        for hash in clap_matches.values_of("refs").unwrap() {
            let blob_ref = match BlobRef::new(&hash) {
                Ok(blob_ref) => blob_ref,
                Err(_) => {
                    eprintln!("{}\t\tINVALID", &hash);
                    continue;
                }
            };

            match blob_ref.exists() {
                true if show_metadata => println!(
                    "{}\t\tPRESENT\t\t{:?}",
                    blob_ref,
                    blob_ref.metadata().unwrap()
                ),
                true => println!("{}\t\tPRESENT", blob_ref),
                false => println!("{}\t\tMISSING", blob_ref),
            }
        }
    }

    if let Some(clap_matches) = clap_matches.subcommand_matches("delete") {
        for hash in clap_matches.values_of("refs").unwrap() {
            let blob_ref = match BlobRef::new(&hash) {
                Ok(blob_ref) if !blob_ref.exists() => {
                    println!("{}\t\tMISSING", blob_ref);
                    continue;
                }
                Ok(blob_ref) => blob_ref,
                Err(_) => {
                    eprintln!("{}\t\tINVALID", &hash);
                    continue;
                }
            };

            if clap_matches.is_present("interactive") {
                let mut confirm = String::new();
                print!("Do you want to delete {}? [y/n]: ", blob_ref);
                io::stdout().flush().unwrap();
                io::stdin().read_line(&mut confirm).unwrap();

                if confirm.trim().to_ascii_lowercase() != "y" {
                    continue;
                }
            };

            match blob_ref.delete() {
                Ok(_) => println!("{}\t\tDELETED", blob_ref),
                Err(_) => eprintln!("{}\t\tERROR", blob_ref),
            }
        }
    }

    if let Some(clap_matches) = clap_matches.subcommand_matches("import") {
        let input_path = Path::new(clap_matches.value_of("dir").unwrap());
        let parallel = clap_matches.is_present("parallel");

        if let Err(e) = blob_store::add_folder(input_path, parallel, true) {
            eprintln!("Could not add {:?}: {}", input_path, e)
        }
    }

    if let Some(clap_matches) = clap_matches.subcommand_matches("start") {
        let port = clap_matches.value_of("port").unwrap();
        server::start_server(String::from(port)).unwrap()
    }
}
