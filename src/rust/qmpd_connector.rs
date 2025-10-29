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

use qobject::*;

use crate::rust::entities::{ColumnSort, MPSCCommand, QSong, SongField};
use crate::rust::settings::{InternalSettings, Settings};
use base64::prelude::*;
use bincode::config;
use bincode::serde::encode_to_vec;
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use mpd_client::client::{ConnectionEvent, Subsystem};
use mpd_client::{
    ClientController, ClientIdler, commands, filter::Filter, responses, responses::PlayState, responses::Song, tag::Tag,
};
use num_traits::FromPrimitive;
use std::cmp::Reverse;
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
    pub repeat: bool,
    pub single: bool,
    pub shuffle: bool,
    pub rt_idle: Runtime,
    pub rt_timeline: Runtime,
    pub rt_action: Runtime,
    pub tx_actions: Option<Sender<MPSCCommand>>,
    pub rx_actions: Option<Receiver<MPSCCommand>>,
}

impl qobject::QMPDConnector {
    fn idle(mut self: Pin<&mut QMPDConnector>) {
        let mut mpd_idle = self.as_mut().rust_mut().idle_client.take();
        let qt_thread = self.qt_thread();
        let tx_actions = self.tx_actions.clone();
        let rt_idle = &self.rt_idle;
        rt_idle.spawn(async move {
            let mpd_idle = mpd_idle.as_mut().expect("idle client is None");
            loop {
                match mpd_idle.next().await {
                    Some(ConnectionEvent::SubsystemChange(Subsystem::Database)) => {
                        let _ = qt_thread.queue(|mut qobject| {
                            qobject.as_mut().db_updated(true);
                        });
                    }
                    Some(ConnectionEvent::SubsystemChange(Subsystem::Queue)) => {
                        log::debug!("Server queue changed");
                        if let Some(ref sender) = tx_actions {
                            let _ = sender.send(MPSCCommand::IdleQueue).await;
                        } else {
                            log::warn!("Connection not available");
                        }
                    }
                    Some(ConnectionEvent::SubsystemChange(Subsystem::Player)) => {
                        log::debug!("Server player changed");
                        if let Some(ref sender) = tx_actions {
                            let _ = sender.send(MPSCCommand::IdlePlayer).await;
                        } else {
                            log::warn!("Connection not available");
                        }
                    }
                    Some(ConnectionEvent::SubsystemChange(Subsystem::Options)) => {
                        log::debug!("Server options changed");
                        if let Some(ref sender) = tx_actions {
                            let _ = sender.send(MPSCCommand::IdleOptions).await;
                        } else {
                            log::warn!("Connection not available");
                        }
                    }
                    Some(e) => println!("Yay {:?}", e),
                    None => {
                        log::warn!("Connection lost");
                        let _ = qt_thread.queue(|mut qobject| {
                            qobject.as_mut().connection_update(QString::from("disconnected"));
                        });
                        break;
                    }
                }
            }
        });
    }

    fn init_ui(self: Pin<&mut Self>) {
        let mpd_client = self.client.clone().expect("mpd client is None");
        let qt_thread = self.qt_thread();
        let rt_action = &self.rt_action;
        rt_action.spawn(async move {
            let command = commands::Status;
            match mpd_client.command(command).await {
                Ok(rsp) => {
                    let play_state = QString::from(format!("{:#?}", rsp.state));
                    let _ = qt_thread.queue(|mut qobject| {
                        qobject.as_mut().init_actions();
                        qobject.as_mut().play_state_changed(play_state);
                        qobject.as_mut().init_timeline();
                    });
                }
                Err(e) => {
                    log::error!("{}", e);
                }
            };
        });
    }

