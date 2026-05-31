use crate::SongField;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use which::which;
use xdg::BaseDirectories;

/// Default output plugin type used when no configuration exists.
const DEFAULT_OUTPUT_PLUGIN_TYPE: &str = "pipewire";

/// Default background opacity percentage (0-100).
const DEFAULT_BACKGROUND_OPACITY: usize = 75;

/// Default background blur percentage (0-100).
const DEFAULT_BACKGROUND_BLUR: usize = 95;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct InternalSettings {
    pub app_data_dir: PathBuf,
    pub app_cache_dir: PathBuf,
    pub app_cover_cache: PathBuf,
    pub app_config_dir: PathBuf,
    pub app_config_file: PathBuf,
    pub mpd_binary: PathBuf,
    pub native_socket: String,
    pub native_config: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub struct PersistentConfig {
    pub init_wizard: bool,
    pub mpd_socket: String,
    pub output_plugin_type: String,
    pub search_groups: Vec<SongField>,
    pub native_music_dir: String,
    pub background_opacity: usize,
    pub background_blur: usize,
}

impl PersistentConfig {
    /// Persist settings to the configuration file.
    ///
    /// Creates parent directories if they don't exist.
    /// Returns an error if serialization or file writing fails.
    pub fn dump(&self) -> std::io::Result<()> {
        let internal_settings = InternalSettings::get();

        // Ensure parent directory exists before writing
        if let Some(parent) = internal_settings.app_config_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let settings_file = toml::to_string(self).unwrap();
        fs::write(&internal_settings.app_config_file, settings_file)?;
        Ok(())
    }

    /// Load settings from disk, returning a default configuration if the file
    /// doesn't exist or contains invalid TOML.
    pub fn load() -> Self {
        let internal_settings = InternalSettings::get();

        // Default home directory path (used for fallback values)
        let xdg_home = std::env::home_dir().unwrap_or_else(|| PathBuf::from("/"));

        match fs::read_to_string(&internal_settings.app_config_file) {
            Ok(settings_file) => {
                toml::from_str(&settings_file).unwrap_or_else(|_| {
                    eprintln!(
                        "Warning: Failed to parse settings file, using defaults"
                    );
                    Self::default_values(&internal_settings, &xdg_home)
                })
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    // Config file doesn't exist yet - this is expected on first run
                    Self::default_values(&internal_settings, &xdg_home)
                } else {
                    eprintln!(
                        "Warning: Failed to read settings file: {}. Using defaults",
                        e
                    );
                    Self::default_values(&internal_settings, &xdg_home)
                }
            }
        }
    }

    /// Build default [`Settings`] from internal configuration and home directory.
    fn default_values(
        internal_settings: &InternalSettings,
        xdg_home: &Path,
    ) -> Self {
        Self {
            init_wizard: true,
            mpd_socket: internal_settings.native_socket.clone(),
            native_music_dir: xdg_home.join("Music/").display().to_string(),
            output_plugin_type: DEFAULT_OUTPUT_PLUGIN_TYPE.to_owned(),
            search_groups: vec![
                SongField::Directory,
                SongField::Artist,
                SongField::Album,
                SongField::Genre,
            ],
            background_opacity: DEFAULT_BACKGROUND_OPACITY,
            background_blur: DEFAULT_BACKGROUND_BLUR,
        }
    }
}

impl InternalSettings {
    /// Get the singleton [`InternalSettings`] instance.
    pub fn get() -> &'static Self {
        static INSTANCE: OnceLock<InternalSettings> = OnceLock::new();
        INSTANCE.get_or_init(InternalSettings::default)
    }
}

impl Default for InternalSettings {
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
            app_data_dir: app_data,
            app_cache_dir: app_cache.clone(),
            app_cover_cache: app_cache.join("covers"),
            app_config_dir: app_config.clone(),
            app_config_file: app_config.join("settings.toml"),
            mpd_binary: which("mpd").unwrap_or_default(),
            native_socket: mpd_data.join("socket").display().to_string(),
            native_config: mpd_data.join("mpd.conf").display().to_string(),
        }
    }
}
