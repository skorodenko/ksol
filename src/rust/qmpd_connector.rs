use qobject::*;

use crate::rust::action_pool::ActionPool;
use crate::rust::entities::{ColumnSort, MPSCCommand, SongField};
use crate::rust::init_hooks::init_native_mpd_config;
use crate::rust::mpris_interface::Player;
use crate::rust::settings::{InternalSettings, Settings};
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use mpd_client::client::{ConnectionEvent, Subsystem};
use mpd_client::{ClientController, ClientIdler, commands};
use mpris_server::{LoopStatus, PlaybackStatus, Property, Server};
use num_traits::FromPrimitive;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::net::{TcpStream, UnixStream};
use tokio::process::Command;
use tokio::runtime::{Builder, Handle, Runtime};
use tokio::sync::Notify;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing;
use which::which;

pub struct MPDConnector {
    pub client: Option<ClientController>,
    pub idle_client: Option<ClientIdler>,
    pub cancel: Option<CancellationToken>,
    pub repeat: bool,
    pub single: bool,
    pub shuffle: bool,
    pub rt_idle: Runtime,
    pub rt_action: Runtime,
    pub tx_mpris: Option<Sender<Vec<Property>>>,
    pub rx_mpris: Option<Receiver<Vec<Property>>>,
    pub tx_actions: Option<Sender<MPSCCommand>>,
    pub rx_actions: Option<Receiver<MPSCCommand>>,
}

impl qobject::QMPDConnector {
    fn idle(mut self: Pin<&mut QMPDConnector>) {
        let mut mpd_idle = self.as_mut().rust_mut().idle_client.take();
        let qt_thread = self.qt_thread();
        let cancel_token = self.cancel.clone().expect("Cancel token is None");
        let tx_actions = self.tx_actions.clone();
        let mut rx_mpris = self.as_mut().rust_mut().rx_mpris.take().expect("mpris receiver is None");
        let rt_idle = &self.rt_idle;
        tracing::debug!("Starting idle");
        rt_idle.spawn(async move {
            let notify = Arc::new(Notify::new());
            let notify_player = notify.clone();
            let notify_queue = notify.clone();
            let mpd_idle = mpd_idle.as_mut().expect("idle client is None");
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            let mpris_server =
                Server::new("ksol", Player { action_sender: tx_actions.clone().expect("Actions sender is None") }).await.unwrap();
            loop {
                let mpris = &mpris_server;
                tokio::select! {
                    response = mpd_idle.next() => {
                        match response {
                            Some(ConnectionEvent::SubsystemChange(Subsystem::Database)) => {
                                let _ = qt_thread.queue(|mut qobject| {
                                    qobject.as_mut().db_updated(true);
                                });
                            }
                            Some(ConnectionEvent::SubsystemChange(Subsystem::Queue)) => {
                                notify_queue.notify_one();
                                if let Some(ref sender) = tx_actions {
                                    let _ = sender.send(MPSCCommand::IdleQueue).await;
                                } else {
                                    tracing::warn!("Connection not available");
                                }
                            }
                            Some(ConnectionEvent::SubsystemChange(Subsystem::Player)) => {
                                notify_player.notify_one();
                                if let Some(ref sender) = tx_actions {
                                    let _ = sender.send(MPSCCommand::IdlePlayer).await;
                                } else {
                                    tracing::warn!("Connection not available");
                                }
                            }
                            Some(ConnectionEvent::SubsystemChange(Subsystem::Options)) => {
                                if let Some(ref sender) = tx_actions {
                                    let _ = sender.send(MPSCCommand::IdleOptions).await;
                                } else {
                                    tracing::warn!("Connection not available");
                                }
                            }
                            None => {
                                tracing::warn!("Connection lost");
                                let _ = qt_thread.queue(|mut qobject| {
                                    qobject.as_mut().connection_update(QString::from("disconnected"));
                                });
                                break;
                            }
                            _ => {},
                        }
                    },
                    Some(response) = rx_mpris.recv() => {
                        let _ = mpris.properties_changed(response).await;
                    },
                    _ = notify.notified() => {
                        if let Some(ref sender) = tx_actions {
                            let _ = sender.send(MPSCCommand::IdleTimeline).await;
                        } else {
                            tracing::warn!("Connection not available (timeline)");
                        }
                    },
                    _ = interval.tick() => {
                        if let Some(ref sender) = tx_actions {
                            let _ = sender.send(MPSCCommand::IdleTimeline).await;
                        } else {
                            tracing::warn!("Connection not available (timeline)");
                        }
                    },
                    _ = cancel_token.cancelled() => {
                        tracing::warn!("Gracefully closing idle task");
                        break;
                    },
                };
            }
        });
    }

