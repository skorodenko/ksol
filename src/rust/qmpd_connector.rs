/// The bridge definition for our QObject
#[cxx_qt::bridge]
pub mod qobject {
    extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        type QMPDConnector = super::MPDConnector;

        #[qsignal]
        #[cxx_name = "connectionUpdate"]
        fn connection_update(self: Pin<&mut QMPDConnector>, status: QString);

        #[qinvokable]
        #[cxx_name = "connect"]
        fn connect(self: Pin<&mut QMPDConnector>);
    }
}

use qobject::*;

use core::pin::Pin;
use log::debug;
use std::path::PathBuf;
use subprocess::Popen;
use which::which;

/// The Rust struct for the QObject
#[derive(Default)]
pub struct MPDConnector {
    mpd_server: Option<Popen>,
}

impl qobject::QMPDConnector {
    pub fn connect(self: Pin<&mut Self>) {
        debug!("Connecting to mpd");
        self.connection_update(QString::from("connecting"));
        let mpd_binary: PathBuf = match which("mpd") {
            Ok(v) => v,
            Err(_) => PathBuf::from(""),
        };
    }
}
