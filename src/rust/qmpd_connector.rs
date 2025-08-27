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

        #[qinvokable]
        #[cxx_name = "updateDb"]
        fn update_db(self: Pin<&mut QMPDConnector>);
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
use tokio::time::{Duration, sleep, timeout};
use which::which;

#[derive(Default)]
pub struct MPDConnector {
    pub mpd_client: Arc<Mutex<MpdClient>>,
}

impl qobject::QMPDConnector {
    fn idle(self: Pin<&mut Self>) {
        let mutex = self.mpd_client.clone();
        let qt_thread = self.qt_thread();
        tokio::spawn(async move {
            loop {
                let mut mpd_client = mutex.lock().await;
                let idle = timeout(Duration::from_millis(100), mpd_client.idle()).await;
                match idle {
                    Ok(Ok(_)) => println!("Task finished within timeout."),
                    Ok(Err(e)) => println!("Task failed: {:?}", e),
                    Err(_) => {} 
                }
                //log::debug!("{:?}", idle);
            }
        });
    }

    pub fn update_db(self: Pin<&mut QMPDConnector>) {
        log::debug!("Updating MPD DB");
        let mutex = self.mpd_client.clone();
        tokio::spawn(async move {
            let mut mpd_client = mutex.lock().await;
            let res = mpd_client.update("".into()).await;
            log::debug!("{:?}", res);
        });
    }

    fn start_native_server(self: Pin<&mut Self>, mpd_binary: &PathBuf, native_config: &String) {
        log::debug!("Starting native mpd server");
        let cmpd_binary = mpd_binary.clone();
        //let cnative_config = native_config.clone();
        tokio::spawn(async move {
            let mut command = Command::new(cmpd_binary);
            command.arg("--no-daemon");
            //command.arg(cnative_config);
            command.kill_on_drop(true);
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
            let state = loop {
                let settings = Settings::load();
                let mut mpd_client = mutex.lock().await;
                match mpd_client.connect(&settings.mpd_socket).await {
                    Ok(_) => {
                        log::debug!("Succesfully connected to MPD server");
                        break "connected";
                    }
                    Err(e) => {
                        retcount = retcount - 1;
                        if retcount <= 0 {
                            log::error!("Failed to connect to MPD server");
                            break "disconnected";
                        }
                        log::warn!("Failed to connect: {}", e.to_string());
                        tokio::time::sleep(tokio::time::Duration::from_secs_f32(timeout)).await;
                    }
                };
            };
            let _ = qt_thread.queue(|mut qobject| {
                let _ = qobject.as_mut().connection_update(state.into());
                qobject.as_mut().idle();
            });
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
