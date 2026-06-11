use qobject::*;

#[cxx_qt::bridge]
mod qobject {

    extern "C++" {
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;

        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[repr(i32)]
    #[namespace = "OutputPlugin"]
    enum OutputPlugin {
        Pipewire = 0x1,
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(bool, init_wizard, cxx_name = "initWizard")]
        #[qproperty(QString, mpd_socket, cxx_name = "mpdSocket")]
        #[qproperty(QString, native_music_dir, cxx_name = "nativeMusicDir")]
        #[qproperty(OutputPlugin, native_output_plugin, cxx_name = "nativeOutputPlugin")]
        #[qproperty(usize, background_blur, cxx_name = "backgroundBlur")]
        #[qproperty(usize, background_colorization, cxx_name = "backgroundColorization")]
        type QAppSettings = super::AppSettings;
    }
}

use crate::utils::persist::PersistentConfig;

pub struct AppSettings {
    pub init_wizard: bool,
    pub mpd_socket: QString,
    pub native_music_dir: QString,
    pub native_output_plugin: OutputPlugin,
    pub background_blur: usize,
    pub background_colorization: usize,
}

impl AppSettings {
    fn save(self) {
        let config = PersistentConfig::from(self);
        config.dump().inspect_err(|e| eprintln!("Failed to save config {}", e));
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        //let config = PersistentConfig::load();
        //Self::from(config)
        Self {
            init_wizard: Default::default(),
            mpd_socket: Default::default(),
            native_music_dir: Default::default(),
            native_output_plugin: OutputPlugin::Pipewire,
            background_blur: Default::default(),
            background_colorization: Default::default(),
        }
    }
}

impl From<PersistentConfig> for AppSettings {
    fn from(value: PersistentConfig) -> Self {
        Self {
            init_wizard: value.init_wizard,
            mpd_socket: value.mpd_socket.into(),
            native_music_dir: value.native_music_dir.into(),
            native_output_plugin: OutputPlugin { repr: value.native_output_plugin },
            background_blur: value.background_blur,
            background_colorization: value.background_colorization,
        }
    }
}
