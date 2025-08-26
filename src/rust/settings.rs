use crate::rust::entities::SongField;
use once_cell::sync::OnceCell;
use serde;
use std::fs::create_dir;
use std::path::{Path, PathBuf};
use xdg::BaseDirectories;

#[derive(Debug)]
pub struct InternalSettings {
    pub native_socket: String,
    pub native_config: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Settings {
    pub mpd_socket: String,
    pub search_groups: Vec<SongField>,
}

impl Settings {
    pub fn load() -> &'static Self {
        static INSTANCE: OnceCell<Settings> = OnceCell::new();
        INSTANCE.get_or_init(Settings::default)
    }

    pub fn init_files() {
        let xdg_dirs = BaseDirectories::with_prefix("ksol");
        let app_config = xdg_dirs.get_config_home().unwrap();
        let app_data = xdg_dirs.get_data_home().unwrap();
        let mpd_config = app_data.join("mpd");

        let _ = create_dir(app_config);
        let _ = create_dir(app_data);
        let _ = create_dir(mpd_config);
    }
}

impl InternalSettings {
    pub fn load() -> &'static Self {
        static INSTANCE: OnceCell<InternalSettings> = OnceCell::new();
        INSTANCE.get_or_init(InternalSettings::default)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::init_files();

        let xdg_dirs = BaseDirectories::with_prefix("ksol");
        let app_config = xdg_dirs.get_config_home().unwrap();
        let app_data = xdg_dirs.get_data_home().unwrap();
        let mpd_config = app_data.join("mpd");

        Self {
            mpd_socket: "localhost:6600".to_string(),
            search_groups: vec![
                SongField::Directory,
                SongField::Artist,
                SongField::Album,
                SongField::Track,
            ],
        }
    }
}

impl Default for InternalSettings {
    fn default() -> Self {
        let xdg_dirs = BaseDirectories::with_prefix("ksol");
        let app_config = xdg_dirs.get_config_home().unwrap();
        let app_data = xdg_dirs.get_data_home().unwrap();
        let mpd_config = app_data.join("mpd");

        Self {
            native_socket: "localhost:6600".to_string(),
            native_config: mpd_config.join("mpd.conf").to_str().unwrap().to_string(),
        }
    }
}
