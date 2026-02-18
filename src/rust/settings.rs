use crate::rust::entities::{ColumnSort, SongField};

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
    pub app_cover_cache: PathBuf,
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
    pub output_plugin_type: String,
    pub search_groups: Vec<SongField>,
    pub column_width: Vec<f64>,
    pub column_sort: ColumnSort,
    pub active_group: SongField,
    pub background_opacity: usize,
    pub background_blur: usize,
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
        let settings = Self::load().blocking_read().clone();
        let settings_file = toml::to_string(&settings).expect("Failed to serialize settings");
        let internal_settings = InternalSettings::load();
        fs::write(&internal_settings.app_config_file, settings_file).expect("Failed to write settings file");
    }

    fn init_dirs() {
        let xdg_dirs = BaseDirectories::with_prefix("ksol");
        let app_config = xdg_dirs.get_config_home().unwrap();
        let app_data = xdg_dirs.get_data_home().unwrap();
        let app_cache = xdg_dirs.get_cache_home().unwrap();

        // Create all directories at once to avoid duplicate calls
        let dirs_to_create =
            [&app_config, &app_data, &app_cache, &app_config.join("mpd"), &app_data.join("mpd"), &app_cache.join("mpd")];

        for dir in dirs_to_create {
            let _ = fs::create_dir_all(dir);
        }
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
            app_cache_dir: app_cache.clone(),
            app_cover_cache: app_cache.join("covers"),
            app_config_dir: app_config.clone(),
            app_config_file: app_config.join("settings.toml").display().to_string(),
            mpd_binary: which("mpd").unwrap_or_default(),
            native_socket: mpd_data.join("socket").display().to_string(),
            native_config: mpd_data.join("mpd.conf").display().to_string(),
        }
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
                native_music_dir: xdg_home.join("Music/").display().to_string(),
                output_plugin_type: String::from("pipewire"),
                search_groups: vec![SongField::Directory, SongField::Artist, SongField::Album, SongField::Genre],
                column_width: SongField::iter().map(|_| 1_f64 / 14_f64).collect(),
                column_sort: ColumnSort::Ascending(SongField::Track),
                active_group: SongField::Directory,
                background_opacity: 75,
                background_blur: 95,
            },
        }
    }
}
