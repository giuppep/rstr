use ignore::{WalkBuilder, WalkState};
#[cfg(feature = "progress_bar")]
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc;

/// Given a path to a directory it recursively walks all its children in parallel
/// and returns a list of paths to files.
pub fn collect_file_paths(path: &Path) -> Vec<PathBuf> {
    if path.is_file() {
        return vec![path.to_path_buf()];
    }

    let walker = WalkBuilder::new(path);
    let (tx, rx) = mpsc::channel();
    walker.build_parallel().run(|| {
        let tx = tx.clone();
        Box::new(move |entry| match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    tx.send(path.into()).expect("Err")
                }
                WalkState::Continue
            }
            Err(_) => WalkState::Continue,
        })
    });

    drop(tx);
    let mut paths = vec![];
    for path in rx.iter() {
        paths.push(path)
    }
    paths
}

#[cfg(feature = "progress_bar")]
pub fn progress_bar(length: u64) -> ProgressBar {
    let pb = ProgressBar::new(length);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})\n{msg}"),
    );
    pb
}
