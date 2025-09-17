/// The bridge definition for our QObject
#[cxx_qt::bridge]
pub mod qobject {
    extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("cxx-qt-lib/qbytearray.h");
        type QByteArray = cxx_qt_lib::QByteArray;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        type QMPDConnector = super::MPDConnector;

        #[qsignal]
        #[cxx_name = "connectionUpdate"]
        fn connection_update(self: Pin<&mut QMPDConnector>, status: QString);

        #[qsignal]
        #[cxx_name = "playStateChanged"]
        fn play_state_changed(self: Pin<&mut QMPDConnector>, status: QString);

        #[qsignal]
        #[cxx_name = "activeSongChanged"]
        fn active_song_changed(self: Pin<&mut QMPDConnector>, song_id: u64);

        #[qsignal]
        #[cxx_name = "timelineUpdate"]
        fn timeline_update(self: Pin<&mut QMPDConnector>, duration: u64, elapsed: u64);

        #[qsignal]
        #[cxx_name = "getPlaylistsResult"]
        fn get_playlists_result(self: Pin<&mut QMPDConnector>, result: QByteArray);

        #[qsignal]
        #[cxx_name = "stagePlaylistResult"]
        fn stage_playlist_result(self: Pin<&mut QMPDConnector>, result: QByteArray);

        #[qsignal]
        #[cxx_name = "dbUpdated"]
        fn db_updated(self: Pin<&mut QMPDConnector>, status: bool);

        #[qinvokable]
        #[cxx_name = "connect"]
        fn connect(self: Pin<&mut QMPDConnector>);

        #[qinvokable]
        #[cxx_name = "playSong"]
        fn play_song(self: Pin<&mut QMPDConnector>, id: u64);

        #[qinvokable]
        #[cxx_name = "playToggle"]
        fn play_toggle(self: Pin<&mut QMPDConnector>);

        #[qinvokable]
        #[cxx_name = "playNext"]
        fn play_next(self: Pin<&mut QMPDConnector>);

        #[qinvokable]
        #[cxx_name = "playPrevious"]
        fn play_previous(self: Pin<&mut QMPDConnector>);

        #[qinvokable]
        #[cxx_name = "updateDb"]
        fn update_db(self: Pin<&mut QMPDConnector>);

        #[qinvokable]
        #[cxx_name = "getPlaylists"]
        fn get_playlists(self: Pin<&mut QMPDConnector>, value: i32);

        #[qinvokable]
        #[cxx_name = "stagePlaylist"]
        fn stage_playlist(self: Pin<&mut QMPDConnector>, name: QString, group: i32);
    }

    impl cxx_qt::Threading for QMPDConnector {}
}

use qobject::*;

use crate::rust::entities::{QSong, SongField};
use crate::rust::settings::{InternalSettings, Settings};
use bincode::config;
use bincode::serde::encode_to_vec;
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use log;
use mpd_client::client::{ConnectionEvent, ConnectionEvents, Subsystem};
use mpd_client::{
    Client, commands, filter::Filter, responses::PlayState, responses::Song, tag::Tag,
};
use num_traits::FromPrimitive;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use tokio::net::TcpStream;
use tokio::time::{Duration, sleep};
use which::which;

#[derive(Default)]
pub struct MPDConnector {
    pub client: Option<Client>,
    pub idle: Option<ConnectionEvents>,
    pub server: Option<Child>,
}

impl qobject::QMPDConnector {
    fn idle(mut self: Pin<&mut QMPDConnector>) {
        let mut mpd_idle = self.as_mut().rust_mut().idle.take();
        let mpd_client = self.client.clone().unwrap();
        let qt_thread = self.qt_thread();
        tokio::task::spawn_blocking(move || {
            tokio::spawn(async move {
                loop {
                    match mpd_idle.as_mut().unwrap().next().await {
                        Some(ConnectionEvent::SubsystemChange(Subsystem::Database)) => {
                            let _ = qt_thread.queue(|mut qobject| {
                                qobject.as_mut().db_updated(true);
                            });
                        }
                        Some(ConnectionEvent::SubsystemChange(Subsystem::Queue)) => {
                            log::debug!("Server queue changed");
                            // Propagate playlist to other components
                            let command = commands::Queue::all();
                            let result = mpd_client.command(command).await.unwrap();
                            let result: Vec<QSong> = result.into_iter().map(QSong::from).collect();
                            let bcode: &[u8] = &encode_to_vec(result, config::standard()).unwrap();
                            let bcode = QByteArray::from(bcode);
                            let _ = qt_thread.queue(|mut qobject| {
                                qobject.as_mut().stage_playlist_result(bcode);
                            });
                            tokio::task::yield_now().await;
                        }
                        Some(ConnectionEvent::SubsystemChange(Subsystem::Player)) => {
                            log::debug!("Server player changed");
                            let command = commands::Status;
                            let result = mpd_client.command(command).await.unwrap();
                            let play_state = QString::from(format!("{:#?}", result.state));
                            let song_id = result.current_song.unwrap().1.0;
                            let _ = qt_thread.queue(move |mut qobject| {
                                qobject.as_mut().play_state_changed(play_state);
                                qobject.as_mut().active_song_changed(song_id);
                            });
                            tokio::task::yield_now().await;
                        }
                        Some(e) => println!("Yay {:?}", e),
                        None => {
                            log::warn!("Connection lost");
                            let _ = qt_thread.queue(|mut qobject| {
                                qobject.as_mut().connection_update("disconnected".into());
                                qobject.as_mut().idle();
                            });
                            sleep(Duration::from_millis(500)).await;
                        }
                    }
                }
            });
        });
    }

