use clap::{App, Arg, SubCommand};
use std::path::Path;
mod blob;
use blob::Blob;

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
        .get_matches();

    if let Some(clap_matches) = clap_matches.subcommand_matches("add") {
        let input_path = Path::new(clap_matches.value_of("file").unwrap());
        println!("{:?}", input_path);

        let blob = Blob::from_path(&input_path);
        println!("{}", blob);

        println!("File's hash: {}", blob.hash);

        let path = blob.save();
        println!("File saved in {:?}", &path);
    }

    if let Some(clap_matches) = clap_matches.subcommand_matches("get") {
        let hash = clap_matches.value_of("hash").unwrap();

        let blob = Blob::from_hash(&hash);

        println!("Retrieved {}", blob.expect("Not found"));
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_hashing() {
//         let f = std::fs::read("test/test_file.txt").unwrap();
//         let h = hash_file(&f);
//         assert_eq!(h, "f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de")
//     }
// }
