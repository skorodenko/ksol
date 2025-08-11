use config::{Config, ConfigError, File, FileFormat};
use serde;
use xdg::BaseDirectories;

#[derive(serde::Deserialize, Debug)]
pub struct MPDSettings {
    pub socket: String,
    pub native_socket: String,
    pub native_config: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct Settings {
    pub mpd: MPDSettings,
}

impl Settings {
    pub fn load() -> Result<Self, ConfigError> {
        let xdg_dirs = BaseDirectories::with_prefix("ksol");
        let app_config = xdg_dirs.get_config_home();
        let app_data = xdg_dirs.get_data_home();

        Config::builder()
            .set_default("mpd.socket", "test")?
            .add_source(File::new("test", FileFormat::Toml))
            .build()?
            .try_deserialize()
    }
}