    fn init_ui(self: Pin<&mut Self>) {
        let mpd_client = self.client.clone().expect("mpd client is None");
        let qt_thread = self.qt_thread();
        let rt_action = &self.rt_action;
        tracing::debug!("Starting init_ui");
        rt_action.spawn(async move {
            let command = commands::Status;
            match mpd_client.command(command).await {
                Ok(rsp) => {
                    let play_state = QString::from(format!("{:#?}", rsp.state));
                    let _ = qt_thread.queue(|mut qobject| {
                        qobject.as_mut().init_actions();
                        qobject.as_mut().play_state_changed(play_state);
                    });
                }
                Err(e) => {
                    tracing::error!("{}", e);
                }
            };
        });
    }

    fn init_actions(mut self: Pin<&mut Self>) {
        let rx_actions = self.as_mut().rust_mut().rx_actions.take().expect("Actions receiver is None");
        let mpd_client = self.client.clone().expect("mpd client is None");
        let cancel_token = self.cancel.clone().expect("Cancel token is None");
        let qt_thread = self.qt_thread();
        let rt_handle = self.rt_action.handle().clone();
        tracing::debug!("Starting init_actions");
        let pool = ActionPool::new(rt_handle, mpd_client, cancel_token, qt_thread, rx_actions);
        pool.spawn_workers();
    }

    pub fn play_toggle(self: Pin<&mut Self>) {
        let tx_actions = self.tx_actions.clone();
        if let Some(ref sender) = tx_actions {
            let _ = sender.blocking_send(MPSCCommand::PlayToggle);
        } else {
            tracing::warn!("Connection not available");
        }
    }

    pub fn play_song(self: Pin<&mut Self>, id: u64) {
        let tx_actions = self.tx_actions.clone();
        if let Some(ref sender) = tx_actions {
            let _ = sender.blocking_send(MPSCCommand::PlaySong(id));
        } else {
            tracing::warn!("Connection not available");
        }
    }

    pub fn play_next(self: Pin<&mut Self>) {
        let tx_actions = self.tx_actions.clone();
        if let Some(ref sender) = tx_actions {
            let _ = sender.blocking_send(MPSCCommand::Next);
        } else {
            tracing::warn!("Connection not available");
        }
    }

    pub fn play_previous(self: Pin<&mut Self>) {
        let tx_actions = self.tx_actions.clone();
        if let Some(ref sender) = tx_actions {
            let _ = sender.blocking_send(MPSCCommand::Previous);
        } else {
            tracing::warn!("Connection not available");
        }
    }

    pub fn play_seek(self: Pin<&mut QMPDConnector>, value: u64) {
        let seek_to = Duration::from_secs(value);
        let tx_actions = self.tx_actions.clone();
        if let Some(ref sender) = tx_actions {
            let _ = sender.blocking_send(MPSCCommand::Seek(seek_to));
        } else {
            tracing::warn!("Connection not available");
        }
    }

    pub fn update_db(self: Pin<&mut QMPDConnector>) {
        tracing::debug!("Updating MPD DB");
        let tx_actions = self.tx_actions.clone();
        if let Some(ref sender) = tx_actions {
            let _ = sender.blocking_send(MPSCCommand::UpdateDb);
        } else {
            tracing::warn!("Connection not available");
        }
    }

    pub fn get_playlists(self: Pin<&mut QMPDConnector>, group: i32) {
        let group = SongField::from_i32(group).expect("bad group value");
        let tx_actions = self.tx_actions.clone();
        if let Some(ref sender) = tx_actions {
            let _ = sender.blocking_send(MPSCCommand::GetPlaylists(group));
        } else {
            tracing::warn!("Connection not available");
        }
    }

