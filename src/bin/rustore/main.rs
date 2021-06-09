use std::{io, io::Write, path::Path, path::PathBuf};
mod cli;
mod server;
use clap::value_t_or_exit;
use cli::app;
use rustore::blob::{self, BlobRef};

fn main() {
    let clap_matches = app().get_matches();

    let data_store_path = clap_matches.value_of("data_store_path").unwrap();
    std::env::set_var("RUSTORE_DATA_PATH", data_store_path);

    if let Some(clap_matches) = clap_matches.subcommand_matches("add") {
        let input_paths = clap_matches
            .values_of("files")
            .unwrap()
            .into_iter()
            .map(|p| Path::new(p))
            .collect();
        blob::add_files(input_paths, true).unwrap();
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

    if let Some(clap_matches) = clap_matches.subcommand_matches("start") {
        let port = value_t_or_exit!(clap_matches.value_of("port"), u16);
        let log_level = value_t_or_exit!(clap_matches.value_of("log_level"), log::Level);
        let tmp_folder = value_t_or_exit!(clap_matches.value_of("tmp_folder"), PathBuf);

        let config = server::Config::new(port, log_level, tmp_folder);
        server::start_server(config).unwrap()
    }
}
