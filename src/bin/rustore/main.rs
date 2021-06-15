mod cli;
mod security;
mod server;
mod utils;
use clap::value_t_or_exit;
use cli::app;
use rustore::blob;
use security::generate_token;
use std::path::PathBuf;
use utils::{check_blobs, delete_blobs};

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
        blob::add_files(&input_paths[..], true);
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

        if clap_matches.subcommand_matches("generate-token").is_some() {
            let token = generate_token();
            println!("{}", token)
        }
    }
}
