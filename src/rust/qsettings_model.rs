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
        #[qproperty(bool, initWizard, READ = get_init_wizard, WRITE = set_init_wizard, NOTIFY = init_wizard_changed)]
        type QSettingsModel = super::SettingsModel;

        #[qsignal]
        fn mpd_socket_changed(self: Pin<&mut QSettingsModel>);

        #[qsignal]
        fn init_wizard_changed(self: Pin<&mut QSettingsModel>);

        #[qsignal]
        #[cxx_name = "checkServerConnectionResult"]
        fn check_server_connection_result(self: Pin<&mut QSettingsModel>, result: bool);

        #[qinvokable]
        fn get_mpd_socket(self: Pin<&mut QSettingsModel>) -> QString;

        #[qinvokable]
        fn set_mpd_socket(self: Pin<&mut QSettingsModel>, value: QString);

        #[qinvokable]
        fn get_init_wizard(self: Pin<&mut QSettingsModel>) -> bool;

        #[qinvokable]
        fn set_init_wizard(self: Pin<&mut QSettingsModel>, value: bool);

        #[qinvokable]
        #[cxx_name = "mpdBinaryAvailable"]
        fn mpd_binary_available(self: Pin<&mut QSettingsModel>) -> bool;

        #[qinvokable]
        #[cxx_name = "checkServerConnection"]
        fn check_server_connection(self: Pin<&mut QSettingsModel>, url: QString);
    }
}

use qobject::*;

use crate::rust::settings::{InternalSettings, Settings};
use core::pin::Pin;
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
        self.mpd_socket_changed();
    }

    pub fn mpd_binary_available(self: Pin<&mut QSettingsModel>) -> bool {
        let isettings = InternalSettings::load();
        isettings.mpd_binary.exists()
    }

    fn get_init_wizard(self: Pin<&mut QSettingsModel>) -> bool {
        let settings = Settings::load().blocking_read();
        settings.init_wizard
    }

    fn set_init_wizard(self: Pin<&mut QSettingsModel>, value: bool) {
        let mut settings = Settings::load().blocking_write();
        settings.init_wizard = value;
    }

    pub fn check_server_connection(self: Pin<&mut QSettingsModel>, url: QString) {
        println!("{}", url);
        match UnixStream::connect(url.to_string()) {
            Ok(_) => self.check_server_connection_result(true),
            Err(e) => {
                println!("{}", e);
                self.check_server_connection_result(false)
            },
        }
    }
}
