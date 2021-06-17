use directories::BaseDirs;
use std::default::Default;
use std::path::PathBuf;

#[derive(Debug)]
pub struct ServerSettings {
    pub port: u16,
    pub log_level: log::Level,
    pub tmp_directory: PathBuf,
    pub token_store_path: PathBuf,
}

#[derive(Debug)]
pub struct Settings {
    pub data_store_dir: PathBuf,
    pub server: ServerSettings,
}

impl Default for ServerSettings {
    fn default() -> Self {
        let dirs = BaseDirs::new().unwrap();
        let token_store_path = dirs.config_dir().join("rustore").join(".tokens");
        ServerSettings {
            port: 3123,
            log_level: log::Level::Info,
            tmp_directory: PathBuf::from("/tmp/rustore/"),
            token_store_path,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        let dirs = BaseDirs::new().unwrap();
        let data_store_dir = dirs.data_dir().join("rustore");
        Settings {
            data_store_dir,
            server: ServerSettings::default(),
        }
    }
}

impl ServerSettings {
    pub fn set_env_vars(&self) {
        std::env::set_var("RUSTORE_TMP_FOLDER", &self.tmp_directory);
        std::env::set_var("RUSTORE_TOKEN_STORE_PATH", &self.token_store_path);
    }

    pub fn create_dirs(&self) {
        std::fs::create_dir_all(&self.tmp_directory).unwrap();
        std::fs::create_dir_all(&self.token_store_path.parent().unwrap()).unwrap();
    }
}