    fn init_timeline(self: Pin<&mut Self>) {
        let mpd_client = self.client.clone().expect("mpd client is None");
        let qt_thread = self.qt_thread();
        let rt_timeline = &self.rt_timeline;
        rt_timeline.spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                let command = commands::Status;
                match mpd_client.command(command).await {
                    Ok(rsp) => {
                        let duration = rsp.duration.unwrap_or(Duration::new(0, 0));
                        let elapsed = rsp.elapsed.unwrap_or(Duration::new(0, 0));
                        let _ = qt_thread.queue(move |mut qobject| {
                            qobject.as_mut().timeline_update(duration.as_secs(), elapsed.as_secs());
                        });
                        interval.tick().await;
                    }
                    Err(e) => {
                        log::error!("{}", e);
                    }
                };
            }
        });
    }

    fn init_actions(mut self: Pin<&mut Self>) {
        let mut rx_actions = self.as_mut().rust_mut().rx_actions.take();
        let mpd_client = self.client.clone().expect("mpd client is None");
        let qt_thread = self.qt_thread();
        let rt_action = &self.rt_action;
        rt_action.spawn(async move {
            let rx_actions = rx_actions.as_mut().expect("actions reciever is None");
            loop {
                match rx_actions.recv().await {
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
                        match mpd_client.command(command).await {
                            Ok(rsp) => match rsp.state {
                                PlayState::Paused => {
                                    let command = commands::SetPause(false);
                                    let _ = mpd_client.command(command).await;
                                }
                                PlayState::Playing => {
                                    let command = commands::SetPause(true);
                                    let _ = mpd_client.command(command).await;
                                }
                                PlayState::Stopped => {
                                    let command = commands::Play::current();
                                    let _ = mpd_client.command(command).await;
                                }
                            },
                            Err(e) => {
                                log::error!("{}", e);
                            }
                        };
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
                                let command = commands::ListDirs::root();
                                if let Ok(rsp) = mpd_client.command(command).await {
                                    rsp
                                } else {
                                    log::warn!("Connection not available");
                                    vec![]
                                }
                            }
                            _ => {
                                let command = commands::List::new(group);
                                if let Ok(rsp) = mpd_client.command(command).await {
                                    rsp.values().map(|x| x.to_string()).collect()
                                } else {
                                    log::warn!("Connection not available");
                                    vec![]
                                }
                            }
                        };
                        result.sort();
                        let bcode: &[u8] = &encode_to_vec(result, config::standard()).expect("failed to encode to bcode");
                        let bcode = QByteArray::from(bcode);
                        let _ = qt_thread.queue(|mut qobject| {
                            qobject.as_mut().get_playlists_result(bcode);
                        });
                    }
                    Some(MPSCCommand::SortPlaylist(sort_order)) => {
                        let command = commands::Queue::all();
                        match mpd_client.command(command).await {
                            Ok(songs) => {
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
                                        SongField::Lastmodified => songs.sort_by_key(|k| k.clone().lastmodified),
                                        SongField::Duration => songs.sort_by_key(|k| k.clone().duration),
                                        SongField::Directory => songs.sort_by_key(|k| k.clone().directory),
                                    },
                                    ColumnSort::Descending(col) => match col {
                                        SongField::Track => songs.sort_by_key(|k| Reverse(k.clone().track)),
                                        SongField::Title => songs.sort_by_key(|k| Reverse(k.clone().title)),
                                        SongField::Artist => songs.sort_by_key(|k| Reverse(k.clone().artist)),
                                        SongField::Album => songs.sort_by_key(|k| Reverse(k.clone().album)),
                                        SongField::Date => songs.sort_by_key(|k| Reverse(k.clone().date)),
                                        SongField::Genre => songs.sort_by_key(|k| Reverse(k.clone().genre)),
                                        SongField::Disc => songs.sort_by_key(|k| Reverse(k.clone().disc)),
                                        SongField::Composer => songs.sort_by_key(|k| Reverse(k.clone().composer)),
                                        SongField::Albumartist => songs.sort_by_key(|k| Reverse(k.clone().artist)),
                                        SongField::File => songs.sort_by_key(|k| Reverse(k.clone().file)),
                                        SongField::Format => songs.sort_by_key(|k| Reverse(k.clone().format)),
                                        SongField::Lastmodified => songs.sort_by_key(|k| Reverse(k.clone().lastmodified)),
                                        SongField::Duration => songs.sort_by_key(|k| Reverse(k.clone().duration)),
                                        SongField::Directory => songs.sort_by_key(|k| Reverse(k.clone().directory)),
                                    },
                                };
                                let move_commands: Vec<commands::Move> = songs
                                    .iter()
                                    .enumerate()
                                    .map(|(i, x)| commands::Move::id(x.id.into()).to_position(i.into()))
                                    .collect();
                                let _ = mpd_client.command_list(move_commands).await;
                            }
                            Err(e) => {
                                log::error!("{}", e);
                            }
                        }
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
                        let add_commands: Vec<commands::Add> = songs.iter().map(|x| commands::Add::uri(x.url.as_str())).collect();
                        let _ = mpd_client.command_list(add_commands).await;
                    }
                    Some(MPSCCommand::Seek(seek_to)) => {
                        let command = commands::Seek(commands::SeekMode::Absolute(seek_to));
                        let _ = mpd_client.command(command).await;
                    }
                    Some(MPSCCommand::ShuffleToggle(current_state)) => {
                        let command = commands::SetRandom(!current_state);
                        let _ = mpd_client.command(command).await;
                    }
                    Some(MPSCCommand::RepeatToggle(current_repeat, current_single)) => {
                        match (current_repeat, current_single) {
                            (false, false) | (false, true) => {
                                let command = commands::SetRepeat(true);
                                let _ = mpd_client.command(command).await;
                                let command = commands::SetSingle(commands::SingleMode::Disabled);
                                let _ = mpd_client.command(command).await;
                            }
                            (true, false) => {
                                let command = commands::SetRepeat(true);
                                let _ = mpd_client.command(command).await;
                                let command = commands::SetSingle(commands::SingleMode::Enabled);
                                let _ = mpd_client.command(command).await;
                            }
                            (true, true) => {
                                let command = commands::SetRepeat(false);
                                let _ = mpd_client.command(command).await;
                                let command = commands::SetSingle(commands::SingleMode::Disabled);
                                let _ = mpd_client.command(command).await;
                            }
                        };
                    }
                    Some(MPSCCommand::IdlePlayer) => {
                        let command = commands::Status;
                        match mpd_client.command(command).await {
                            Ok(rsp) => {
                                let play_state = QString::from(format!("{:#?}", rsp.state));
                                let (song_pos, song_id) = match rsp.current_song {
                                    Some(song) => (song.0.0, song.1.0),
                                    None => (0, 0),
                                };
                                let _ = qt_thread.queue(move |mut qobject| {
                                    qobject.as_mut().play_state_changed(play_state);
                                    qobject.as_mut().active_song_changed(song_pos, song_id);
                                });
                            }
                            Err(e) => {
                                log::error!("{}", e);
                            }
                        };
                    }
                    Some(MPSCCommand::IdleQueue) => {
                        // Propagate playlist to other components
                        let command = commands::Queue::all();
                        match mpd_client.command(command).await {
                            Ok(rsp) => {
                                let songs: Vec<QSong> = rsp.into_iter().map(QSong::from).collect();
                                let bcode: &[u8] = &encode_to_vec(songs, config::standard()).expect("failed to encode to bcode");
                                let bcode = QByteArray::from(bcode);
                                let _ = qt_thread.queue(|mut qobject| {
                                    qobject.as_mut().stage_playlist_result(bcode);
                                });
                            }
                            Err(e) => {
                                log::error!("{}", e);
                            }
                        };
                    }
                    Some(MPSCCommand::IdleOptions) => {
                        let command = commands::Status;
                        match mpd_client.command(command).await {
                            Ok(rsp) => {
                                let repeat = rsp.repeat;
                                let shuffle = rsp.random;
                                let single = !matches!(rsp.single, commands::SingleMode::Disabled);
                                let _ = qt_thread.queue(move |mut qobject| {
                                    qobject.as_mut().rust_mut().repeat = repeat;
                                    qobject.as_mut().rust_mut().single = single;
                                    qobject.as_mut().rust_mut().shuffle = shuffle;
                                    qobject.update_options();
                                });
                            }
                            Err(e) => {
                                log::error!("{}", e);
                            }
                        }
                    }
                    Some(MPSCCommand::UpdateArt) => {
                        let command = commands::CurrentSong;
                        if let Ok(Some(song)) = mpd_client.command(command).await {
                            let command = commands::AlbumArtEmbedded::new(&song.song.url);
                            if let Ok(Some(art)) = mpd_client.command(command).await {
                                let mut image = art.data;
                                let mut offset = image.len();
                                let mime = art.mime.unwrap();
                                while image.len() < art.size {
                                    let command = commands::AlbumArtEmbedded::new(&song.song.url).offset(offset);
                                    if let Ok(Some(art)) = mpd_client.command(command).await {
                                        offset += art.data.len();
                                        image.extend_from_slice(&art.data);
                                    } 
                                }
                                let image = format!("data:{};base64,{}", mime, BASE64_STANDARD.encode(image));
                                let _ = qt_thread.queue(move |qobject| {
                                    qobject.album_art_update(QString::from(image));
                                });
                            }
                        } else {
                            log::error!("Error updating album art");
                        }
                    }
                    None => {}
                }
            }
        });
    }

    pub fn play_toggle(self: Pin<&mut Self>) {
        let tx_actions = self.tx_actions.clone();
        if let Some(ref sender) = tx_actions {
            let _ = sender.blocking_send(MPSCCommand::PlayToggle);
        } else {
            log::warn!("Connection not available");
        }
    }

    pub fn play_song(self: Pin<&mut Self>, id: u64) {
        let tx_actions = self.tx_actions.clone();
        if let Some(ref sender) = tx_actions {
            let _ = sender.blocking_send(MPSCCommand::PlaySong(id));
        } else {
            log::warn!("Connection not available");
        }
    }

    pub fn play_next(self: Pin<&mut Self>) {
        let tx_actions = self.tx_actions.clone();
        if let Some(ref sender) = tx_actions {
            let _ = sender.blocking_send(MPSCCommand::Next);
        } else {
            log::warn!("Connection not available");
        }
    }

    pub fn play_previous(self: Pin<&mut Self>) {
        let tx_actions = self.tx_actions.clone();
        if let Some(ref sender) = tx_actions {
            let _ = sender.blocking_send(MPSCCommand::Previous);
        } else {
            log::warn!("Connection not available");
        }
    }

    pub fn play_seek(self: Pin<&mut QMPDConnector>, value: u64) {
        let seek_to = Duration::from_secs(value);
        let tx_actions = self.tx_actions.clone();
        if let Some(ref sender) = tx_actions {
            let _ = sender.blocking_send(MPSCCommand::Seek(seek_to));
        } else {
            log::warn!("Connection not available");
        }
    }

    pub fn update_db(self: Pin<&mut QMPDConnector>) {
        log::debug!("Updating MPD DB");
        let tx_actions = self.tx_actions.clone();
        if let Some(ref sender) = tx_actions {
            let _ = sender.blocking_send(MPSCCommand::UpdateDb);
        } else {
            log::warn!("Connection not available");
        }
    }

    pub fn get_playlists(self: Pin<&mut QMPDConnector>, group: i32) {
        let group = SongField::from_i32(group).expect("bad group value");
        let tx_actions = self.tx_actions.clone();
        if let Some(ref sender) = tx_actions {
            let _ = sender.blocking_send(MPSCCommand::GetPlaylists(group));
        } else {
            log::warn!("Connection not available");
        }
    }

    pub fn stage_playlist(self: Pin<&mut QMPDConnector>, name: QString, group: i32) {
        let name = String::from(name);
        let group = SongField::from_i32(group).expect("bad group value");
        let tx_actions = self.tx_actions.clone();
        if let Some(ref sender) = tx_actions {
            let _ = sender.blocking_send(MPSCCommand::StagePlaylist(name, group));
        } else {
            log::warn!("Connection not available");
        }
    }

    pub fn sort_playlist(self: Pin<&mut QMPDConnector>, sort_column: i32, sort_order: i32) {
        let sort_column = SongField::from_i32(sort_column).expect("bad sort_column value");
        let sort_order = ColumnSort::from((sort_order, sort_column));
        let tx_actions = self.tx_actions.clone();
        if let Some(ref sender) = tx_actions {
            let _ = sender.blocking_send(MPSCCommand::SortPlaylist(sort_order));
        } else {
            log::warn!("Connection not available");
        }
    }

    pub fn shuffle_toggle(self: Pin<&mut QMPDConnector>, current_state: bool) {
        let tx_actions = self.tx_actions.clone();
        if let Some(ref sender) = tx_actions {
            let _ = sender.blocking_send(MPSCCommand::ShuffleToggle(current_state));
        } else {
            log::warn!("Connection not available");
        }
    }

    fn repeat_toggle(self: Pin<&mut QMPDConnector>, current_repeat: bool, current_single: bool) {
        let tx_actions = self.tx_actions.clone();
        if let Some(ref sender) = tx_actions {
            let _ = sender.blocking_send(MPSCCommand::RepeatToggle(current_repeat, current_single));
        } else {
            log::warn!("Connection not available");
        }
    }

    pub fn sync_state(self: Pin<&mut QMPDConnector>) {
        let tx_actions = self.tx_actions.clone();
        if let Some(ref sender) = tx_actions {
            let _ = sender.blocking_send(MPSCCommand::IdleQueue);
            let _ = sender.blocking_send(MPSCCommand::IdlePlayer);
            let _ = sender.blocking_send(MPSCCommand::IdleOptions);
        } else {
            log::warn!("Connection not available");
        }
    }

    fn start_native_server(mut self: Pin<&mut Self>, mpd_binary: &PathBuf, native_config: &String) {
        log::debug!("Starting native mpd server");
        let qt_thread = self.qt_thread();
        let cmpd_binary = mpd_binary.clone();
        let cnative_config = native_config.clone();
        let native_server = self.as_mut().rust_mut().server.take();
        match native_server {
            Some(mut server) => {
                match server.try_wait() {
                    // Previous server instance exited
                    Ok(Some(_)) => {
                        std::thread::spawn(move || {
                            let mut command = Command::new(cmpd_binary);
                            command.args(["--no-daemon", &cnative_config]);
                            let handle = command.spawn().expect("Failed to start mpd server");
                            let _ = qt_thread.queue(move |mut qobject| {
                                qobject.as_mut().rust_mut().server.replace(handle);
                            });
                        });
                    }
                    // Previous server is alive
                    Ok(None) => {}
                    // No idea what is here)
                    Err(err) => {
                        panic!("{}", err)
                    }
                }
            }
            None => {
                std::thread::spawn(move || {
                    let mut command = Command::new(cmpd_binary);
                    command.args(["--no-daemon", &cnative_config]);
                    let handle = command.spawn().expect("Failed to start mpd server");
                    let _ = qt_thread.queue(move |mut qobject| {
                        qobject.as_mut().rust_mut().server.replace(handle);
                    });
                });
            }
        }
    }

    fn connect_client(self: Pin<&mut Self>) {
        let qt_thread = self.qt_thread();
        let mut retcount = 3;
        let rt_idle = &self.rt_idle;
        rt_idle.spawn(async move {
            let (state, mpd_client, mpd_idle) = loop {
                let settings = Settings::load();
                //match TcpStream::connect(&settings.mpd_socket).await {
                match UnixStream::connect(&settings.mpd_socket).await {
                    Ok(connection) => {
                        let client = ClientController::connect(connection).await.expect("failed to init controller");
                        let connection_2 =
                            UnixStream::connect(&settings.mpd_socket).await.expect("failed to establish 2nd connection");
                        let idle = ClientIdler::connect(connection_2).await.expect("failed to init idler");
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
                        tokio::time::sleep(tokio::time::Duration::from_millis(75)).await;
                    }
                };
            };
            let _ = qt_thread.queue(|mut qobject| {
                qobject.as_mut().rust_mut().client = mpd_client;
                qobject.as_mut().rust_mut().idle_client = mpd_idle;
                qobject.as_mut().connection_update(state.into());
            });
        });
    }

    pub fn connect(self: Pin<&mut Self>) {
        log::debug!("Connecting to mpd");
        self.connection_update(QString::from("connecting"));
    }
}

