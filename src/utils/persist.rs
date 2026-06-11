use super::globals::Globals;
use super::misc::AtomicF64Vec;
use crate::qt::settings::AppSettings;
use crate::{ColumnSort, HeaderColumn, SongField};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tokio::sync::RwLock;

/// Default output plugin type used when no configuration exists.
const DEFAULT_OUTPUT_PLUGIN_TYPE: i32 = 1;

/// Default background opacity percentage (0-100).
const DEFAULT_BACKGROUND_COLORIZATION: usize = 75;

/// Default background blur percentage (0-100).
const DEFAULT_BACKGROUND_BLUR: usize = 95;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub struct PersistentConfig {
    pub init_wizard: bool,
    pub mpd_socket: String,
    //pub search_groups: Vec<i32>,
    pub native_music_dir: String,
    pub native_output_plugin: i32,
    pub background_blur: usize,
    pub background_colorization: usize,
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
        let globals = Globals::get();

        // Default home directory path (used for fallback values)
        let xdg_home = std::env::home_dir().unwrap_or_else(|| PathBuf::from("/"));

        match fs::read_to_string(&globals.app_config_file) {
            Ok(settings_file) => toml::from_str(&settings_file).unwrap_or_else(|_| {
                eprintln!("Warning: Failed to parse settings file, using defaults");
                Self::default_values(&globals, &xdg_home)
            }),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    // Config file doesn't exist yet - this is expected on first run
                    Self::default_values(&globals, &xdg_home)
                } else {
                    eprintln!("Warning: Failed to read settings file: {}. Using defaults", e);
                    Self::default_values(&globals, &xdg_home)
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
            native_output_plugin: DEFAULT_OUTPUT_PLUGIN_TYPE,
            background_blur: DEFAULT_BACKGROUND_BLUR,
            background_colorization: DEFAULT_BACKGROUND_COLORIZATION,
        }
    }
}

impl From<AppSettings> for PersistentConfig {
    fn from(value: AppSettings) -> Self {
        Self {
            init_wizard: value.init_wizard,
            mpd_socket: value.mpd_socket.into(),
            native_music_dir: value.native_music_dir.into(),
            native_output_plugin: value.native_output_plugin.repr,
            background_blur: value.background_blur,
            background_colorization: value.background_colorization,
        }
    }
}

pub struct StateFile {
    pub header_columns: Vec<HeaderColumn>,
    pub column_sort: ColumnSort,
    pub active_group: SongField,
}

impl StateFile {
    pub fn load() -> Self {
        Self::default_values()
    }

    pub fn dump(self) {
        todo!()
    }

    pub fn default_values() -> Self {
        let header_columns = vec![
            HeaderColumn { name: "#".into(), width: 1f64 / 14f64, hidden: false },
            HeaderColumn { name: "Title".into(), width: 1f64 / 14f64, hidden: false },
            HeaderColumn { name: "Artist".into(), width: 1f64 / 14f64, hidden: false },
            HeaderColumn { name: "Album".into(), width: 1f64 / 14f64, hidden: false },
            HeaderColumn { name: "Date".into(), width: 1f64 / 14f64, hidden: false },
            HeaderColumn { name: "Genre".into(), width: 1f64 / 14f64, hidden: false },
            HeaderColumn { name: "Disc".into(), width: 1f64 / 14f64, hidden: false },
            HeaderColumn { name: "Composer".into(), width: 1f64 / 14f64, hidden: false },
            HeaderColumn { name: "Albumartist".into(), width: 1f64 / 14f64, hidden: false },
            HeaderColumn { name: "File".into(), width: 1f64 / 14f64, hidden: false },
            HeaderColumn { name: "Format".into(), width: 1f64 / 14f64, hidden: false },
            HeaderColumn { name: "Lastmodified".into(), width: 1f64 / 14f64, hidden: false },
            HeaderColumn { name: "Duration".into(), width: 1f64 / 14f64, hidden: false },
            HeaderColumn { name: "Directory".into(), width: 1f64 / 14f64, hidden: false },
        ];
        let column_sort = ColumnSort::Ascending(SongField::Track);
        let active_group = SongField::Directory;

        Self { header_columns, column_sort, active_group }
    }
}
