use qobject::*;

use crate::utils::settings::{InternalSettings, Settings};
use crate::{ColumnSort, SongField};
use core::pin::Pin;
use num_traits::{FromPrimitive, ToPrimitive};
use std::net::TcpStream;
use std::os::unix::net::UnixStream;
use tracing;

#[derive(Default)]
pub struct SettingsModel;

impl qobject::QSettingsModel {
    /// Helper function to read settings and apply a closure to them
    fn with_settings_read<F, R>(f: F) -> R
    where
        F: FnOnce(&Settings) -> R,
    {
        let settings = Settings::load().blocking_read();
        f(&settings)
    }

    /// Helper function to write settings and apply a closure to them
    fn with_settings_write<F>(f: F)
    where
        F: FnOnce(&mut Settings),
    {
        let mut settings = Settings::load().blocking_write();
        f(&mut settings);
    }

    pub fn get_mpd_socket(self: Pin<&mut QSettingsModel>) -> QString {
        Self::with_settings_read(|settings| QString::from(&settings.mpd_socket))
    }

    pub fn set_mpd_socket(self: Pin<&mut QSettingsModel>, value: QString) {
        Self::with_settings_write(|settings| {
            settings.mpd_socket = value.into();
        });
        self.mpd_server_settings_update();
    }

    pub fn get_native_mpd_socket(self: Pin<&mut QSettingsModel>) -> QString {
        let isettings = InternalSettings::load();
        QString::from(&isettings.native_socket)
    }

    fn get_native_mpd_music_dir(self: Pin<&mut QSettingsModel>) -> QString {
        Self::with_settings_read(|settings| {
            QString::from(&settings.native_music_dir)
        })
    }

    fn set_native_mpd_music_dir(
        self: Pin<&mut QSettingsModel>,
        value: QString,
    ) {
        Self::with_settings_write(|settings| {
            settings.native_music_dir = value.into();
        });
        self.mpd_server_settings_update();
    }

    fn get_output_plugin_type(self: Pin<&mut QSettingsModel>) -> QString {
        Self::with_settings_read(|settings| {
            QString::from(&settings.output_plugin_type)
        })
    }

    fn set_output_plugin_type(self: Pin<&mut QSettingsModel>, value: QString) {
        Self::with_settings_write(|settings| {
            settings.output_plugin_type = value.into();
        });
        self.mpd_server_settings_update();
    }

    fn get_init_wizard(self: Pin<&mut QSettingsModel>) -> bool {
        Self::with_settings_read(|settings| settings.init_wizard)
    }

    fn set_init_wizard(self: Pin<&mut QSettingsModel>, value: bool) {
        Self::with_settings_write(|settings| {
            settings.init_wizard = value;
        });
    }

    pub fn get_column_width(
        self: Pin<&mut QSettingsModel>,
        column: usize,
    ) -> f64 {
        Self::with_settings_read(|settings| {
            *settings.column_width.get(column).unwrap()
        })
    }

    pub fn set_column_width(
        self: Pin<&mut QSettingsModel>,
        column: usize,
        value: f64,
    ) {
        Self::with_settings_write(|settings| {
            settings.column_width[column] = value;
        });
    }

    pub fn get_background_blur(self: Pin<&mut QSettingsModel>) -> usize {
        Self::with_settings_read(|settings| settings.background_blur)
    }

    pub fn set_background_blur(self: Pin<&mut QSettingsModel>, value: usize) {
        Self::with_settings_write(|settings| {
            settings.background_blur = value;
        });
        self.mpd_appearance_settings_update();
    }

    pub fn get_background_opacity(self: Pin<&mut QSettingsModel>) -> usize {
        Self::with_settings_read(|settings| settings.background_opacity)
    }

    pub fn set_background_opacity(
        self: Pin<&mut QSettingsModel>,
        value: usize,
    ) {
        Self::with_settings_write(|settings| {
            settings.background_opacity = value;
        });
        self.mpd_appearance_settings_update();
    }

    fn get_sort_order(self: &QSettingsModel) -> i32 {
        Self::with_settings_read(|settings| match settings.column_sort {
            ColumnSort::Inactive => 0,
            ColumnSort::Ascending(_) => 1,
            ColumnSort::Descending(_) => -1,
        })
    }

    fn get_sort_column(self: &QSettingsModel) -> i32 {
        Self::with_settings_read(|settings| match settings.column_sort {
            ColumnSort::Inactive => -1,
            ColumnSort::Ascending(col) => col.to_i32().unwrap_or(-1),
            ColumnSort::Descending(col) => col.to_i32().unwrap_or(-1),
        })
    }

    pub fn toggle_sort_column(self: Pin<&mut QSettingsModel>, column: i32) {
        let column = SongField::from_i32(column).unwrap();
        Self::with_settings_write(|settings| match settings.column_sort {
            ColumnSort::Inactive => {
                settings.column_sort = ColumnSort::Ascending(column);
            }
            ColumnSort::Ascending(old_column) => {
                if old_column != column {
                    settings.column_sort = ColumnSort::Ascending(column);
                } else {
                    settings.column_sort = ColumnSort::Descending(column);
                }
            }
            ColumnSort::Descending(_) => {
                settings.column_sort = ColumnSort::Ascending(column);
            }
        });
        self.update_sort_column();
    }

