/// The bridge definition for our QObject
#[cxx_qt::bridge]
pub mod qobject {

    extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        /// An alias to the QString type
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        // The QObject definition
        // We tell CXX-Qt that we want a QObject class with the name MyObject
        // based on the Rust struct MyObjectRust.
        #[qobject]
        #[qml_element]
        type QMPDConnector = super::MPDConnector;
    }

    extern "RustQt" {
        // Declare the invokable methods we want to expose on the QObject
        #[qinvokable]
        #[cxx_name = "connect"]
        fn connect(self: Pin<&mut QMPDConnector>);
    }
}

use log::debug;
use core::pin::Pin;
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
        debug!("Calling 'connect'");
        let mpd_binary: PathBuf = match which("mpd") {
            Ok(v) => v,
            Err(_) => PathBuf::from(""),
        };
        println!("{:?}", mpd_binary.into_os_string().into_string());
    }
}
