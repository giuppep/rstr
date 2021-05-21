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
        clap_matches
            .values_of("files")
            .unwrap()
            .for_each(|input_path| {
                let input_path = Path::new(input_path);
                let blob_ref = blob_store::add_file(input_path);
                println!("{}\t\t\t\t{}", input_path.to_str().unwrap(), blob_ref.hash)
            })
    }

    if let Some(clap_matches) = clap_matches.subcommand_matches("check") {
        let show_metadata = clap_matches.is_present("metadata");

        clap_matches.values_of("refs").unwrap().for_each(|hash| {
            let blob_ref = BlobRef::new(&hash);
            match blob_ref.exists() {
                true if show_metadata => println!(
                    "{}\t\tPRESENT\t\t{:?}",
                    blob_ref,
                    blob_ref.get_metadata().unwrap()
                ),
                true => println!("{}\t\tPRESENT", blob_ref),
                false => println!("{}\t\tMISSING", blob_ref),
            }
        })
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

    if let Some(clap_matches) = clap_matches.subcommand_matches("start") {
        let port = clap_matches.value_of("port").unwrap();
        server::start_server(String::from(port)).unwrap()
    }
}