    pub fn stage_playlist(self: Pin<&mut QMPDConnector>, name: QString, group: i32) {
        let name = String::from(name);
        let group = SongField::from_i32(group).expect("bad group value");
        let tx_actions = self.tx_actions.clone();
        if let Some(ref sender) = tx_actions {
            let _ = sender.blocking_send(MPSCCommand::StagePlaylist(name, group));
        } else {
            tracing::warn!("Connection not available");
        }
    }

    pub fn sort_playlist(self: Pin<&mut QMPDConnector>, sort_column: i32, sort_order: i32) {
        let sort_column = SongField::from_i32(sort_column).expect("bad sort_column value");
        let sort_order = ColumnSort::from((sort_order, sort_column));
        let tx_actions = self.tx_actions.clone();
        if let Some(ref sender) = tx_actions {
            let _ = sender.blocking_send(MPSCCommand::SortPlaylist(sort_order));
        } else {
            tracing::warn!("Connection not available");
        }
    }

    pub fn shuffle_toggle(self: Pin<&mut QMPDConnector>, current_state: bool) {
        let tx_actions = self.tx_actions.clone();
        if let Some(ref sender) = tx_actions {
            let _ = sender.blocking_send(MPSCCommand::ShuffleToggle(current_state));
        } else {
            tracing::warn!("Connection not available");
        }
    }

    fn repeat_toggle(self: Pin<&mut QMPDConnector>, current_repeat: bool, current_single: bool) {
        let tx_actions = self.tx_actions.clone();
        if let Some(ref sender) = tx_actions {
            let _ = sender.blocking_send(MPSCCommand::RepeatToggle(current_repeat, current_single));
        } else {
            tracing::warn!("Connection not available");
        }
    }

    pub fn sync_state(self: Pin<&mut QMPDConnector>) {
        let tx_actions = self.tx_actions.clone();
        if let Some(ref sender) = tx_actions {
            let _ = sender.blocking_send(MPSCCommand::IdleQueue);
            let _ = sender.blocking_send(MPSCCommand::IdlePlayer);
            let _ = sender.blocking_send(MPSCCommand::IdleOptions);
            let _ = sender.blocking_send(MPSCCommand::IdleTimeline);
        } else {
            tracing::warn!("Connection not available");
        }
    }

    fn spawn_server_instance(self: Pin<&mut Self>, mpd_binary: &PathBuf, native_config: &String) {
        let rt_idle = &self.rt_idle;
        let mpd_binary = mpd_binary.clone();
        let native_config = native_config.clone();
        let cancel_token = self.cancel.clone().expect("Cancel token is None");
        rt_idle.spawn(async move {
            let mut command = Command::new(mpd_binary);
            command.args(["--no-daemon", &native_config]);
            let mut handle = command.spawn().expect("Failed to start mpd server");
            cancel_token.cancelled().await;
            match handle.kill().await {
                Ok(_) => tracing::warn!("Gracefully closed native mpd server"),
                Err(e) => tracing::error!("Failed to gracefully close native mpd sever {}", e),
            };
        });
    }

    fn start_native_server(self: Pin<&mut Self>, mpd_binary: &PathBuf, native_config: &String) {
        tracing::debug!("Starting native mpd server");
        let settings = Settings::load().blocking_read().clone();
        let isettings = InternalSettings::load().clone();
        init_native_mpd_config(settings, isettings);
        self.spawn_server_instance(mpd_binary, native_config);
    }

