use crate::rust::entities::SongField;
use serde;
//use std::sync::Mutex;
use once_cell::sync::OnceCell;
use xdg::BaseDirectories;

#[derive(serde::Deserialize, Debug)]
pub struct MPDSettings {
    pub socket: String,
    pub native_socket: String,
    pub native_config: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct AppSettings {
    pub search_groups: Vec<SongField>,
}

#[derive(serde::Deserialize, Debug, Default)]
pub struct Settings {
    #[serde(default = "MPDSettings::default")]
    pub mpd: MPDSettings,

    #[serde(default = "AppSettings::default")]
    pub app: AppSettings,
}

impl Settings {
    pub fn load() -> &'static Self {
        let xdg_dirs = BaseDirectories::with_prefix("ksol");
        let app_config = xdg_dirs.get_config_home();
        let app_data = xdg_dirs.get_data_home();

        static INSTANCE: OnceCell<Settings> = OnceCell::new();
        INSTANCE.get_or_init(Settings::default)
    }
}

impl Default for MPDSettings {
    fn default() -> Self {
        Self {
            socket: "".to_string(),
            native_socket: "".to_string(),
            native_config: "".to_string(),
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            search_groups: vec![SongField::Directory, SongField::Artist, SongField::Album, SongField::Track],
        }
    }
}
