mod cli;
mod security;
mod server;
mod settings;
mod utils;
use clap::value_t_or_exit;
use cli::app;
use rustore::blob;
use security::generate_token;
use settings::Settings;
use std::path::PathBuf;
use utils::{check_blobs, delete_blobs};

fn main() {
    let clap_matches = app().get_matches();

    if let Some(clap_matches) = clap_matches.subcommand_matches("create-config") {
        let settings = Settings::default();
        settings
            .to_file(clap_matches.value_of("path").map(PathBuf::from))
            .unwrap();
        return;
    }

    let mut settings = Settings::from_file(clap_matches.value_of("config").map(PathBuf::from))
        .unwrap_or(Settings::default());

    if let Some(data_store_path) = clap_matches.value_of("data_store_path") {
        settings.data_store_dir = data_store_path.into();
    }

    settings.set_env_vars();

    if let Some(clap_matches) = clap_matches.subcommand_matches("add") {
        let input_paths: Vec<PathBuf> = clap_matches
            .values_of("files")
            .unwrap()
            .into_iter()
            .map(PathBuf::from)
            .collect();
        let threads = value_t_or_exit!(clap_matches.value_of("threads"), u8);
        let verbose = clap_matches.is_present("verbose");

        blob::add_files(&input_paths[..], threads, verbose);
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
        settings.server.set_env_vars();

        if let Some(clap_matches) = clap_matches.subcommand_matches("start") {
            if let Some(port) = clap_matches.value_of("port") {
                settings.server.port = port.parse().unwrap_or_default()
            }
            if let Some(log_level) = clap_matches.value_of("log_level") {
                settings.server.log_level = log_level.parse().unwrap()
            }

            if let Some(tmp_directory) = clap_matches.value_of("tmp_directory") {
                settings.server.tmp_directory = tmp_directory.parse().unwrap()
            }
            server::start_server(settings.server).unwrap()
        }

        if clap_matches.subcommand_matches("generate-token").is_some() {
            let token = generate_token();
            println!("{}", token)
        }
    }
}