    fn connect_client(self: Pin<&mut Self>) {
        let qt_thread = self.qt_thread();
        let mut retcount = 5;
        let rt_idle = &self.rt_idle;
        rt_idle.spawn(async move {
            let (state, mpd_client, mpd_idle) = loop {
                let settings = Settings::load().read().await;
                tracing::debug!("Trying to connect to server {}", settings.mpd_socket);
                tokio::select! {
                    Ok(connection) = TcpStream::connect(&settings.mpd_socket) => {
                        let client = ClientController::connect(connection).await.expect("failed to init controller");
                        let connection_2 =
                            TcpStream::connect(&settings.mpd_socket).await.expect("failed to establish 2nd connection");
                        let idle = ClientIdler::connect(connection_2).await.expect("failed to init idler");
                        tracing::debug!("Succesfully connected to MPD server (tcp)");
                        break ("connected", Some(client), Some(idle));
                    },
                    Ok(connection) = UnixStream::connect(&settings.mpd_socket) => {
                        let client = ClientController::connect(connection).await.expect("failed to init controller");
                        let connection_2 =
                            UnixStream::connect(&settings.mpd_socket).await.expect("failed to establish 2nd connection");
                        let idle = ClientIdler::connect(connection_2).await.expect("failed to init idler");
                        tracing::debug!("Succesfully connected to MPD server (unix_socket)");
                        break ("connected", Some(client), Some(idle));
                    },
                    () = tokio::time::sleep(tokio::time::Duration::from_millis(80)) => {
                        retcount -= 1;
                        if retcount <= 0 {
                            tracing::error!("Failed to connect to MPD server");
                            break ("disconnected-action", None, None);
                        }
                        tracing::warn!("Failed to connect to MPD server, retrying");
                    },
                };
            };
            let _ = qt_thread.queue(move |mut qobject| {
                qobject.as_mut().rust_mut().client = mpd_client;
                qobject.as_mut().rust_mut().idle_client = mpd_idle;
                qobject.as_mut().connection_update(state.into());
            });
        });
    }

    pub fn connect(self: Pin<&mut Self>) {
        tracing::debug!("Connecting to mpd");
        if let Some(ref cancel) = self.cancel {
            tracing::debug!("Gracefully stopping previous connection");
            cancel.cancel();
        }
        self.connection_update(QString::from("disconnected"));
    }
}

impl cxx_qt::Initialize for qobject::QMPDConnector {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut()
            .on_connection_update(|mut qobject, msg| match String::from(msg).as_str() {
                "disconnected-action" => {
                    tracing::debug!("Exhausted all reconnection attempts");
                }
                "disconnected" => {
                    let settings = Settings::load().blocking_read();
                    let isettings = InternalSettings::load();
                    let cancel_token = CancellationToken::new();
                    if let Some(token) = qobject.cancel.clone() {
                        tracing::debug!("Issuing cancel on disconnect");
                        token.cancel();
                    };
                    qobject.as_mut().rust_mut().cancel.replace(cancel_token);
                    if settings.mpd_socket == isettings.native_socket {
                        tracing::debug!("Using native mpd server");
                        match which("mpd") {
                            Ok(mpd_binary) => {
                                tracing::debug!("Found mpd binary {:?}", mpd_binary);
                                qobject.as_mut().start_native_server(&mpd_binary, &isettings.native_config);
                            }
                            Err(err) => panic!("Using native socket, but no mpd binary was found, {:?}", err),
                        };
                    };
                    qobject.as_mut().connection_update(QString::from("connecting"));
                }
                "connected" => {
                    let (tx_actions, rx_actions) = tokio::sync::mpsc::channel(32);
                    let (tx_mpris, rx_mpris) = tokio::sync::mpsc::channel(32);
                    qobject.as_mut().rust_mut().tx_actions.replace(tx_actions);
                    qobject.as_mut().rust_mut().rx_actions.replace(rx_actions);
                    qobject.as_mut().rust_mut().tx_mpris.replace(tx_mpris);
                    qobject.as_mut().rust_mut().rx_mpris.replace(rx_mpris);
                    qobject.as_mut().idle();
                    qobject.as_mut().init_ui();
                    qobject.as_mut().sync_state();
                }
                "connecting" => {
                    tracing::debug!("Connecting to server");
                    qobject.connect_client();
                }
                _ => unreachable!(),
            })
            .release();
        self.as_mut()
            .on_active_song_changed(|qobject, _song_pos, _song_id| {
                let tx_actions = qobject.tx_actions.clone();
                if let Some(ref sender) = tx_actions {
                    let _ = sender.blocking_send(MPSCCommand::UpdateArt);
                } else {
                    tracing::warn!("Connection not available");
                }
            })
            .release();
        self.as_mut()
            .on_stage_playlist_result(|qobject, _data| {
                let tx_actions = qobject.tx_actions.clone();
                if let Some(ref sender) = tx_actions {
                    let _ = sender.blocking_send(MPSCCommand::UpdateArt);
                } else {
                    tracing::warn!("Connection not available");
                }
            })
            .release();
        // Mpris interface
        self.as_mut()
            .on_play_state_changed(|qobject, state| {
                let tx_mpris = qobject.tx_mpris.clone();
                if let Some(ref mpris) = tx_mpris {
                    let mpris_changes = match String::from(state).as_str() {
                        "Playing" => vec![
                            Property::PlaybackStatus(PlaybackStatus::Playing),
                            Property::CanGoNext(true),
                            Property::CanGoPrevious(true),
                        ],
                        "Paused" => vec![
                            Property::PlaybackStatus(PlaybackStatus::Paused),
                            Property::CanGoNext(true),
                            Property::CanGoPrevious(true),
                        ],
                        "Stopper" => vec![
                            Property::PlaybackStatus(PlaybackStatus::Stopped),
                            Property::CanGoNext(false),
                            Property::CanGoPrevious(false),
                        ],
                        _ => unreachable!(),
                    };
                    let _ = mpris.blocking_send(mpris_changes);
                };
            })
            .release();
        self.as_mut()
            .on_update_options(|qobject| {
                let tx_mpris = qobject.tx_mpris.clone();
                if let Some(ref mpris) = tx_mpris {
                    let repeat = qobject.repeat;
                    let single = qobject.single;
                    let shuffle = qobject.shuffle;
                    let mut mpris_changes = Vec::new();
                    match (repeat, single) {
                        (false, false) | (false, true) => mpris_changes.push(Property::LoopStatus(LoopStatus::None)),
                        (true, false) => mpris_changes.push(Property::LoopStatus(LoopStatus::Playlist)),
                        (true, true) => mpris_changes.push(Property::LoopStatus(LoopStatus::Track)),
                    };
                    match shuffle {
                        true => mpris_changes.push(Property::Shuffle(true)),
                        false => mpris_changes.push(Property::Shuffle(false)),
                    };
                    let _ = mpris.blocking_send(mpris_changes);
                }
            })
            .release();
    }
}

