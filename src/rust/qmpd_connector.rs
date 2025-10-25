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
        #[cxx_name = "syncQueue"]
        fn sync_queue(self: Pin<&mut QMPDConnector>);

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

        #[qinvokable]
        #[cxx_name = "sortPlaylist"]
        fn sort_playlist(self: Pin<&mut QMPDConnector>, sort_column: i32, sort_order: i32);
    }

    impl cxx_qt::Threading for QMPDConnector {}
}

use qobject::*;

use crate::rust::entities::{ColumnSort, MPSCCommand, QSong, SongField};
use crate::rust::settings::{InternalSettings, Settings};
use bincode::config;
use bincode::serde::encode_to_vec;
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use log;
use mpd_client::client::{ConnectionEvent, Subsystem};
use mpd_client::{
    ClientController, ClientIdler, commands, filter::Filter, responses::PlayState, responses::Song,
    tag::Tag,
};
use num_traits::FromPrimitive;
use std::cmp::Reverse;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use tokio::net::{TcpStream, UnixStream};
use tokio::runtime::{Builder, Runtime};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::{Duration, sleep};
use which::which;

pub struct MPDConnector {
    pub client: Option<ClientController>,
    pub idle_client: Option<ClientIdler>,
    pub server: Option<Child>,
    pub rt_idle: Runtime,
    pub rt_timeline: Runtime,
    pub rt_action: Runtime,
    pub tx_actions: Sender<MPSCCommand>,
    pub rx_actions: Option<Receiver<MPSCCommand>>,
}

