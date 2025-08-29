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

        #[qsignal]
        #[cxx_name = "dbUpdated"]
        fn db_updated(self: Pin<&mut QMPDConnector>, status: bool);

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

use core::pin::Pin;
use cxx_qt::Threading;
use log;
use mpd_client::client::{ConnectionEvent, ConnectionEvents, Subsystem};
use mpd_client::{Client, commands::Update};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::process::Command;
use tokio::sync::RwLock;
use tokio::time::{Duration, sleep};
use which::which;

#[derive(Default)]
pub struct MPDConnector {
    pub client: Arc<RwLock<Option<Client>>>,
    pub idle: Arc<RwLock<Option<ConnectionEvents>>>,
}

impl qobject::QMPDConnector {
    fn idle(self: Pin<&mut Self>) {
        let mpd_idle = self.idle.clone();
        let qt_thread = self.qt_thread();
        tokio::spawn(async move {
            loop {
                let mut mpd_idle = mpd_idle.write().await;
                match mpd_idle.as_mut().unwrap().next().await {
                    Some(ConnectionEvent::SubsystemChange(Subsystem::Update)) => {
                        let _ = qt_thread.queue(|mut qobject| {
                            let _ = qobject.as_mut().db_updated(true);
                        });
                    }
                    Some(e) => println!("Yay {:?}", e),
                    None => {
                        log::warn!("Connection lost?");
                        sleep(Duration::from_millis(300)).await;
                    }
                }
                sleep(Duration::from_millis(50)).await;
            }
        });
    }

    pub fn update_db(self: Pin<&mut QMPDConnector>) {
        log::debug!("Updating MPD DB");
        let mpd_client = self.client.clone();
        tokio::spawn(async move {
            let mpd_client = mpd_client.read().await;
            let command = Update::new();
            let _ = mpd_client.as_ref().unwrap().command(command).await;
        });
    }

    fn start_native_server(self: Pin<&mut Self>, mpd_binary: &PathBuf, _native_config: &String) {
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

    fn connect_client(self: Pin<&mut Self>) {
        let mpd_client = self.client.clone();
        let mpd_idle = self.idle.clone();
        let qt_thread = self.qt_thread();
        let mut retcount = 5;
        tokio::spawn(async move {
            let state = loop {
                let settings = Settings::load();
                let mut mpd_client = mpd_client.write().await;
                let mut mpd_idle = mpd_idle.write().await;
                match TcpStream::connect(&settings.mpd_socket).await {
                    Ok(connection) => {
                        let mpd_connection = Client::connect(connection).await.unwrap();
                        mpd_client.replace(mpd_connection.0);
                        mpd_idle.replace(mpd_connection.1);
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
                        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
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
        self.connect_client();
    }
}
