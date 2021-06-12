use clap::{crate_authors, crate_version, App, AppSettings, Arg, SubCommand};

fn server_commands() -> App<'static, 'static> {
    SubCommand::with_name("server")
        .setting(AppSettings::SubcommandRequiredElseHelp)
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
        )
        .subcommand(SubCommand::with_name("generate-token").about("Generate an API Token."))
}
pub fn app() -> App<'static, 'static> {
    let app = App::new("rustore")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Simple content addressable blob store")
        .setting(AppSettings::SubcommandRequiredElseHelp)
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
                .setting(AppSettings::ArgRequiredElseHelp)
                .about("Adds a new file/directory to the blob store.")
                .arg(
                    Arg::with_name("files")
                        .required(true)
                        .index(1)
                        .multiple(true)
                        .value_name("PATH")
                        .help("Path to the file/directory to add"),
                ),
        )
        .subcommand(
            SubCommand::with_name("delete")
                .setting(AppSettings::ArgRequiredElseHelp)
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
                        .short("I")
                        .help("Ask for confirmation before deleting each blob."),
                ),
        )
        .subcommand(
            SubCommand::with_name("check")
                .setting(AppSettings::ArgRequiredElseHelp)
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
        .subcommand(server_commands());
    app
}
