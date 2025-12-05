use qobject::*;

use crate::rust::entities::{ColumnSort, QSong, SongField};
use crate::rust::init_hooks::init_native_mpd_config;
use crate::rust::mpd_actions;
use crate::rust::mpris_actions;
use crate::rust::mpris_interface::Player;
use crate::rust::services::{MPDActionService, MPRISActionService};
use crate::rust::settings::{InternalSettings, Settings};
use bytes::Bytes;
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use moka::future::Cache;
use mpd_client::client::{ConnectionEvent, Subsystem};
use mpd_client::{ClientController, ClientIdler, commands};
use mpris_server::{LoopStatus, Metadata, PlaybackStatus, Property, Server, Time};
use num_traits::FromPrimitive;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::{TcpStream, UnixStream};
use tokio::process::Command;
use tokio::runtime::{Builder, Runtime};
use tokio::sync::{Notify, watch};
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;
use tower::{Service, ServiceBuilder};
use tracing;
use which::which;

pub struct MPDConnector {
    pub client: Option<ClientController>,
    pub idle_client: Option<ClientIdler>,
    pub cancel: Option<CancellationToken>,
    pub active_song: (watch::Sender<QSong>, watch::Receiver<QSong>),
    pub repeat: bool,
    pub single: bool,
    pub shuffle: bool,
    pub runtime: Runtime,
    pub cover_cache: Cache<Bytes, Arc<String>>,
    pub mpd_service: Option<MPDActionService>,
    pub mpris_service: Option<MPRISActionService>,
}

