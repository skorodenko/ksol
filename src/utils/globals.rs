use crate::utils::init_hooks::init_dirs;
use std::path::PathBuf;
use std::sync::OnceLock;
use which::which;
use xdg::BaseDirectories;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Globals {
    pub app_data_dir: PathBuf,
    pub app_cache_dir: PathBuf,
    pub app_cover_cache: PathBuf,
    pub app_config_dir: PathBuf,
    pub app_config_file: PathBuf,
    pub app_state_file: PathBuf,
    pub mpd_binary: PathBuf,
    pub native_socket: String,
    pub native_config: String,
}

impl Globals {
    /// Get the singleton [`InternalSettings`] instance.
    pub fn get() -> &'static Self {
        static INSTANCE: OnceLock<Globals> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            init_dirs();
            Globals::default()
        })
    }
}

impl Default for Globals {
    fn default() -> Self {
        let xdg_dirs = BaseDirectories::with_prefix("ksol");

        // Use `expect` with descriptive messages since this runs once at startup
        let app_config = xdg_dirs.get_config_home().expect(
            "Failed to determine config directory. \
             Is XDG_DATA_HOME or $HOME set?",
        );
        let app_cache = xdg_dirs.get_cache_home().expect(
            "Failed to determine cache directory. \
             Is XDG_CACHE_HOME or $HOME set?",
        );
        let app_data = xdg_dirs.get_data_home().expect(
            "Failed to determine data directory. \
             Is XDG_DATA_HOME or $HOME set?",
        );

        let mpd_data = app_data.join("mpd");

        Self {
            app_data_dir: app_data.clone(),
            app_cache_dir: app_cache.clone(),
            app_cover_cache: app_cache.join("covers"),
            app_config_dir: app_config.clone(),
            app_config_file: app_config.join("settings.toml"),
            app_state_file: app_data.join("state.bin"),
            mpd_binary: which("mpd").unwrap_or_default(),
            native_socket: mpd_data.join("socket").display().to_string(),
            native_config: mpd_data.join("mpd.conf").display().to_string(),
        }
    }
}