impl qobject::QMPDConnector {
    fn idle(mut self: Pin<&mut QMPDConnector>) {
        let mut mpd_idle = self.as_mut().rust_mut().idle_client.take();
        let qt_thread = self.qt_thread();
        let tx_actions = self.tx_actions.clone();
        let rt_idle = &self.rt_idle;
        rt_idle.spawn(async move {
            loop {
                match mpd_idle.as_mut().unwrap().next().await {
                    Some(ConnectionEvent::SubsystemChange(Subsystem::Database)) => {
                        let _ = qt_thread.queue(|mut qobject| {
                            qobject.as_mut().db_updated(true);
                        });
                    }
                    Some(ConnectionEvent::SubsystemChange(Subsystem::Queue)) => {
                        log::debug!("Server queue changed");
                        let _ = tx_actions.send(MPSCCommand::IdleQueue).await;
                    }
                    Some(ConnectionEvent::SubsystemChange(Subsystem::Player)) => {
                        log::debug!("Server player changed");
                        let _ = tx_actions.send(MPSCCommand::IdlePlayer).await;
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
    }

    fn init_ui(self: Pin<&mut Self>) {
        let mpd_client = self.client.clone();
        let qt_thread = self.qt_thread();
        let rt_action = &self.rt_action;
        rt_action.spawn(async move {
            let command = commands::Status;
            let result = mpd_client.unwrap().command(command).await.unwrap();
            let play_state = QString::from(format!("{:#?}", result.state));
            let _ = qt_thread.queue(|mut qobject| {
                qobject.as_mut().init_actions();
                qobject.as_mut().play_state_changed(play_state);
                qobject.as_mut().init_timeline();
            });
        });
    }

    fn init_timeline(self: Pin<&mut Self>) {
        let mpd_client = self.client.clone().unwrap();
        let qt_thread = self.qt_thread();
        let rt_timeline = &self.rt_timeline;
        rt_timeline.spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
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
    }

    fn init_actions(mut self: Pin<&mut Self>) {
        let mut rx_actions = self.as_mut().rust_mut().rx_actions.take();
        let mpd_client = self.client.clone().unwrap();
        let qt_thread = self.qt_thread();
        let rt_action = &self.rt_action;
        rt_action.spawn(async move {
            loop {
                match rx_actions.as_mut().unwrap().recv().await {
                    Some(MPSCCommand::Next) => {
                        let command = commands::Next;
                        let _ = mpd_client.command(command).await;
                    }
                    Some(MPSCCommand::Previous) => {
                        let command = commands::Previous;
                        let _ = mpd_client.command(command).await;
                    }
                    Some(MPSCCommand::PlaySong(id)) => {
                        let command = commands::Play::song(commands::SongId::from(id));
                        let _ = mpd_client.command(command).await;
                    }
                    Some(MPSCCommand::PlayToggle) => {
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
                    }
                    Some(MPSCCommand::UpdateDb) => {
                        let command = commands::Update::new();
                        let _ = mpd_client.command(command).await;
                        let _ = qt_thread.queue(|mut qobject| {
                            qobject.as_mut().db_updated(false);
                        });
                    }
                    Some(MPSCCommand::GetPlaylists(group)) => {
                        let group = Tag::from(group);
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
                    }
                    Some(MPSCCommand::SortPlaylist(sort_order)) => {
                        let command = commands::Queue::all();
                        let songs = mpd_client.command(command).await.unwrap();
                        let mut songs: Vec<QSong> = songs.into_iter().map(QSong::from).collect();
                        match sort_order {
                            ColumnSort::Inactive => (),
                            ColumnSort::Ascending(col) => match col {
                                SongField::Track => songs.sort_by_key(|k| k.clone().track),
                                SongField::Title => songs.sort_by_key(|k| k.clone().title),
                                SongField::Artist => songs.sort_by_key(|k| k.clone().artist),
                                SongField::Album => songs.sort_by_key(|k| k.clone().album),
                                SongField::Date => songs.sort_by_key(|k| k.clone().date),
                                SongField::Genre => songs.sort_by_key(|k| k.clone().genre),
                                SongField::Disc => songs.sort_by_key(|k| k.clone().disc),
                                SongField::Composer => songs.sort_by_key(|k| k.clone().composer),
                                SongField::Albumartist => songs.sort_by_key(|k| k.clone().artist),
                                SongField::File => songs.sort_by_key(|k| k.clone().file),
                                SongField::Format => songs.sort_by_key(|k| k.clone().format),
                                SongField::Lastmodified => {
                                    songs.sort_by_key(|k| k.clone().lastmodified)
                                }
                                SongField::Duration => songs.sort_by_key(|k| k.clone().duration),
                                SongField::Directory => songs.sort_by_key(|k| k.clone().directory),
                            },
                            ColumnSort::Descending(col) => match col {
                                SongField::Track => songs.sort_by_key(|k| Reverse(k.clone().track)),
                                SongField::Title => songs.sort_by_key(|k| Reverse(k.clone().title)),
                                SongField::Artist => {
                                    songs.sort_by_key(|k| Reverse(k.clone().artist))
                                }
                                SongField::Album => songs.sort_by_key(|k| Reverse(k.clone().album)),
                                SongField::Date => songs.sort_by_key(|k| Reverse(k.clone().date)),
                                SongField::Genre => songs.sort_by_key(|k| Reverse(k.clone().genre)),
                                SongField::Disc => songs.sort_by_key(|k| Reverse(k.clone().disc)),
                                SongField::Composer => {
                                    songs.sort_by_key(|k| Reverse(k.clone().composer))
                                }
                                SongField::Albumartist => {
                                    songs.sort_by_key(|k| Reverse(k.clone().artist))
                                }
                                SongField::File => songs.sort_by_key(|k| Reverse(k.clone().file)),
                                SongField::Format => {
                                    songs.sort_by_key(|k| Reverse(k.clone().format))
                                }
                                SongField::Lastmodified => {
                                    songs.sort_by_key(|k| Reverse(k.clone().lastmodified))
                                }
                                SongField::Duration => {
                                    songs.sort_by_key(|k| Reverse(k.clone().duration))
                                }
                                SongField::Directory => {
                                    songs.sort_by_key(|k| Reverse(k.clone().directory))
                                }
                            },
                        };
                        let move_commands: Vec<commands::Move> = songs
                            .iter()
                            .enumerate()
                            .map(|(i, x)| commands::Move::id(x.id.into()).to_position(i.into()))
                            .collect();
                        let _ = mpd_client.command_list(move_commands).await.unwrap();
                    }
                    Some(MPSCCommand::StagePlaylist(name, group)) => {
                        let tag = Tag::from(group);
                        // Query playlist
                        let songs: Vec<Song> = match tag {
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
                        let add_commands: Vec<commands::Add> = songs
                            .iter()
                            .map(|x| commands::Add::uri(x.url.as_str()))
                            .collect();
                        let _ = mpd_client.command_list(add_commands).await.unwrap();
                    }
                    Some(MPSCCommand::IdlePlayer) => {
                        let command = commands::Status;
                        let result = mpd_client.command(command).await.unwrap();
                        let play_state = QString::from(format!("{:#?}", result.state));
                        let song_id = result.current_song.unwrap().1.0;
                        let _ = qt_thread.queue(move |mut qobject| {
                            qobject.as_mut().play_state_changed(play_state);
                            qobject.as_mut().active_song_changed(song_id);
                        });
                    }
                    Some(MPSCCommand::IdleQueue) => {
                        // Propagate playlist to other components
                        let command = commands::Queue::all();
                        let result = mpd_client.command(command).await.unwrap();
                        let result: Vec<QSong> = result.into_iter().map(QSong::from).collect();
                        let bcode: &[u8] = &encode_to_vec(result, config::standard()).unwrap();
                        let bcode = QByteArray::from(bcode);
                        let _ = qt_thread.queue(|mut qobject| {
                            qobject.as_mut().stage_playlist_result(bcode);
                        });
                    }
                    None => {}
                }
            }
        });
    }

    pub fn play_toggle(self: Pin<&mut Self>) {
        let tx_actions = self.tx_actions.clone();
        let _ = tx_actions.blocking_send(MPSCCommand::PlayToggle);
    }

    pub fn play_song(self: Pin<&mut Self>, id: u64) {
        let tx_actions = self.tx_actions.clone();
        let _ = tx_actions.blocking_send(MPSCCommand::PlaySong(id));
    }

    pub fn play_next(self: Pin<&mut Self>) {
        let tx_actions = self.tx_actions.clone();
        let _ = tx_actions.blocking_send(MPSCCommand::Next);
    }

    pub fn play_previous(self: Pin<&mut Self>) {
        let tx_actions = self.tx_actions.clone();
        let _ = tx_actions.blocking_send(MPSCCommand::Previous);
    }

    pub fn update_db(self: Pin<&mut QMPDConnector>) {
        log::debug!("Updating MPD DB");
        let tx_actions = self.tx_actions.clone();
        let _ = tx_actions.blocking_send(MPSCCommand::UpdateDb);
    }

    pub fn get_playlists(self: Pin<&mut QMPDConnector>, group: i32) {
        let group = SongField::from_i32(group).unwrap();
        let tx_actions = self.tx_actions.clone();
        let _ = tx_actions.blocking_send(MPSCCommand::GetPlaylists(group));
    }

    pub fn stage_playlist(self: Pin<&mut QMPDConnector>, name: QString, group: i32) {
        let name = String::from(name);
        let group = SongField::from_i32(group).unwrap();
        let tx_actions = self.tx_actions.clone();
        let _ = tx_actions.blocking_send(MPSCCommand::StagePlaylist(name, group));
    }

    pub fn sort_playlist(self: Pin<&mut QMPDConnector>, sort_column: i32, sort_order: i32) {
        let sort_column = SongField::from_i32(sort_column).unwrap();
        let sort_order = ColumnSort::from((sort_order, sort_column));
        let tx_actions = self.tx_actions.clone();
        let _ = tx_actions.blocking_send(MPSCCommand::SortPlaylist(sort_order));
    }

    pub fn sync_queue(self: Pin<&mut QMPDConnector>) {
        let tx_actions = self.tx_actions.clone();
        let _ = tx_actions.blocking_send(MPSCCommand::IdleQueue);
    }

    fn start_native_server(self: Pin<&mut Self>, mpd_binary: &PathBuf, native_config: &String) {
        log::debug!("Starting native mpd server");
        let qt_thread = self.qt_thread();
        let cmpd_binary = mpd_binary.clone();
        let cnative_config = native_config.clone();
        std::thread::spawn(move || {
            let mut command = Command::new(cmpd_binary);
            command.arg("--no-daemon");
            command.arg(cnative_config);
            let handle = command.spawn().expect("Failed to start mpd server");
            let _ = qt_thread.queue(move |mut qobject| {
                qobject.as_mut().rust_mut().server.replace(handle);
            });
        });
    }

    fn connect_client(self: Pin<&mut Self>) {
        let qt_thread = self.qt_thread();
        let mut retcount = 5;
        let rt_idle = &self.rt_idle;
        rt_idle.spawn(async move {
            let (state, mpd_client, mpd_idle) = loop {
                let settings = Settings::load();
                //match TcpStream::connect(&settings.mpd_socket).await {
                match UnixStream::connect(&settings.mpd_socket).await {
                    Ok(connection) => {
                        let client = ClientController::connect(connection).await.unwrap();
                        let idle = ClientIdler::connect(
                            UnixStream::connect(&settings.mpd_socket).await.unwrap(),
                        )
                        .await
                        .unwrap();
                        log::debug!("Succesfully connected to MPD server");
                        break ("connected", Some(client), Some(idle));
                    }
                    Err(e) => {
                        retcount -= 1;
                        if retcount <= 0 {
                            log::error!("Failed to connect to MPD server");
                            break ("disconnected", None, None);
                        }
                        log::warn!("Failed to connect: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                    }
                };
            };
            let _ = qt_thread.queue(|mut qobject| {
                qobject.as_mut().rust_mut().client = mpd_client;
                qobject.as_mut().rust_mut().idle_client = mpd_idle;
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
}

impl Default for MPDConnector {
    fn default() -> Self {
        let rt_idle = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_io()
            .enable_time()
            .build()
            .unwrap();
        let rt_timeline = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_time()
            .build()
            .unwrap();
        let rt_action = Builder::new_multi_thread()
            .worker_threads(1)
            .build()
            .unwrap();
        let (tx_actions, rx_actions) = tokio::sync::mpsc::channel(64);
        Self {
            client: None,
            idle_client: None,
            server: None,
            rt_idle,
            rt_timeline,
            rt_action,
            tx_actions,
            rx_actions: Some(rx_actions),
        }
    }
}

impl Drop for MPDConnector {
    fn drop(&mut self) {
        if let Some(server) = self.server.as_mut() {
            server.kill().unwrap()
        }
    }
}
