use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn project_dirs() -> ProjectDirs {
    ProjectDirs::from("", "", "rustore").unwrap()
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct ServerSettings {
    /// Port on which to run the rustore server
    pub port: u16,
    /// Level of logging for rustore server
    pub log_level: log::Level,
    /// Path to a directory where the server can store temporary files
    pub tmp_directory: PathBuf,
    /// Path to a file containing a list of valid API tokens
    pub token_store_path: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Settings {
    /// Path to the directory where all the blobs will be stored
    pub data_store_dir: PathBuf,
    /// Server settings, see [`ServerSettings`]
    pub server: ServerSettings,
}

impl Default for ServerSettings {
    fn default() -> Self {
        let token_store_path = project_dirs().config_dir().join(".tokens");
        ServerSettings {
            port: 3123,
            log_level: log::Level::Info,
            tmp_directory: std::env::temp_dir().join("rustore"),
            token_store_path,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        let data_store_dir = project_dirs().data_dir().into();
        Settings {
            data_store_dir,
            server: ServerSettings::default(),
        }
    }
}

impl Settings {
    /// Default path for the configuration file.
    fn default_config_path() -> PathBuf {
        project_dirs().config_dir().join("rustore.toml")
    }

    /// Load the rustore configuration from a `toml` file.
    ///
    /// # Examples
    /// ```no_run
    /// # use std::path::PathBuf;
    /// let config_file = PathBuf::from("/home/giuppep/.config/rustore.toml")
    /// let settings = Settings::from_file(config_file).unwrap()
    /// ```
    ///
    /// # Errors
    ///
    /// The function will error if the file cannot be read or deserialized properly.
    pub fn from_file(path: Option<PathBuf>) -> Result<Self, &'static str> {
        // TODO: proper error handling
        let path = path.unwrap_or(Settings::default_config_path());

        let content = match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(_) => return Err("File cannot be read"),
        };

        match toml::from_str(&content) {
            Ok(settings) => Ok(settings),
            Err(_) => Err("cannot deserialize"),
        }
    }

    /// Save the rustore configuration to a file. If the path is not specified, it
    /// saves it to the default path.
    pub fn to_file(&self, path: Option<PathBuf>) -> std::io::Result<()> {
        let path = path.unwrap_or(Settings::default_config_path());

        let toml_str = toml::to_string(&self).unwrap();
        let mut file = File::create(&path)?;
        file.write_all(&toml_str.as_bytes())?;
        println!("Created config in {:?}", &path);

        Ok(())
    }
}

impl ServerSettings {
    /// Create all directories definied in the current configuration.
    pub fn create_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.tmp_directory)?;
        std::fs::create_dir_all(&self.token_store_path.parent().unwrap())?;
        Ok(())
    }
}
