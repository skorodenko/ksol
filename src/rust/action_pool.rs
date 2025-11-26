use crate::rust::entities::{ColumnSort, MPSCCommand, QSong, SongField};
use crate::rust::qmpd_connector::qobject::QMPDConnector;
use base64::prelude::*;
use bincode::config;
use bincode::serde::encode_to_vec;
use bytes::Bytes;
use cxx_qt::{CxxQtThread, CxxQtType};
use cxx_qt_lib::{QByteArray, QString};
use mpd_client::{ClientController, commands, filter::Filter, responses::PlayState, responses::Song, tag::Tag};
use quick_cache::unsync::Cache;
use tokio::runtime::Handle;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing;

pub(crate) struct ActionPool {
    num_workers: usize,
    art_url_cache: Cache<Bytes, String>,
    pub rt_handle: Handle,
    pub mpd_client: ClientController,
    pub cancel_token: CancellationToken,
    pub qt_thread: CxxQtThread<QMPDConnector>,
    pub rx_actions: Receiver<MPSCCommand>,
}

impl ActionPool {
    pub fn new(
        rt_handle: Handle,
        mpd_client: ClientController,
        cancel_token: CancellationToken,
        qt_thread: CxxQtThread<QMPDConnector>,
        rx_actions: Receiver<MPSCCommand>,
    ) -> ActionPool {
        let num_workers = rt_handle.metrics().num_workers();
        let art_url_cache = Cache::new(16);
        ActionPool { num_workers, art_url_cache, mpd_client, rt_handle, cancel_token, qt_thread, rx_actions }
    }

    pub fn spawn_workers(self) {
        let mut pool: Vec<Sender<MPSCCommand>> = Vec::with_capacity(self.num_workers);
        for id in 0..self.num_workers {
            let (wtx, wrx) = mpsc::channel::<MPSCCommand>(128);
            pool.push(wtx);
            self.rt_handle.spawn(ActionPool::worker(
                id,
                wrx,
                self.art_url_cache.clone(),
                self.mpd_client.clone(),
                self.cancel_token.clone(),
                self.qt_thread.clone(),
            ));
        }
        let mut rx = self.rx_actions;
        let mpd_client = self.mpd_client.clone();
        self.rt_handle.spawn(async move {
            let mut idx = 0;
            let _ = mpd_client.command(commands::SetBinaryLimit(16384)).await;
            while let Some(cmd) = rx.recv().await {
                let mut forward = cmd;
                loop {
                    match pool[idx].try_send(forward) {
                        Ok(_) => {
                            idx = (idx + 1) % self.num_workers;
                            break;
                        }
                        Err(mpsc::error::TrySendError::Full(cmd)) => {
                            forward = cmd;
                            idx = (idx + 1) % self.num_workers;
                            tokio::task::yield_now().await;
                        }
                        Err(mpsc::error::TrySendError::Closed(_)) => {
                            tracing::warn!("Gracefully closed action workers");
                            return;
                        }
                    }
                }
            }
        });
    }

