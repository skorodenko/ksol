use crate::rust::entities::SongField;
use crate::rust::init_hooks::init_configs;
use serde;
use std::fs::create_dir;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tokio::sync::RwLock;
use which::which;
use xdg::BaseDirectories;

#[derive(Debug, serde::Serialize)]
pub struct InternalSettings {
    pub app_data_dir: PathBuf,
    pub app_cache_dir: PathBuf,
    pub app_config_dir: PathBuf,
    pub mpd_binary: PathBuf,
    pub native_socket: String,
    pub native_config: String,
    pub native_music_dir: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Settings {
    pub mpd_socket: String,
    pub search_groups: Vec<SongField>,
}

impl Settings {
    pub fn load() -> &'static RwLock<Settings> {
        static INSTANCE: OnceLock<RwLock<Settings>> = OnceLock::new();
        INSTANCE.get_or_init(|| RwLock::new(Settings::default()))
    }

    pub fn init_files() {
        let internal_settings = InternalSettings::load();

        let xdg_dirs = BaseDirectories::with_prefix("ksol");
        let app_config = xdg_dirs.get_config_home().unwrap();
        let app_data = xdg_dirs.get_data_home().unwrap();
        let mpd_config = app_config.join("mpd");
        let mpd_data = app_data.join("mpd");

        let _ = create_dir(app_config);
        let _ = create_dir(app_data);
        let _ = create_dir(mpd_config);
        let _ = create_dir(mpd_data);

        init_configs(internal_settings);
    }
}

impl InternalSettings {
    pub fn load() -> &'static Self {
        static INSTANCE: OnceLock<InternalSettings> = OnceLock::new();
        INSTANCE.get_or_init(InternalSettings::default)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::init_files();

        Self {
            mpd_socket: "/home/rinkuro/.local/share/ksol/mpd/socket".to_string(),
            search_groups: vec![SongField::Directory, SongField::Artist, SongField::Album, SongField::Genre],
        }
    }
}

impl Default for InternalSettings {
    fn default() -> Self {
        let xdg_dirs = BaseDirectories::with_prefix("ksol");
        let app_config = xdg_dirs.get_config_home().unwrap();
        let app_cache = xdg_dirs.get_cache_home().unwrap();
        let app_data = xdg_dirs.get_data_home().unwrap();
        let mpd_config = app_config.join("mpd");

        Self {
            app_data_dir: app_data,
            app_cache_dir: app_cache,
            app_config_dir: app_config,
            mpd_binary: which("mpd").unwrap_or_default(),
            native_socket: "/home/rinkuro/.local/share/ksol/mpd/socket".to_string(),
            native_config: mpd_config.join("mpd.conf").to_str().unwrap().to_string(),
            native_music_dir: String::from("/home/rinkuro/Music"),
        }
    }
}