    pub fn mpd_binary_available(self: Pin<&mut QSettingsModel>) -> bool {
        let isettings = InternalSettings::load();
        isettings.mpd_binary.exists()
    }

    pub fn check_server_connection(
        self: Pin<&mut QSettingsModel>,
        url: QString,
    ) -> bool {
        tracing::debug!("Checking server connection for url: {}", url);
        UnixStream::connect(url.to_string()).is_ok()
            || TcpStream::connect(url.to_string()).is_ok()
    }
}

#[cxx_qt::bridge]
mod qobject {
    extern "C++" {
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;

        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(QString, mpdSocket, READ = get_mpd_socket, WRITE = set_mpd_socket, NOTIFY = mpd_server_settings_update)]
        #[qproperty(QString, nativeMpdMusicDir, READ = get_native_mpd_music_dir, WRITE = set_native_mpd_music_dir, NOTIFY = mpd_server_settings_update)]
        #[qproperty(QString, outputPluginType, READ = get_output_plugin_type, WRITE = set_output_plugin_type, NOTIFY = mpd_server_settings_update)]
        #[qproperty(usize, backgroundBlur, READ = get_background_blur, WRITE = set_background_blur, NOTIFY = mpd_appearance_settings_update)]
        #[qproperty(usize, backgroundOpacity, READ = get_background_opacity, WRITE = set_background_opacity, NOTIFY = mpd_appearance_settings_update)]
        #[qproperty(bool, initWizard, READ = get_init_wizard, WRITE = set_init_wizard, NOTIFY = init_wizard_changed)]
        #[qproperty(i32, sortOrder, READ = get_sort_order, NOTIFY = update_sort_column)]
        #[qproperty(i32, sortColumn, READ = get_sort_column, NOTIFY = update_sort_column)]
        type QSettingsModel = super::SettingsModel;

        #[qsignal]
        #[cxx_name = "mpdServerSettingsUpdate"]
        fn mpd_server_settings_update(self: Pin<&mut QSettingsModel>);

        #[qsignal]
        fn mpd_appearance_settings_update(self: Pin<&mut QSettingsModel>);

        #[qsignal]
        fn native_mpd_music_dir_changed(self: Pin<&mut QSettingsModel>);

        #[qsignal]
        fn init_wizard_changed(self: Pin<&mut QSettingsModel>);

        #[qsignal]
        #[cxx_name = "updateSortColumn"]
        fn update_sort_column(self: Pin<&mut QSettingsModel>);

        #[qinvokable]
        fn get_mpd_socket(self: Pin<&mut QSettingsModel>) -> QString;

        #[qinvokable]
        fn set_mpd_socket(self: Pin<&mut QSettingsModel>, value: QString);

        #[qinvokable]
        #[cxx_name = "getNativeMpdSocket"]
        fn get_native_mpd_socket(self: Pin<&mut QSettingsModel>) -> QString;

        #[qinvokable]
        fn get_native_mpd_music_dir(self: Pin<&mut QSettingsModel>) -> QString;

        #[qinvokable]
        fn set_native_mpd_music_dir(
            self: Pin<&mut QSettingsModel>,
            value: QString,
        );

        #[qinvokable]
        fn get_output_plugin_type(self: Pin<&mut QSettingsModel>) -> QString;

        #[qinvokable]
        fn set_output_plugin_type(
            self: Pin<&mut QSettingsModel>,
            value: QString,
        );

        #[qinvokable]
        fn get_init_wizard(self: Pin<&mut QSettingsModel>) -> bool;

        #[qinvokable]
        fn set_init_wizard(self: Pin<&mut QSettingsModel>, value: bool);

        #[qinvokable]
        #[cxx_name = "getColumnWidth"]
        fn get_column_width(
            self: Pin<&mut QSettingsModel>,
            column: usize,
        ) -> f64;

        #[qinvokable]
        #[cxx_name = "setColumnWidth"]
        fn set_column_width(
            self: Pin<&mut QSettingsModel>,
            column: usize,
            value: f64,
        );

        #[qinvokable]
        fn get_background_blur(self: Pin<&mut QSettingsModel>) -> usize;

        #[qinvokable]
        fn set_background_blur(self: Pin<&mut QSettingsModel>, value: usize);

        #[qinvokable]
        fn get_background_opacity(self: Pin<&mut QSettingsModel>) -> usize;

        #[qinvokable]
        fn set_background_opacity(self: Pin<&mut QSettingsModel>, value: usize);

        #[qinvokable]
        fn get_sort_order(self: &QSettingsModel) -> i32;

        #[qinvokable]
        fn get_sort_column(self: &QSettingsModel) -> i32;

        #[qinvokable]
        #[cxx_name = "toggleSortColumn"]
        fn toggle_sort_column(self: Pin<&mut QSettingsModel>, column: i32);

        #[qinvokable]
        #[cxx_name = "mpdBinaryAvailable"]
        fn mpd_binary_available(self: Pin<&mut QSettingsModel>) -> bool;

        #[qinvokable]
        #[cxx_name = "checkServerConnection"]
        fn check_server_connection(
            self: Pin<&mut QSettingsModel>,
            url: QString,
        ) -> bool;
    }
}
