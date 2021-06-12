use rustore::blob::BlobRef;
use std::{io, io::Write};
pub fn delete_blobs<'a, I>(hashes: I, interactive: bool)
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

pub fn check_blobs<'a, I>(hashes: I, show_metadata: bool)
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
