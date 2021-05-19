use clap::{App, AppSettings, Arg, SubCommand};

pub fn app() -> App<'static, 'static> {
    let app = App::new("rustore")
        .version("0.1.0")
        .author("Giuseppe Papallo <giuseppe@papallo.it>")
        .about("Simmple content addressable blob store")
        .global_setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("add")
                .about("Adds a new file to the blob store")
                .arg(
                    Arg::with_name("file")
                        .required(true)
                        .index(1)
                        .value_name("FILE")
                        .help("Path to the file to add"),
                ),
        )
        .subcommand(
            SubCommand::with_name("import")
                .about("Recursively imports files from a directory into the blob store")
                .arg(
                    Arg::with_name("dir")
                        .required(true)
                        .index(1)
                        .value_name("DIRECTORY")
                        .help("Path to the directory to add"),
                )
                .arg(
                    Arg::with_name("output")
                        .long("output")
                        .short("o")
                        .takes_value(true)
                        .value_name("FILE")
                        .default_value("/tmp/rustore/output.json")
                        .required(false)
                        .help("Where to save the output of the import."),
                )
                .arg(
                    Arg::with_name("parallel")
                        .long("parallel")
                        .required(false)
                        .help("Whether to run in parallel"),
                ),
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("Retrieves a file from the blob store")
                .arg(
                    Arg::with_name("hash")
                        .required(true)
                        .index(1)
                        .value_name("HASH")
                        .help("The hash of the file to retrieve"),
                )
                .arg(
                    Arg::with_name("metadata")
                        .long("metadata")
                        .required(false)
                        .help("Return just the metadata"),
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
                ),
        );
    app
}
