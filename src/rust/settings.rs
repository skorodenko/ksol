use crate::rust::entities::{ColumnSort, SongField};
use serde;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
use strum::IntoEnumIterator;
use tokio::sync::RwLock;
use which::which;
use xdg::BaseDirectories;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct InternalSettings {
    pub app_data_dir: PathBuf,
    pub app_cache_dir: PathBuf,
    pub app_config_dir: PathBuf,
    pub app_config_file: String,
    pub mpd_binary: PathBuf,
    pub native_socket: String,
    pub native_config: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Settings {
    pub init_wizard: bool,
    pub mpd_socket: String,
    pub native_music_dir: String,
    pub search_groups: Vec<SongField>,
    pub column_width: Vec<f64>,
    pub column_sort: ColumnSort,
    pub active_group: SongField,
}

impl Settings {
    pub fn load() -> &'static RwLock<Settings> {
        static INSTANCE: OnceLock<RwLock<Settings>> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            Settings::init_dirs();
            RwLock::new(Settings::default())
        })
    }

    pub fn dump() {
        let settings: Settings = Self::load().blocking_read().clone();
        let settings_file = toml::to_string(&settings).expect("Failed to dump settings file");
        let internal_settings = InternalSettings::load();
        fs::write(&internal_settings.app_config_file, settings_file).expect("Failed to write settings file");
    }

    fn init_dirs() {
        let xdg_dirs = BaseDirectories::with_prefix("ksol");
        let app_config = xdg_dirs.get_config_home().unwrap();
        let app_data = xdg_dirs.get_data_home().unwrap();
        let mpd_config = app_config.join("mpd");
        let mpd_data = app_data.join("mpd");

        let _ = fs::create_dir(app_config);
        let _ = fs::create_dir(app_data);
        let _ = fs::create_dir(mpd_config);
        let _ = fs::create_dir(mpd_data);
    }
}

impl InternalSettings {
    pub fn load() -> &'static Self {
        static INSTANCE: OnceLock<InternalSettings> = OnceLock::new();
        INSTANCE.get_or_init(InternalSettings::default)
    }
}

impl Default for InternalSettings {
    fn default() -> Self {
        let xdg_dirs = BaseDirectories::with_prefix("ksol");
        let app_config = xdg_dirs.get_config_home().unwrap();
        let app_cache = xdg_dirs.get_cache_home().unwrap();
        let app_data = xdg_dirs.get_data_home().unwrap();
        let mpd_data = app_data.join("mpd");

        Self {
            app_data_dir: app_data,
            app_cache_dir: app_cache,
            app_config_dir: app_config.clone(),
            app_config_file: app_config.join("settings.toml").to_str().unwrap().to_string(),
            mpd_binary: which("mpd").unwrap_or_default(),
            native_socket: mpd_data.join("socket").to_str().unwrap().to_string(),
            native_config: mpd_data.join("mpd.conf").to_str().unwrap().to_string(),
        }
    }
}

impl Drop for Settings {
    fn drop(&mut self) {
        let settings_file = toml::to_string(self).expect("Failed to dump settings file");
        let internal_settings = InternalSettings::load();
        fs::write(&internal_settings.app_config_file, settings_file).expect("Failed to write settings file");
    }
}

impl Default for Settings {
    fn default() -> Self {
        let internal_settings = InternalSettings::load();
        let settings_file = fs::read_to_string(&internal_settings.app_config_file).unwrap_or_default();

        let xdg_home = std::env::home_dir().expect("Failed to get $HOME");

        match toml::from_str(&settings_file) {
            Ok(val) => val,
            Err(_) => Self {
                init_wizard: true,
                mpd_socket: internal_settings.native_socket.clone(),
                native_music_dir: xdg_home.join("Music/").to_str().unwrap().to_string(),
                search_groups: vec![SongField::Directory, SongField::Artist, SongField::Album, SongField::Genre],
                column_width: SongField::iter().map(|_| 1_f64 / 14_f64).collect(),
                column_sort: ColumnSort::Ascending(SongField::Track),
                active_group: SongField::Directory,
            },
        }
    }
}
