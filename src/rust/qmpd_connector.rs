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

    impl cxx_qt::Threading for QMPDConnector {}
}

use qobject::*;

use crate::rust::settings::{InternalSettings, Settings};

use async_mpd::MpdClient;
use core::pin::Pin;
use cxx_qt::Threading;
use log;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;
use which::which;

#[derive(Default)]
pub struct MPDConnector {
    pub mpd_client: Arc<Mutex<MpdClient>>,
}

impl qobject::QMPDConnector {
    fn start_native_server(self: Pin<&mut Self>, mpd_binary: &PathBuf, native_config: &String) {
        log::debug!("Starting native mpd server");
        let cmpd_binary = mpd_binary.clone();
        //let cnative_config = native_config.clone();
        tokio::spawn(async move {
            let mut command = Command::new(cmpd_binary);
            command.arg("--no-daemon");
            //command.arg(cnative_config);
            let _ = command
                .spawn()
                .expect("Failed to start mpd server")
                .wait()
                .await
                .expect("Failed to run mpd server");
        });
    }

    fn connect_client(self: Pin<&mut Self>, retcount: Option<i32>, timeout: Option<f32>) {
        let mutex = self.mpd_client.clone();
        let qt_thread = self.qt_thread();
        let mut retcount = retcount.unwrap_or(5);
        let timeout = timeout.unwrap_or(1.5);
        tokio::spawn(async move {
            loop {
                let settings = Settings::load();
                let mut mpd_client = mutex.lock().await;
                match mpd_client.connect(&settings.mpd_socket).await {
                    Ok(_) => {
                        let _ = qt_thread.queue(|qobject| {
                            let _ = qobject.connection_update("connected".into());
                        });
                        log::debug!("Succesfully connected to MPD server");
                        break;
                    }
                    Err(e) => {
                        retcount = retcount - 1;
                        if retcount <= 0 {
                            log::error!("Failed to connect to MPD server");
                            break;
                        }
                        log::warn!("Failed to connect: {}", e.to_string());
                        tokio::time::sleep(tokio::time::Duration::from_secs_f32(timeout)).await;
                    }
                };
            }
        });
    }

    pub fn connect(mut self: Pin<&mut Self>) {
        log::debug!("Connecting to mpd");
        let settings = Settings::load();
        let isettings = InternalSettings::load();
        self.as_mut().connection_update(QString::from("connecting"));
        if settings.mpd_socket == isettings.native_socket {
            log::debug!("Using native mpd server");
            match which("mpd") {
                Ok(v) => {
                    log::debug!("Found mpd binary {:?}", v);
                    self.as_mut()
                        .start_native_server(&v, &isettings.native_config);
                }
                Err(_) => panic!("Using native socket, but no mpd binary was found"),
            };
        }
        self.connect_client(None, None);
    }
}
