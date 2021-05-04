use clap::{App, Arg, SubCommand};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

const DATA_LOC: &str = "/tmp/rustore/";

struct Blob {
    filename: String,
    content: Vec<u8>,
    hash: String,
}

impl Blob {
    fn from_path(path: &Path) -> Blob {
        let filename = path
            .file_name()
            .expect("Something went wrong extracting file name")
            .to_str()
            .expect("Could not convert filename to string")
            .to_string();
        let content = fs::read(path).unwrap();
        let hash = hash_content(&content);
        Blob {
            filename,
            content,
            hash,
        }
    }

    fn from_hash(hash: &str) -> Result<Blob, &'static str> {
        let dir = hash_to_path(hash);
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let content = fs::read(&path).unwrap();
            let filename = path
                .file_name()
                .expect("Something went wrong extracting file name")
                .to_str()
                .expect("Could not convert filename to string")
                .to_string();
            let blob = Blob {
                filename,
                content,
                hash: String::from_str(&hash).expect("oops"),
            };
            return Ok(blob);
        }
        Err("File Not Found")
    }

    fn get_dir(&self) -> PathBuf {
        hash_to_path(&self.hash)
    }
}

impl std::fmt::Display for Blob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Blob({}, {})", self.filename, &self.hash[..10])
    }
}

fn hash_content(content: &Vec<u8>) -> String {
    format!("{:x}", Sha256::digest(content))
}

fn hash_to_path(hash: &str) -> PathBuf {
    let path = Path::new(DATA_LOC)
        .join(&hash[0..2])
        .join(&hash[2..4])
        .join(&hash[4..6])
        .join(&hash[6..]);

    path
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

        let mut path = blob.get_dir();

        fs::create_dir_all(&path).expect("Cannot create dir");

        path = path.join(blob.filename);

        fs::write(&path, blob.content).expect("Unable to write file");

        println!("File saved in {:?}", path);
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