impl qobject::QMPDConnector {
    fn idle(mut self: Pin<&mut QMPDConnector>) {
        let mut mpd_idle = self.as_mut().rust_mut().idle_client.take();
        let qt_thread = self.qt_thread();
        let cancel_token = self.cancel.clone().expect("Cancel token is None");
        let mpd_service = self.mpd_service.clone();
        let runtime = self.runtime.handle().clone();
        tracing::debug!("Starting idle");
        self.runtime.spawn(async move {
            let notify = Arc::new(Notify::new());
            let notify_player = notify.clone();
            let notify_queue = notify.clone();
            let mpd_idle = mpd_idle.as_mut().expect("idle client is None");
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
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
                                if let Some(mut service) = mpd_service.clone() {
                                    runtime.spawn(service.call(mpd_actions::IdleQueue::new(qt_thread.clone())));
                                } else {
                                    tracing::error!("Action service not available");
                                }
                            }
                            Some(ConnectionEvent::SubsystemChange(Subsystem::Player)) => {
                                notify_player.notify_one();
                                interval.reset_immediately();
                                if let Some(mut service) = mpd_service.clone() {
                                    runtime.spawn(service.call(mpd_actions::IdlePlayer::new(qt_thread.clone())));
                                } else {
                                    tracing::error!("Action service not available");
                                }
                            }
                            Some(ConnectionEvent::SubsystemChange(Subsystem::Options)) => {
                                if let Some(mut service) = mpd_service.clone() {
                                    runtime.spawn(service.call(mpd_actions::IdleOptions::new(qt_thread.clone())));
                                } else {
                                    tracing::error!("Action service not available");
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
                    _ = notify.notified() => {
                        if let Some(mut service) = mpd_service.clone() {
                            runtime.spawn(service.call(mpd_actions::IdleTimeline::new(qt_thread.clone())));
                        } else {
                            tracing::error!("Action service not available");
                        }
                    },
                    _ = interval.tick() => {
                        if let Some(mut service) = mpd_service.clone() {
                            runtime.spawn(service.call(mpd_actions::IdleTimeline::new(qt_thread.clone())));
                        } else {
                            tracing::error!("Action service not available");
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
        tracing::debug!("Starting init_ui");
        self.runtime.spawn(async move {
            let command = commands::Status;
            match mpd_client.command(command).await {
                Ok(rsp) => {
                    let play_state = QString::from(format!("{:#?}", rsp.state));
                    let _ = qt_thread.queue(|mut qobject| {
                        qobject.as_mut().play_state_changed(play_state);
                    });
                }
                Err(e) => {
                    tracing::error!("{}", e);
                }
            };
        });
    }

    pub fn play_toggle(self: Pin<&mut Self>) {
        if let Some(mut service) = self.mpd_service.clone() {
            self.runtime.spawn(service.call(mpd_actions::PlayToggle));
        } else {
            tracing::error!("Action service not available");
        }
    }

    pub fn play_song(self: Pin<&mut Self>, id: u64) {
        if let Some(mut service) = self.mpd_service.clone() {
            self.runtime.spawn(service.call(mpd_actions::PlaySong::new(id)));
        } else {
            tracing::error!("Action service not available");
        }
    }

    pub fn play_next(self: Pin<&mut Self>) {
        if let Some(mut service) = self.mpd_service.clone() {
            self.runtime.spawn(service.call(mpd_actions::Next));
        } else {
            tracing::error!("Action service not available");
        }
    }

    pub fn play_previous(self: Pin<&mut Self>) {
        if let Some(mut service) = self.mpd_service.clone() {
            self.runtime.spawn(service.call(mpd_actions::Previous));
        } else {
            tracing::error!("Action service not available");
        }
    }

    pub fn play_seek(self: Pin<&mut QMPDConnector>, value: u64) {
        let seek_to = Duration::from_secs(value);
        if let Some(mut service) = self.mpd_service.clone() {
            self.runtime.spawn(service.call(mpd_actions::Seek::new(seek_to)));
        } else {
            tracing::error!("Action service not available");
        }
    }

    pub fn update_db(self: Pin<&mut QMPDConnector>) {
        tracing::debug!("Updating MPD DB");
        let qt_thread = self.qt_thread();
        if let Some(mut service) = self.mpd_service.clone() {
            self.runtime.spawn(service.call(mpd_actions::UpdateDB::new(qt_thread)));
        } else {
            tracing::error!("Action service not available");
        }
    }

    pub fn get_playlists(self: Pin<&mut QMPDConnector>, group: i32) {
        let group = SongField::from_i32(group).expect("bad group value");
        let qt_thread = self.qt_thread();
        if let Some(mut service) = self.mpd_service.clone() {
            self.runtime.spawn(service.call(mpd_actions::GetPlaylists::new(group, qt_thread)));
        } else {
            tracing::error!("Action service not available");
        }
    }

    pub fn stage_playlist(self: Pin<&mut QMPDConnector>, name: QString, group: i32) {
        let name = String::from(name);
        let group = SongField::from_i32(group).expect("bad group value");
        if let Some(mut service) = self.mpd_service.clone() {
            self.runtime.spawn(service.call(mpd_actions::StagePlaylist::new(name, group)));
        } else {
            tracing::error!("Action service not available");
        }
    }

    pub fn sort_playlist(self: Pin<&mut QMPDConnector>, sort_column: i32, sort_order: i32) {
        let sort_column = SongField::from_i32(sort_column).expect("bad sort_column value");
        let sort_order = ColumnSort::from((sort_order, sort_column));
        if let Some(mut service) = self.mpd_service.clone() {
            self.runtime.spawn(service.call(mpd_actions::SortPlaylist::new(sort_order)));
        } else {
            tracing::error!("Action service not available");
        }
    }

    pub fn shuffle_toggle(self: Pin<&mut QMPDConnector>, current_state: bool) {
        if let Some(mut service) = self.mpd_service.clone() {
            self.runtime.spawn(service.call(mpd_actions::ShuffleToggle::new(current_state)));
        } else {
            tracing::error!("Action service not available");
        }
    }

    fn repeat_toggle(self: Pin<&mut QMPDConnector>, current_repeat: bool, current_single: bool) {
        if let Some(mut service) = self.mpd_service.clone() {
            self.runtime.spawn(service.call(mpd_actions::RepeatToggle::new(current_repeat, current_single)));
        } else {
            tracing::error!("Action service not available");
        }
    }

    pub fn sync_state(self: Pin<&mut QMPDConnector>) {
        let qt_thread = self.qt_thread();
        if let Some(mut service) = self.mpd_service.clone() {
            self.runtime.spawn(async move {
                let _ = service.call(mpd_actions::SetBinaryLimit(16384)).await;
                let _ = service.call(mpd_actions::IdleQueue::new(qt_thread.clone())).await;
                let _ = service.call(mpd_actions::IdlePlayer::new(qt_thread.clone())).await;
                let _ = service.call(mpd_actions::IdleOptions::new(qt_thread.clone())).await;
                let _ = service.call(mpd_actions::IdleTimeline::new(qt_thread)).await;
            });
        } else {
            tracing::error!("Action service not available");
        }
    }

    fn spawn_server_instance(self: Pin<&mut Self>, mpd_binary: &PathBuf, native_config: &String) {
        let mpd_binary = mpd_binary.clone();
        let native_config = native_config.clone();
        let cancel_token = self.cancel.clone().expect("Cancel token is None");
        self.runtime.spawn(async move {
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
        self.runtime.spawn(async move {
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

    pub fn get_active_song_id(self: Pin<&mut Self>) -> u64 {
        let song = self.active_song.1.borrow();
        song.id
    }

    pub fn get_active_song_position(self: Pin<&mut Self>) -> usize {
        let song = self.active_song.1.borrow();
        song.position
    }

    pub fn get_active_song_title(self: Pin<&mut QMPDConnector>) -> QString {
        let song = self.active_song.1.borrow();
        QString::from(&song.title)
    }

    pub fn get_active_song_artist(self: Pin<&mut QMPDConnector>) -> QString {
        let song = self.active_song.1.borrow();
        QString::from(&song.artist)
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
                    let mpd_service = ServiceBuilder::new().service(MPDActionService::new(qobject.client.clone().unwrap()));
                    let mpris = qobject
                        .runtime
                        .block_on(async { Server::new("ksol", Player { mpd_service: mpd_service.clone() }).await.unwrap() });
                    let mpris_service = ServiceBuilder::new().service(MPRISActionService::new(mpris));
                    qobject.as_mut().rust_mut().mpd_service.replace(mpd_service);
                    qobject.as_mut().rust_mut().mpris_service.replace(mpris_service);
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
            .on_active_song_changed(|qobject| {
                let qt_thread = qobject.qt_thread();
                let cover_cache = qobject.cover_cache.clone();
                let song_watch = qobject.active_song.1.clone();
                if let Some(mut service) = qobject.mpd_service.clone() {
                    qobject.runtime.spawn(service.call(mpd_actions::UpdateArt::new(qt_thread, cover_cache, song_watch)));
                } else {
                    tracing::error!("Action service not available");
                }
            })
            .release();
        self.as_mut()
            .on_stage_playlist_result(|qobject, _data| {
                let qt_thread = qobject.qt_thread();
                let cover_cache = qobject.cover_cache.clone();
                let song_watch = qobject.active_song.1.clone();
                if let Some(mut service) = qobject.mpd_service.clone() {
                    qobject.runtime.spawn(service.call(mpd_actions::UpdateArt::new(qt_thread, cover_cache, song_watch)));
                } else {
                    tracing::error!("Action service not available");
                }
            })
            .release();

        // Mpris interface
        self.as_mut()
            .on_play_state_changed(|qobject, state| {
                if let Some(mut service) = qobject.mpris_service.clone() {
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
                    qobject.runtime.spawn(service.call(mpris_actions::PropertyUpdate::new(mpris_changes)));
                };
            })
            .release();
        self.as_mut()
            .on_update_options(|qobject| {
                if let Some(mut service) = qobject.mpris_service.clone() {
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
                    qobject.runtime.spawn(service.call(mpris_actions::PropertyUpdate::new(mpris_changes)));
                }
            })
            .release();
        self.as_mut()
            .on_timeline_update(|qobject, _, elapsed| {
                if let Some(mut service) = qobject.mpris_service.clone() {
                    qobject.runtime.spawn(service.call(mpris_actions::Seeked::new(Duration::from_secs(elapsed))));
                }
            })
            .release();
        self.as_mut()
            .on_album_art_update(|qobject, cover| {
                if let Some(mut service) = qobject.mpris_service.clone() {
                    let song = qobject.active_song.1.borrow();
                    let metadata = Metadata::builder()
                        .title(&song.title)
                        .artist([&song.artist])
                        .album(&song.album)
                        .length(Time::from_secs(song.duration.as_secs() as i64))
                        .art_url(String::from(cover))
                        .build();
                    qobject.runtime.spawn(service.call(mpris_actions::PropertyUpdate::new(vec![Property::Metadata(metadata)])));
                };
            })
            .release();
    }
}

impl Default for MPDConnector {
    fn default() -> Self {
        let runtime = Builder::new_multi_thread().enable_io().enable_time().global_queue_interval(1).event_interval(121).build().unwrap();
        let cover_cache = Cache::new(16);
        let active_song = watch::channel(QSong::default());
        Self {
            runtime,
            cover_cache,
            active_song,
            client: None,
            idle_client: None,
            cancel: None,
            mpd_service: None,
            mpris_service: None,
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
        #[qproperty(u64, activeSongId, READ = get_active_song_id, NOTIFY = active_song_changed)]
        #[qproperty(usize, activeSongPosition, READ = get_active_song_position, NOTIFY = active_song_changed)]
        #[qproperty(QString, activeSongTitle, READ = get_active_song_title, NOTIFY = active_song_changed)]
        #[qproperty(QString, activeSongArtist, READ = get_active_song_artist, NOTIFY = active_song_changed)]
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
        fn active_song_changed(self: Pin<&mut QMPDConnector>);

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
        fn get_active_song_id(self: Pin<&mut QMPDConnector>) -> u64;

        #[qinvokable]
        fn get_active_song_position(self: Pin<&mut QMPDConnector>) -> usize;

        #[qinvokable]
        fn get_active_song_title(self: Pin<&mut QMPDConnector>) -> QString;

        #[qinvokable]
        fn get_active_song_artist(self: Pin<&mut QMPDConnector>) -> QString;

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