    fn init_ui(self: Pin<&mut Self>) {
        let mpd_client = self.client.clone();
        let qt_thread = self.qt_thread();
        tokio::spawn(async move {
            let command = commands::Status;
            let result = mpd_client.unwrap().command(command).await.unwrap();
            let play_state = QString::from(format!("{:#?}", result.state));
            let _ = qt_thread.queue(|mut qobject| {
                qobject.as_mut().play_state_changed(play_state);
                qobject.as_mut().start_timeline();
            });
        });
    }

    fn start_timeline(self: Pin<&mut Self>) {
        let mpd_client = self.client.clone().unwrap();
        let qt_thread = self.qt_thread();
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        tokio::task::spawn_blocking(move || {
            tokio::spawn(async move {
                loop {
                    let command = commands::Status;
                    let status = mpd_client.command(command).await.unwrap();
                    let duration = status.duration.unwrap_or(Duration::new(0, 0));
                    let elapsed = status.elapsed.unwrap_or(Duration::new(0, 0));
                    let _ = qt_thread.queue(move |mut qobject| {
                        qobject
                            .as_mut()
                            .timeline_update(duration.as_secs(), elapsed.as_secs());
                    });
                    interval.tick().await;
                }
            });
        });
    }

    pub fn get_playlists(self: Pin<&mut QMPDConnector>, group: i32) {
        let group = Tag::from(SongField::from_i32(group).unwrap());
        let mpd_client = self.client.clone().unwrap();
        let qt_thread = self.qt_thread();
        tokio::spawn(async move {
            let mut result: Vec<String> = match group {
                Tag::Other(value) if value == "Directory".into() => {
                    let command = commands::ListAllIn::root();
                    let res = mpd_client.command(command).await.unwrap();
                    res.iter()
                        .map(|x| {
                            x.file_path()
                                .parent()
                                .unwrap_or(Path::new("Root"))
                                .to_str()
                                .unwrap()
                                .to_string()
                        })
                        .collect::<HashSet<_>>()
                        .into_iter()
                        .collect()
                }
                _ => {
                    let command = commands::List::new(group);
                    mpd_client
                        .command(command)
                        .await
                        .unwrap()
                        .values()
                        .map(|x| x.to_string())
                        .collect()
                }
            };
            result.sort();
            let bcode: &[u8] = &encode_to_vec(result, config::standard()).unwrap();
            let bcode = QByteArray::from(bcode);
            let _ = qt_thread.queue(|mut qobject| {
                qobject.as_mut().get_playlists_result(bcode);
            });
        });
    }

    pub fn stage_playlist(self: Pin<&mut QMPDConnector>, name: QString, group: i32) {
        let tag = Tag::from(SongField::from_i32(group).unwrap());
        let name = String::from(name);
        let mpd_client = self.client.clone().unwrap();
        tokio::spawn(async move {
            // Query playlist
            let result: Vec<Song> = match tag {
                Tag::Other(value) if value == "Directory".into() => {
                    let command = commands::ListAllIn::directory(&name);
                    mpd_client.command(command).await.unwrap_or(Vec::default())
                }
                _ => {
                    let filter = Filter::tag(tag, name);
                    let command = commands::Find::new(filter);
                    mpd_client.command(command).await.unwrap_or(Vec::default())
                }
            };
            // Clear current queue
            let clear_command = commands::ClearQueue;
            let _ = mpd_client.command(clear_command).await;
            // Populate new queue
            let add_commands: Vec<commands::Add> = result
                .iter()
                .map(|x| commands::Add::uri(x.url.as_str()))
                .collect();
            let _ = mpd_client.command_list(add_commands).await.unwrap();
        });
    }