    async fn worker(
        id: usize,
        mut rx: Receiver<MPSCCommand>,
        mut art_url_cache: Cache<Bytes, String>,
        mpd_client: ClientController,
        cancel_token: CancellationToken,
        qt_thread: CxxQtThread<QMPDConnector>,
    ) {
        loop {
            tokio::select! {
                Some(response) = rx.recv() => {
                    match response {
                        MPSCCommand::Next => {
                            let command = commands::Next;
                            let _ = mpd_client.command(command).await;
                        }
                        MPSCCommand::Previous => {
                            let command = commands::Previous;
                            let _ = mpd_client.command(command).await;
                        }
                        MPSCCommand::PlaySong(id) => {
                            let command = commands::Play::song(commands::SongId::from(id));
                            let _ = mpd_client.command(command).await;
                        }
                        MPSCCommand::PlayToggle => {
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
                                    tracing::error!("{}", e);
                                }
                            };
                        }
                        MPSCCommand::UpdateDb => {
                            let command = commands::Update::new();
                            let _ = mpd_client.command(command).await;
                            let _ = qt_thread.queue(|mut qobject| {
                                qobject.as_mut().db_updated(false);
                            });
                        }
                        MPSCCommand::GetPlaylists(group) => {
                            let group = Tag::from(group);
                            let mut result: Vec<String> = match group {
                                Tag::Other(value) if value == "Directory".into() => {
                                    let command = commands::ListDirs::root();
                                    if let Ok(rsp) = mpd_client.command(command).await {
                                        rsp
                                    } else {
                                        tracing::warn!("Connection not available");
                                        vec![]
                                    }
                                }
                                _ => {
                                    let command = commands::List::new(group);
                                    if let Ok(rsp) = mpd_client.command(command).await {
                                        rsp.values().map(|x| x.to_string()).collect()
                                    } else {
                                        tracing::warn!("Connection not available");
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
                        MPSCCommand::SortPlaylist(sort_order) => {
                            let command = commands::Queue::all();
                            match mpd_client.command(command).await {
                                Ok(songs) => {
                                    let mut songs: Vec<QSong> = songs.into_iter().map(QSong::from).collect();
                                    match sort_order {
                                        ColumnSort::Inactive => (),
                                        ColumnSort::Ascending(col) => match col {
                                            SongField::Track => songs.sort_by(|a, b| a.track.cmp(&b.track)),
                                            SongField::Title => songs.sort_by(|a, b| a.title.cmp(&b.title)),
                                            SongField::Artist => songs.sort_by(|a, b| a.artist.cmp(&b.artist)),
                                            SongField::Album => songs.sort_by(|a, b| a.album.cmp(&b.album)),
                                            SongField::Date => songs.sort_by(|a, b| a.date.cmp(&b.date)),
                                            SongField::Genre => songs.sort_by(|a, b| a.genre.cmp(&b.genre)),
                                            SongField::Disc => songs.sort_by(|a, b| a.disc.cmp(&b.disc)),
                                            SongField::Composer => songs.sort_by(|a, b| a.composer.cmp(&b.composer)),
                                            SongField::Albumartist => songs.sort_by(|a, b| a.artist.cmp(&b.artist)),
                                            SongField::File => songs.sort_by(|a, b| a.file.cmp(&b.file)),
                                            SongField::Format => songs.sort_by(|a, b| a.format.cmp(&b.format)),
                                            SongField::Lastmodified => songs.sort_by(|a, b| a.lastmodified.cmp(&b.lastmodified)),
                                            SongField::Duration => songs.sort_by(|a, b| a.duration.cmp(&b.duration)),
                                            SongField::Directory => songs.sort_by(|a, b| a.directory.cmp(&b.directory)),
                                        },
                                        ColumnSort::Descending(col) => match col {
                                            SongField::Track => songs.sort_by(|b, a| a.track.cmp(&b.track)),
                                            SongField::Title => songs.sort_by(|b, a| a.title.cmp(&b.title)),
                                            SongField::Artist => songs.sort_by(|b, a| a.artist.cmp(&b.artist)),
                                            SongField::Album => songs.sort_by(|b, a| a.album.cmp(&b.album)),
                                            SongField::Date => songs.sort_by(|b, a| a.date.cmp(&b.date)),
                                            SongField::Genre => songs.sort_by(|b, a| a.genre.cmp(&b.genre)),
                                            SongField::Disc => songs.sort_by(|b, a| a.disc.cmp(&b.disc)),
                                            SongField::Composer => songs.sort_by(|b, a| a.composer.cmp(&b.composer)),
                                            SongField::Albumartist => songs.sort_by(|b, a| a.artist.cmp(&b.artist)),
                                            SongField::File => songs.sort_by(|b, a| a.file.cmp(&b.file)),
                                            SongField::Format => songs.sort_by(|b, a| a.format.cmp(&b.format)),
                                            SongField::Lastmodified => songs.sort_by(|b, a| a.lastmodified.cmp(&b.lastmodified)),
                                            SongField::Duration => songs.sort_by(|b, a| a.duration.cmp(&b.duration)),
                                            SongField::Directory => songs.sort_by(|b, a| a.directory.cmp(&b.directory)),
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
                                    tracing::error!("{}", e);
                                }
                            }
                        }
                        MPSCCommand::StagePlaylist(name, group) => {
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
                        MPSCCommand::Seek(seek_to) => {
                            let command = commands::Seek(commands::SeekMode::Absolute(seek_to));
                            let _ = mpd_client.command(command).await;
                        }
                        MPSCCommand::ShuffleToggle(current_state) => {
                            let command = commands::SetRandom(!current_state);
                            let _ = mpd_client.command(command).await;
                        }
                        MPSCCommand::RepeatToggle(current_repeat, current_single) => {
                            let commands = match (current_repeat, current_single) {
                                (false, false) | (false, true) => (commands::SetRepeat(true), commands::SetSingle(commands::SingleMode::Disabled)),
                                (true, false) => (commands::SetRepeat(true), commands::SetSingle(commands::SingleMode::Enabled)),
                                (true, true) => (commands::SetRepeat(false), commands::SetSingle(commands::SingleMode::Disabled)),
                            };
                            let _ = mpd_client.command_list(commands).await;
                        }
                        MPSCCommand::IdlePlayer => {
                            let command_lst = (commands::Status, commands::CurrentSong);
                            match mpd_client.command_list(command_lst).await {
                                Ok(rsp) => {
                                    let play_state = QString::from(format!("{:#?}", rsp.0.state));
                                    let current_song = rsp.1.map(QSong::from);
                                    let _ = qt_thread.queue(move |mut qobject| {
                                        qobject.as_mut().rust_mut().active_song = current_song;
                                        qobject.as_mut().play_state_changed(play_state);
                                        qobject.as_mut().active_song_changed();
                                    });
                                }
                                Err(e) => {
                                    tracing::error!("{}", e);
                                }
                            };
                        }
                        MPSCCommand::IdleQueue => {
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
                                    tracing::error!("{}", e);
                                }
                            };
                        }
                        MPSCCommand::IdleOptions => {
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
                                    tracing::error!("{}", e);
                                }
                            }
                        }
                        MPSCCommand::IdleTimeline => {
                            let command = commands::Status;
                            match mpd_client.command(command).await {
                                Ok(rsp) => {
                                    let duration = rsp.duration.unwrap_or(Duration::new(0, 0));
                                    let elapsed = rsp.elapsed.unwrap_or(Duration::new(0, 0));
                                    let _ = qt_thread.queue(move |mut qobject| {
                                        qobject.as_mut().timeline_update(duration.as_secs(), elapsed.as_secs());
                                    });
                                }
                                Err(e) => {
                                    tracing::error!("{}", e);
                                }
                            };
                        }
                        MPSCCommand::UpdateArt => {
                            let command = commands::CurrentSong;
                            if let Ok(Some(song)) = mpd_client.command(command).await {
                                if let Ok(Some(sign)) = mpd_client.album_art_signature(&song.song.url).await {
                                    if let Some(image) = art_url_cache.get(&sign) {
                                        tracing::debug!("Using cached album art for {}", &song.song.url);
                                        let image = QString::from(image);
                                        let _ = qt_thread.queue(move |qobject| {
                                            qobject.album_art_update(image);
                                        });
                                    } else {
                                        tracing::debug!("New album art request for {}", &song.song.url);
                                        if let Ok(Some((image, Some(mime)))) = mpd_client.album_art(&song.song.url).await {
                                            tracing::debug!("Recieved album art for {}", &song.song.url);
                                            let image = format!("data:{};base64,{}", mime, BASE64_STANDARD.encode(image));
                                            art_url_cache.insert(sign, image.clone());
                                            let _ = qt_thread.queue(move |qobject| {
                                                qobject.album_art_update(QString::from(image));
                                            });
                                        };
                                    };
                                };
                            } else {
                                let _ = qt_thread.queue(move |qobject| {
                                    qobject.album_art_update(QString::from(""));
                                });
                            }
                        }
                    }
                },
                _ = cancel_token.cancelled() => {
                    tracing::warn!("Gracefully closing actions task {}", id);
                    break;
                },
            };
        }
    }
}
