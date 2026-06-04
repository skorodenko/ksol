use crate::SongField;

use super::globals::Globals;
use std::fs;
use std::path::{Path, PathBuf};

/// Default output plugin type used when no configuration exists.
const DEFAULT_OUTPUT_PLUGIN_TYPE: &str = "pipewire";

/// Default background opacity percentage (0-100).
const DEFAULT_BACKGROUND_OPACITY: usize = 75;

/// Default background blur percentage (0-100).
const DEFAULT_BACKGROUND_BLUR: usize = 95;

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
        let internal_settings = Globals::get();

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
        let internal_settings = Globals::get();

        // Default home directory path (used for fallback values)
        let xdg_home =
            std::env::home_dir().unwrap_or_else(|| PathBuf::from("/"));

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
    fn default_values(globals: &Globals, xdg_home: &Path) -> Self {
        Self {
            init_wizard: true,
            mpd_socket: globals.native_socket.clone(),
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
