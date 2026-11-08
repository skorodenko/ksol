use crate::OutputPlugin;
use crate::utils::persist::PersistentConfig;
use qtbridge::qobject;

pub struct QAppSettings {
    pub init_wizard: bool,
    pub mpd_socket: String,
    pub native_music_dir: String,
    pub native_output_plugin: OutputPlugin,
    pub background_blur: usize,
    pub background_colorization: usize,
}

#[qobject(Singleton)]
impl QAppSettings {
    fn dump(&self) {
        let config_file = PersistentConfig {
            init_wizard: self.init_wizard,
            mpd_socket: self.mpd_socket.to_string(),
            native_music_dir: self.native_music_dir.to_string(),
            native_output_plugin: self.native_output_plugin.to_string(),
            background_blur: self.background_blur,
            background_colorization: self.background_colorization,
        };
        tracing::debug!("Dumping settings file: {:?}", config_file);
        config_file.dump();
    }
}

impl Default for QAppSettings {
    fn default() -> Self {
        let config = PersistentConfig::load();

        tracing::debug!("Loaded settings file: {:?}", config);

        Self::from(config)
    }
}

impl From<PersistentConfig> for QAppSettings {
    fn from(value: PersistentConfig) -> Self {
        Self {
            init_wizard: value.init_wizard,
            mpd_socket: value.mpd_socket.into(),
            native_music_dir: value.native_music_dir.into(),
            native_output_plugin: OutputPlugin::Pipewire,
            background_blur: value.background_blur,
            background_colorization: value.background_colorization,
        }
    }
}
