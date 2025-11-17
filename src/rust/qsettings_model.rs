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
        #[qproperty(QString, mpdSocket, READ = get_mpd_socket, WRITE = set_mpd_socket, NOTIFY = mpd_socket_changed)]
        #[qproperty(QString, nativeMpdMusicDir, READ = get_native_mpd_music_dir, WRITE = set_native_mpd_music_dir, NOTIFY = native_mpd_music_dir_changed)]
        #[qproperty(bool, initWizard, READ = get_init_wizard, WRITE = set_init_wizard, NOTIFY = init_wizard_changed)]
        #[qproperty(i32, sortOrder, READ = get_sort_order, NOTIFY = update_sort_column)]
        #[qproperty(i32, sortColumn, READ = get_sort_column, NOTIFY = update_sort_column)]
        type QSettingsModel = super::SettingsModel;

        #[qsignal]
        fn mpd_socket_changed(self: Pin<&mut QSettingsModel>);

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
        fn set_native_mpd_music_dir(self: Pin<&mut QSettingsModel>, value: QString);

        #[qinvokable]
        fn get_init_wizard(self: Pin<&mut QSettingsModel>) -> bool;

        #[qinvokable]
        fn set_init_wizard(self: Pin<&mut QSettingsModel>, value: bool);

        #[qinvokable]
        #[cxx_name = "getColumnWidth"]
        fn get_column_width(self: Pin<&mut QSettingsModel>, column: usize) -> f64;

        #[qinvokable]
        #[cxx_name = "setColumnWidth"]
        fn set_column_width(self: Pin<&mut QSettingsModel>, column: usize, value: f64);

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
        fn check_server_connection(self: Pin<&mut QSettingsModel>, url: QString) -> bool;
    }
}

use qobject::*;

use crate::rust::entities::{ColumnSort, SongField};
use crate::rust::settings::{InternalSettings, Settings};
use core::pin::Pin;
use num_traits::{FromPrimitive, ToPrimitive};
use std::os::unix::net::UnixStream;

#[derive(Default)]
pub struct SettingsModel {}

impl qobject::QSettingsModel {
    pub fn get_mpd_socket(self: Pin<&mut QSettingsModel>) -> QString {
        let settings = Settings::load().blocking_read();
        QString::from(&settings.mpd_socket)
    }

    pub fn set_mpd_socket(self: Pin<&mut QSettingsModel>, value: QString) {
        let mut settings = Settings::load().blocking_write();
        settings.mpd_socket = value.into();
        std::mem::drop(settings);
        self.mpd_socket_changed();
    }

    pub fn get_native_mpd_socket(self: Pin<&mut QSettingsModel>) -> QString {
        let isettings = InternalSettings::load();
        QString::from(&isettings.native_socket)
    }

    fn get_native_mpd_music_dir(self: Pin<&mut QSettingsModel>) -> QString {
        let settings = Settings::load().blocking_read();
        QString::from(&settings.native_music_dir)
    }

    fn set_native_mpd_music_dir(self: Pin<&mut QSettingsModel>, value: QString) {
        let mut settings = Settings::load().blocking_write();
        settings.native_music_dir = value.into();
    }

    fn get_init_wizard(self: Pin<&mut QSettingsModel>) -> bool {
        let settings = Settings::load().blocking_read();
        settings.init_wizard
    }

    fn set_init_wizard(self: Pin<&mut QSettingsModel>, value: bool) {
        let mut settings = Settings::load().blocking_write();
        settings.init_wizard = value;
    }

    pub fn get_column_width(self: Pin<&mut QSettingsModel>, column: usize) -> f64 {
        let settings = Settings::load().blocking_read();
        *settings.column_width.get(column).unwrap()
    }

    pub fn set_column_width(self: Pin<&mut QSettingsModel>, column: usize, value: f64) {
        let mut settings = Settings::load().blocking_write();
        settings.column_width[column] = value;
    }

    fn get_sort_order(self: &QSettingsModel) -> i32 {
        let settings = Settings::load().blocking_read();
        match settings.column_sort {
            ColumnSort::Inactive => 0,
            ColumnSort::Ascending(_) => 1,
            ColumnSort::Descending(_) => -1,
        }
    }

    fn get_sort_column(self: &QSettingsModel) -> i32 {
        let settings = Settings::load().blocking_read();
        match settings.column_sort {
            ColumnSort::Inactive => -1,
            ColumnSort::Ascending(col) => col.to_i32().unwrap_or(-1),
            ColumnSort::Descending(col) => col.to_i32().unwrap_or(-1),
        }
    }

    pub fn toggle_sort_column(self: Pin<&mut QSettingsModel>, column: i32) {
        let column = SongField::from_i32(column).unwrap();
        let mut settings = Settings::load().blocking_write();
        match settings.column_sort {
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
        };
        std::mem::drop(settings);
        self.update_sort_column();
    }

    pub fn mpd_binary_available(self: Pin<&mut QSettingsModel>) -> bool {
        let isettings = InternalSettings::load();
        isettings.mpd_binary.exists()
    }

    pub fn check_server_connection(self: Pin<&mut QSettingsModel>, url: QString) -> bool {
        match UnixStream::connect(url.to_string()) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}