impl Default for MPDConnector {
    fn default() -> Self {
        let rt_idle = Builder::new_multi_thread().worker_threads(1).enable_io().enable_time().build().unwrap();
        let rt_action = Builder::new_multi_thread().worker_threads(3).build().unwrap();
        Self {
            client: None,
            idle_client: None,
            cancel: None,
            rt_idle,
            rt_action,
            tx_actions: None,
            rx_actions: None,
            tx_mpris: None,
            rx_mpris: None,
            repeat: false,
            single: false,
            shuffle: false,
        }
    }
}

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
        #[qproperty(bool, repeat, READ, WRITE, NOTIFY = update_options)]
        #[qproperty(bool, single, READ, WRITE, NOTIFY = update_options)]
        #[qproperty(bool, shuffle, READ, WRITE, NOTIFY = update_options)]
        type QMPDConnector = super::MPDConnector;

        #[qsignal]
        #[cxx_name = "connectionUpdate"]
        fn connection_update(self: Pin<&mut QMPDConnector>, status: QString);

        #[qsignal]
        #[cxx_name = "updateOptions"]
        fn update_options(self: Pin<&mut QMPDConnector>);

        #[qsignal]
        #[cxx_name = "playStateChanged"]
        fn play_state_changed(self: Pin<&mut QMPDConnector>, status: QString);

        #[qsignal]
        #[cxx_name = "activeSongChanged"]
        fn active_song_changed(self: Pin<&mut QMPDConnector>, song_pos: usize, song_id: u64);

        #[qsignal]
        #[cxx_name = "albumArtUpdate"]
        fn album_art_update(self: Pin<&mut QMPDConnector>, art: QString);

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
        #[cxx_name = "syncState"]
        fn sync_state(self: Pin<&mut QMPDConnector>);

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
        #[cxx_name = "playSeek"]
        fn play_seek(self: Pin<&mut QMPDConnector>, value: u64);

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

        #[qinvokable]
        #[cxx_name = "shuffleToggle"]
        fn shuffle_toggle(self: Pin<&mut QMPDConnector>, current_state: bool);

        #[qinvokable]
        #[cxx_name = "repeatToggle"]
        fn repeat_toggle(self: Pin<&mut QMPDConnector>, current_repeat: bool, current_single: bool);
    }

    impl cxx_qt::Threading for QMPDConnector {}
    impl cxx_qt::Initialize for QMPDConnector {}
}
