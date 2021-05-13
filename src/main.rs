use clap::{App, Arg, SubCommand};
use std::fs;
use std::path::Path;
mod blob;
mod server;
use blob::BlobRef;

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

        let content = fs::read(input_path).expect("Could not read file");

        let blob_ref = BlobRef::compute(&content);
        println!("Blob reference: {}", &blob_ref);

        let save_path = &blob_ref.to_path();
        fs::create_dir_all(save_path).expect("Could not create save directory");
        fs::copy(input_path, save_path).expect("Could not save file.");

        // println!("File saved in {:?}", save_path);
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
}