    pub fn update_db(self: Pin<&mut QMPDConnector>) {
        log::debug!("Updating MPD DB");
        let mpd_client = self.client.clone().unwrap();
        let qt_thread = self.qt_thread();
        tokio::spawn(async move {
            let command = commands::Update::new();
            let _ = mpd_client.command(command).await;
            let _ = qt_thread.queue(|mut qobject| {
                qobject.as_mut().db_updated(false);
            });
        });
    }

    fn start_native_server(self: Pin<&mut Self>, mpd_binary: &PathBuf, _native_config: &String) {
        log::debug!("Starting native mpd server");
        let qt_thread = self.qt_thread();
        let cmpd_binary = mpd_binary.clone();
        //let cnative_config = native_config.clone();
        std::thread::spawn(move || {
            let mut command = Command::new(cmpd_binary);
            command.arg("--no-daemon");
            //command.arg(cnative_config);
            let handle = command.spawn().expect("Failed to start mpd server");
            let _ = qt_thread.queue(move |mut qobject| {
                qobject.as_mut().rust_mut().server.replace(handle);
            });
        });
    }

    fn connect_client(self: Pin<&mut Self>) {
        let qt_thread = self.qt_thread();
        let mut retcount = 5;
        tokio::spawn(async move {
            let (state, mpd_client, mpd_idle) = loop {
                let settings = Settings::load();
                let mut mpd_client: Option<Client> = Option::None;
                let mut mpd_idle: Option<ConnectionEvents> = Option::None;
                match TcpStream::connect(&settings.mpd_socket).await {
                    Ok(connection) => {
                        let mpd_connection = Client::connect(connection).await.unwrap();
                        mpd_client.replace(mpd_connection.0);
                        mpd_idle.replace(mpd_connection.1);
                        log::debug!("Succesfully connected to MPD server");
                        break ("connected", mpd_client, mpd_idle);
                    }
                    Err(e) => {
                        retcount -= 1;
                        if retcount <= 0 {
                            log::error!("Failed to connect to MPD server");
                            break ("disconnected", Option::None, Option::None);
                        }
                        log::warn!("Failed to connect: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                    }
                };
            };
            let _ = qt_thread.queue(|mut qobject| {
                qobject
                    .as_mut()
                    .rust_mut()
                    .client
                    .replace(mpd_client.unwrap());
                qobject.as_mut().rust_mut().idle.replace(mpd_idle.unwrap());
                qobject.as_mut().connection_update(state.into());
                qobject.as_mut().idle();
                qobject.as_mut().init_ui();
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
                Err(err) => panic!(
                    "Using native socket, but no mpd binary was found, {:?}",
                    err
                ),
            };
        }
        self.connect_client();
    }

    pub fn play_toggle(self: Pin<&mut Self>) {
        let mpd_client = self.client.clone().unwrap();
        tokio::spawn(async move {
            let command = commands::Status;
            let result = mpd_client.command(command).await.unwrap();
            match result.state {
                PlayState::Paused => {
                    let command = commands::SetPause(false);
                    mpd_client.command(command).await.unwrap();
                }
                PlayState::Playing => {
                    let command = commands::SetPause(true);
                    mpd_client.command(command).await.unwrap();
                }
                PlayState::Stopped => {
                    let command = commands::Play::current();
                    mpd_client.command(command).await.unwrap();
                }
            }
        });
    }

    pub fn play_song(self: Pin<&mut Self>, id: u64) {
        let mpd_client = self.client.clone().unwrap();
        tokio::spawn(async move {
            let command = commands::Play::song(commands::SongId::from(id));
            mpd_client.command(command).await.unwrap();
        });
    }

    pub fn play_next(self: Pin<&mut Self>) {
        let mpd_client = self.client.clone().unwrap();
        tokio::spawn(async move {
            let command = commands::Next;
            let _ = mpd_client.command(command).await;
        });
    }

    pub fn play_previous(self: Pin<&mut Self>) {
        let mpd_client = self.client.clone().unwrap();
        tokio::spawn(async move {
            let command = commands::Previous;
            let _ = mpd_client.command(command).await;
        });
    }
}

impl Drop for MPDConnector {
    fn drop(&mut self) {
        if let Some(server) = self.server.as_mut() {
            server.kill().unwrap()
        }
    }
}
