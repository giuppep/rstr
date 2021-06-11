use std::{io, io::Write, path::PathBuf};
mod cli;
mod server;
use clap::value_t_or_exit;
use cli::app;
use rustore::blob::{self, BlobRef};
use uuid::Uuid;
fn delete_blobs<'a, I>(hashes: I, interactive: bool)
where
    I: Iterator<Item = &'a str>,
{
    for hash in hashes {
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

        if interactive {
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

fn check_blobs<'a, I>(hashes: I, show_metadata: bool)
where
    I: Iterator<Item = &'a str>,
{
    for hash in hashes {
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

fn generate_token(data_store_path: PathBuf) -> String {
    let token = Uuid::new_v4()
        .to_simple()
        .encode_upper(&mut Uuid::encode_buffer())
        .to_string();
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(data_store_path.join(".tokens"))
        .expect("Can't open file.");
    writeln!(&mut file, "{}", token).unwrap();

    token
}

fn main() {
    let clap_matches = app().get_matches();

    let data_store_path = clap_matches.value_of("data_store_path").unwrap();
    std::env::set_var("RUSTORE_DATA_PATH", data_store_path);

    if let Some(clap_matches) = clap_matches.subcommand_matches("add") {
        let input_paths: Vec<PathBuf> = clap_matches
            .values_of("files")
            .unwrap()
            .into_iter()
            .map(PathBuf::from)
            .collect();
        blob::add_files(&input_paths[..], true).unwrap();
    }

    if let Some(clap_matches) = clap_matches.subcommand_matches("check") {
        let show_metadata = clap_matches.is_present("metadata");
        let hashes = clap_matches.values_of("refs").unwrap();

        check_blobs(hashes, show_metadata);
    }

    if let Some(clap_matches) = clap_matches.subcommand_matches("delete") {
        let hashes = clap_matches.values_of("refs").unwrap();
        let interactive = clap_matches.is_present("interactive");

        delete_blobs(hashes, interactive);
    }

    if let Some(clap_matches) = clap_matches.subcommand_matches("server") {
        if let Some(clap_matches) = clap_matches.subcommand_matches("start") {
            let port = value_t_or_exit!(clap_matches.value_of("port"), u16);
            let log_level = value_t_or_exit!(clap_matches.value_of("log_level"), log::Level);
            let tmp_folder = value_t_or_exit!(clap_matches.value_of("tmp_folder"), PathBuf);

            let config = server::Config::new(port, log_level, tmp_folder);
            server::start_server(config).unwrap()
        }

        if let Some(_) = clap_matches.subcommand_matches("generate-token") {
            let token = generate_token(PathBuf::from(data_store_path));
            println!("{}", token)
        }
    }
}
