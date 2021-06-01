use clap::{crate_authors, crate_version, App, AppSettings, Arg, SubCommand};

pub fn app() -> App<'static, 'static> {
    let app = App::new("rustore")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Simple content addressable blob store")
        .global_setting(AppSettings::ArgRequiredElseHelp)
        .arg(
            Arg::with_name("data_store_path")
                .env("RUSTORE_DATA_PATH")
                .long("data-store")
                .short("d")
                .value_name("PATH")
                .required(true)
                .help("Where rustore saves the blobs"),
        )
        .subcommand(
            SubCommand::with_name("add")
                .about("Adds a new file to the blob store.")
                .arg(
                    Arg::with_name("files")
                        .required(true)
                        .index(1)
                        .multiple(true)
                        .value_name("FILE")
                        .help("Path to the file to add"),
                ),
        )
        .subcommand(
            SubCommand::with_name("delete")
                .about("Adds a new file to the blob store.")
                .arg(
                    Arg::with_name("refs")
                        .required(true)
                        .index(1)
                        .value_name("REF")
                        .multiple(true)
                        .help("The reference of the blobs to delete"),
                )
                .arg(
                    Arg::with_name("interactive")
                        .required(false)
                        .takes_value(false)
                        .short("i")
                        .help("Ask for confirmation before deleting each blob."),
                ),
        )
        .subcommand(
            SubCommand::with_name("import")
                .about("Recursively imports files from a directory into the blob store.")
                .arg(
                    Arg::with_name("dir")
                        .required(true)
                        .index(1)
                        .value_name("DIRECTORY")
                        .help("Path to the directory to add"),
                )
                .arg(
                    Arg::with_name("parallel")
                        .long("parallel")
                        .required(false)
                        .help("Whether to run in parallel"),
                ),
        )
        .subcommand(
            SubCommand::with_name("check")
                .about("Given a list of refs, checks if they are present in the blob store.")
                .arg(
                    Arg::with_name("refs")
                        .required(true)
                        .index(1)
                        .value_name("REF")
                        .multiple(true)
                        .help("The reference of the blobs to check"),
                )
                .arg(
                    Arg::with_name("metadata")
                        .long("metadata")
                        .required(false)
                        .help("Prints the blob's metadata"),
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
                        .value_name("PORT")
                        .default_value("3123")
                        .help("The port on which to run"),
                )
                .arg(
                    Arg::with_name("log_level")
                        .long("log-level")
                        .required(false)
                        .takes_value(true)
                        .value_name("LEVEL")
                        .env("RUSTORE_LOG_LEVEL")
                        .possible_values(&["info", "debug", "error"])
                        .help("The level of logging"),
                )
                .arg(
                    Arg::with_name("tmp_folder")
                        .long("tmp-folder")
                        .required(false)
                        .takes_value(true)
                        .value_name("PATH")
                        .env("RUSTORE_TMP_FOLDER")
                        .default_value("/tmp/.rustore/")
                        .help("Path to a tmp folder for rustore"),
                ),
        );
    app
}
