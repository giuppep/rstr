use std::fs;
use std::path::Path;
mod blob;
mod blob_store;
mod cli;
mod server;
use blob::BlobRef;
use cli::app;
use std::io::Write;

fn main() {
    let clap_matches = app().get_matches();

    if let Some(clap_matches) = clap_matches.subcommand_matches("add") {
        let input_path = Path::new(clap_matches.value_of("file").unwrap());

        let blob_ref = blob_store::add_file(input_path);
        println!("Blob reference {}", blob_ref.hash)
    }

    if let Some(clap_matches) = clap_matches.subcommand_matches("get") {
        let hash = clap_matches.value_of("hash").unwrap();

        let blob_ref = BlobRef::new(&hash);
        if blob_ref.exists() {
            println!("Retrieved {}", blob_ref);
            if clap_matches.is_present("metadata") {
                println!("{:?}", blob_ref.get_metadata().unwrap());
            }
        } else {
            println!("No blob corresponding to {}", blob_ref)
        }
    }

    if let Some(clap_matches) = clap_matches.subcommand_matches("start") {
        let port = clap_matches.value_of("port").unwrap();
        server::start_server(String::from(port)).unwrap()
    }

    if let Some(clap_matches) = clap_matches.subcommand_matches("import") {
        let input_path = Path::new(clap_matches.value_of("dir").unwrap());
        let parallel = clap_matches.is_present("parallel");

        let output = blob_store::add_folder(input_path, parallel);

        let output_path = clap_matches.value_of("output").unwrap();

        let mut file = fs::File::create(&output_path).unwrap();
        write!(file, "{}", serde_json::to_string(&output).unwrap()).unwrap();
        println!("Saved lookup in {}", &output_path)
    }
}