impl cxx_qt::Initialize for qobject::QMPDConnector {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut()
            .on_connection_update(|mut qobject, msg| match String::from(msg).as_str() {
                "disconnected" => {
                    let settings = Settings::load();
                    let isettings = InternalSettings::load();
                    if settings.mpd_socket == isettings.native_socket {
                        log::debug!("Using native mpd server");
                        match which("mpd") {
                            Ok(v) => {
                                log::debug!("Found mpd binary {:?}", v);
                                qobject.as_mut().start_native_server(&v, &isettings.native_config);
                            }
                            Err(err) => panic!("Using native socket, but no mpd binary was found, {:?}", err),
                        };
                    }
                    qobject.as_mut().connection_update(QString::from("connecting"));
                }
                "connected" => {
                    let (tx_actions, rx_actions) = tokio::sync::mpsc::channel(64);
                    qobject.as_mut().rust_mut().tx_actions = Some(tx_actions);
                    qobject.as_mut().rust_mut().rx_actions = Some(rx_actions);
                    qobject.as_mut().idle();
                    qobject.as_mut().init_ui();
                    qobject.as_mut().sync_state();
                }
                "connecting" => {
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
                    log::warn!("Connection not available");
                }
            })
            .release();
    }
}

impl Default for MPDConnector {
    fn default() -> Self {
        let rt_idle = Builder::new_multi_thread().worker_threads(1).enable_io().enable_time().build().unwrap();
        let rt_timeline = Builder::new_multi_thread().worker_threads(1).enable_time().build().unwrap();
        let rt_action = Builder::new_multi_thread().worker_threads(1).build().unwrap();
        Self {
            client: None,
            idle_client: None,
            server: None,
            rt_idle,
            rt_timeline,
            rt_action,
            tx_actions: None,
            rx_actions: None,
            repeat: false,
            single: false,
            shuffle: false,
        }
    }
}

impl Drop for MPDConnector {
    fn drop(&mut self) {
        if let Some(server) = self.server.as_mut() {
            server.kill().unwrap();
        }
    }
}
