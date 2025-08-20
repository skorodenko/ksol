use crate::rust::entities::SongField;
use once_cell::sync::OnceCell;
use serde;
use xdg::BaseDirectories;
use std::fs::create_dir;
use std::path::{Path, PathBuf};


#[derive(Debug)]
pub struct InternalSettings {
    pub native_socket: String,
    pub native_config: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct Settings {
    pub mpd_socket: String,
    pub search_groups: Vec<SongField>,
}

impl Settings {
    pub fn load() -> &'static Self {
        static INSTANCE: OnceCell<Settings> = OnceCell::new();
        INSTANCE.get_or_init(Settings::default)
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
        let xdg_dirs = BaseDirectories::with_prefix("ksol");
        let app_config = xdg_dirs.get_config_home().unwrap();
        let app_data = xdg_dirs.get_data_home().unwrap();
        //let mpd_config = app_config.join(("/mpd"));

        create_dir(app_config);
        create_dir(app_data);
        //create_dir(mpd_config);
        
        Self {
            mpd_socket: "".to_string(),
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
        //let mpd_confg = Path::from(app_config).push("mpd");
        
        Self {
            native_socket: "".to_string(),
            native_config: "".to_string(),
        }
    }
}
