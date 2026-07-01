use std::path::PathBuf;

use super::BoxSyncFuture;
use crate::qt::mpd::qobject::QMPDConnector;
use crate::utils::globals::Globals;
use crate::{ColumnSort, QSong, SongField};
use anyhow::{Result, anyhow};
use cxx_qt::{CxxQtThread, CxxQtType};
use cxx_qt_lib::{QByteArray, QString};
use image;
use mpd_client::{ClientController, commands, filter::Filter, responses, tag::Tag};
use tokio::fs;
use tokio::sync::watch;
use tokio::task;
use tokio::time::Duration;
use tracing;

pub trait MPDAction {
    type Response;

    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<Self::Response>>;
}

/// Status command
#[derive(Default, Debug, Clone)]
pub struct Status;

impl MPDAction for Status {
    type Response = responses::Status;

    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<Self::Response>> {
        Box::pin(async move {
            tracing::debug!("Executing Status command");
            let command = commands::Status;
            mpd_client.command(command).await.map_err(|e| {
                tracing::error!("Status command failed: {e}");
                anyhow!("{e}")
            })
        })
    }
}

/// Queue command
#[derive(Default, Debug, Clone)]
pub struct Queue;

impl MPDAction for Queue {
    type Response = Vec<QSong>;

    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<Self::Response>> {
        Box::pin(async move {
            tracing::debug!("Executing Queue command");
            let command = commands::Queue::all();
            let songs: Vec<QSong> = mpd_client
                .command(command)
                .await
                .map_err(|e| {
                    tracing::error!("Queue command failed: {e}");
                    anyhow!("{e}")
                })?
                .into_iter()
                .map(QSong::from)
                .collect();
            Ok(songs)
        })
    }
}

/// Queue command
#[derive(Default, Debug, Clone)]
pub struct CurrentSong;

impl MPDAction for CurrentSong {
    type Response = QSong;

    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<Self::Response>> {
        Box::pin(async move {
            tracing::debug!("Executing CurrentSong command");
            let command = commands::CurrentSong;
            let song = mpd_client.command(command).await.map_err(|e| {
                tracing::error!("CurrentSong command failed: {e}");
                anyhow!("{e}")
            })?;
            match song {
                Some(v) => Ok(QSong::from(v)),
                None => Ok(QSong::default()),
            }
        })
    }
}

/// Next command
#[derive(Default, Debug, Clone)]
pub struct Next;

impl MPDAction for Next {
    type Response = ();

    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<Self::Response>> {
        Box::pin(async move {
            tracing::debug!("Executing Next command");
            let command = commands::Next;
            mpd_client.command(command).await.map_err(|e| {
                tracing::error!("Next command failed: {e}");
                anyhow!("{e}")
            })
        })
    }
}

/// Previous command
#[derive(Default, Debug, Clone)]
pub struct Previous;

impl MPDAction for Previous {
    type Response = ();

    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<Self::Response>> {
        Box::pin(async move {
            tracing::debug!("Executing Previous command");
            let command = commands::Previous;
            mpd_client.command(command).await.map_err(|e| {
                tracing::error!("Previous command failed: {e}");
                anyhow!("{e}")
            })
        })
    }
}

/// Play song command
#[derive(Debug, Clone)]
pub struct PlaySong {
    id: u64,
}

impl PlaySong {
    pub fn new(id: u64) -> PlaySong {
        PlaySong { id }
    }
}

impl MPDAction for PlaySong {
    type Response = ();

    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<Self::Response>> {
        Box::pin(async move {
            tracing::debug!("Executing PlaySong command for song id {}", self.id);
            let command = commands::Play::song(commands::SongId::from(self.id));
            mpd_client.command(command).await.map_err(|e| {
                tracing::error!("PlaySong command failed: {e}");
                anyhow!("{e}")
            })
        })
    }
}

/// Play toggle command
#[derive(Default, Debug, Clone)]
pub struct PlayToggle;

impl MPDAction for PlayToggle {
    type Response = ();

    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<Self::Response>> {
        Box::pin(async move {
            tracing::debug!("Executing PlayToggle command");
            let command = commands::Status;
            let rsp = mpd_client.command(command).await.map_err(|e| {
                tracing::error!("PlayToggle: Status command failed: {e}");
                anyhow!("{e}")
            })?;
            match rsp.state {
                responses::PlayState::Paused => {
                    let command = commands::SetPause(false);
                    mpd_client.command(command).await.map_err(|e| {
                        tracing::error!("PlayToggle: Resume command failed: {e}");
                        anyhow!("{e}")
                    })
                }
                responses::PlayState::Playing => {
                    let command = commands::SetPause(true);
                    mpd_client.command(command).await.map_err(|e| {
                        tracing::error!("PlayToggle: Pause command failed: {e}");
                        anyhow!("{e}")
                    })
                }
                responses::PlayState::Stopped => {
                    let command = commands::Play::current();
                    mpd_client.command(command).await.map_err(|e| {
                        tracing::error!("PlayToggle: Start command failed: {e}");
                        anyhow!("{e}")
                    })
                }
            }
        })
    }
}

// UpadteDB command
#[derive(Clone)]
pub struct UpdateDB {
    qt_thread: CxxQtThread<QMPDConnector>,
}

impl UpdateDB {
    pub fn new(qt_thread: CxxQtThread<QMPDConnector>) -> UpdateDB {
        UpdateDB { qt_thread }
    }
}

impl MPDAction for UpdateDB {
    type Response = ();

    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<Self::Response>> {
        Box::pin(async move {
            tracing::debug!("Executing UpdateDB command");
            let command = commands::Update::new();
            let _ = mpd_client.command(command).await.map_err(|e| {
                tracing::error!("UpdateDB command failed: {e}");
                anyhow!("{e}")
            })?;
            let _ = self.qt_thread.queue(|mut qobject| {
                qobject.as_mut().db_updated(false);
            });
            Ok(())
        })
    }
}

/// Get playlists command
#[derive(Clone)]
pub struct GetPlaylists {
    group: SongField,
    qt_thread: CxxQtThread<QMPDConnector>,
}

impl GetPlaylists {
    pub fn new(group: SongField, qt_thread: CxxQtThread<QMPDConnector>) -> GetPlaylists {
        GetPlaylists { group, qt_thread }
    }
}

impl MPDAction for GetPlaylists {
    type Response = ();

    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<Self::Response>> {
        Box::pin(async move {
            tracing::debug!("Executing GetPlaylists command for group {:?}", self.group);
            let group = Tag::from(self.group);
            let mut result: Vec<String> = match group {
                Tag::Other(value) if value == "Directory".into() => {
                    let command = commands::ListDirs::root();
                    mpd_client.command(command).await.map_err(|e| {
                        tracing::error!("GetPlaylists: ListDirs command failed: {e}");
                        anyhow!("{e}")
                    })?
                }
                _ => {
                    let command = commands::List::new(group);
                    let rsp = mpd_client.command(command).await.map_err(|e| {
                        tracing::error!("GetPlaylists: List command failed: {e}");
                        anyhow!("{e}")
                    })?;
                    rsp.values().map(|x| x.to_string()).collect()
                }
            };
            result.sort();
            let bcode: &[u8] = &wincode::serialize(&result).unwrap();
            let bcode = QByteArray::from(bcode);
            let _ = self.qt_thread.queue(|mut qobject| {
                qobject.as_mut().get_playlists_result(bcode);
            });
            Ok(())
        })
    }
}

/// Sort playlist command
#[derive(Debug, Clone)]
pub struct SortPlaylist {
    order: ColumnSort,
}

impl SortPlaylist {
    pub fn new(order: ColumnSort) -> SortPlaylist {
        SortPlaylist { order }
    }
}

impl MPDAction for SortPlaylist {
    type Response = ();

    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<Self::Response>> {
        Box::pin(async move {
            tracing::debug!("Executing SortPlaylist command with order {:?}", self.order);
            let command = commands::Queue::all();
            let mut songs: Vec<QSong> = mpd_client
                .command(command)
                .await
                .map_err(|e| {
                    tracing::error!("SortPlaylist: Queue command failed: {e}");
                    anyhow!("{e}")
                })?
                .into_iter()
                .map(QSong::from)
                .collect();
            songs.sort_by(|a, b| self.order.cmp_ord(a, b));
            let move_commands: Vec<commands::Move> = songs
                .iter()
                .enumerate()
                .map(|(i, x)| commands::Move::id(x.id.into()).to_position(i.into()))
                .collect();
            mpd_client.command_list(move_commands).await.map_err(|e| {
                tracing::error!("SortPlaylist: Move command list failed: {e}");
                anyhow!("{e}")
            })?;
            Ok(())
        })
    }
}

/// Stage playlist command
#[derive(Debug, Clone)]
pub struct StagePlaylist {
    name: String,
    group: SongField,
    order: ColumnSort,
}

impl StagePlaylist {
    pub fn new(name: String, group: SongField, order: ColumnSort) -> StagePlaylist {
        StagePlaylist { name, group, order }
    }
}

impl MPDAction for StagePlaylist {
    type Response = ();

    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<Self::Response>> {
        Box::pin(async move {
            let tag = Tag::from(self.group);
            tracing::debug!("StagePlaylist: Querying playlist for group {:?}", self.group);
            let songs = match tag {
                Tag::Other(value) if value == "Directory".into() => {
                    let command = commands::ListAllIn::directory(&self.name);
                    tracing::debug!("StagePlaylist: Listing all in directory {}", self.name);
                    mpd_client.command(command).await.map_err(|e| {
                        tracing::error!("StagePlaylist: Failed to list all in directory: {e}");
                        anyhow!("{e}")
                    })?
                }
                _ => {
                    let filter = Filter::tag(tag, &self.name);
                    let command = commands::Find::new(filter);
                    tracing::debug!("StagePlaylist: Finding by tag and name: {}", self.name);
                    mpd_client.command(command).await.map_err(|e| {
                        tracing::error!("StagePlaylist: Failed to find by tag and name: {e}");
                        anyhow!("{e}")
                    })?
                }
            };
            tracing::debug!("StagePlaylist: Found {} songs", songs.len());
            let clear_command = commands::ClearQueue;
            tracing::debug!("StagePlaylist: Clearing current queue");
            mpd_client.command(clear_command).await.map_err(|e| {
                tracing::error!("StagePlaylist: Failed to clear queue: {e}");
                anyhow!("{e}")
            })?;
            let add_commands: Vec<commands::Add> =
                songs.iter().map(|x| commands::Add::uri(x.url.as_str())).collect();
            tracing::debug!("StagePlaylist: Adding {} commands to queue", add_commands.len());
            mpd_client.command_list(add_commands).await.map_err(|e| {
                tracing::error!("StagePlaylist: Failed to add commands to queue: {e}");
                anyhow!("{e}")
            })?;
            tracing::debug!("Sorting playlist with order {:?}", self.order);
            let command = commands::Queue::all();
            let mut songs: Vec<QSong> = mpd_client
                .command(command)
                .await
                .map_err(|e| {
                    tracing::error!("SortPlaylist: Queue command failed: {e}");
                    anyhow!("{e}")
                })?
                .into_iter()
                .map(QSong::from)
                .collect();
            songs.sort_by(|a, b| self.order.cmp_ord(a, b));
            let move_commands: Vec<commands::Move> = songs
                .iter()
                .enumerate()
                .map(|(i, x)| commands::Move::id(x.id.into()).to_position(i.into()))
                .collect();
            mpd_client.command_list(move_commands).await.map_err(|e| {
                tracing::error!("SortPlaylist: Move command list failed: {e}");
                anyhow!("{e}")
            })?;
            Ok(())
        })
    }
}

/// Seek command
#[derive(Debug, Clone)]
pub struct Seek {
    to: Duration,
}

impl Seek {
    pub fn new(to: Duration) -> Seek {
        Seek { to }
    }
}

impl MPDAction for Seek {
    type Response = ();

    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<Self::Response>> {
        Box::pin(async move {
            tracing::debug!("Executing Seek command to {:?}", self.to);
            let command = commands::Seek(commands::SeekMode::Absolute(self.to));
            mpd_client.command(command).await.map_err(|e| {
                tracing::error!("Seek command failed: {e}");
                anyhow!("{e}")
            })
        })
    }
}

/// Shuffle toggle command
#[derive(Debug, Clone)]
pub struct ShuffleToggle {
    current: bool,
}

impl ShuffleToggle {
    pub fn new(current: bool) -> ShuffleToggle {
        ShuffleToggle { current }
    }
}

impl MPDAction for ShuffleToggle {
    type Response = ();

    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<Self::Response>> {
        let current = self.current;
        Box::pin(async move {
            tracing::debug!("Executing ShuffleToggle command (current={})", current);
            let command = commands::SetRandom(!current);
            mpd_client.command(command).await.map_err(|e| {
                tracing::error!("ShuffleToggle command failed: {e}");
                anyhow!("{e}")
            })
        })
    }
}

/// Repeat toggle command
#[derive(Debug, Clone)]
pub struct RepeatToggle {
    repeat: bool,
    single: bool,
}

impl RepeatToggle {
    pub fn new(repeat: bool, single: bool) -> RepeatToggle {
        RepeatToggle { repeat, single }
    }
}

impl MPDAction for RepeatToggle {
    type Response = ();

    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<Self::Response>> {
        Box::pin(async move {
            tracing::debug!(
                "Executing RepeatToggle command (repeat={}, single={})",
                self.repeat,
                self.single
            );
            let commands = match (self.repeat, self.single) {
                (false, false) | (false, true) => {
                    (commands::SetRepeat(true), commands::SetSingle(commands::SingleMode::Disabled))
                }
                (true, false) => {
                    (commands::SetRepeat(true), commands::SetSingle(commands::SingleMode::Enabled))
                }
                (true, true) => (
                    commands::SetRepeat(false),
                    commands::SetSingle(commands::SingleMode::Disabled),
                ),
            };
            mpd_client.command_list(commands).await.map_err(|e| {
                tracing::error!("RepeatToggle command list failed: {e}");
                anyhow!("{e}")
            })?;
            Ok(())
        })
    }
}

/// Idle player command
#[derive(Clone)]
pub struct IdlePlayer {
    qt_thread: CxxQtThread<QMPDConnector>,
}

impl IdlePlayer {
    pub fn new(qt_thread: CxxQtThread<QMPDConnector>) -> Self {
        Self { qt_thread }
    }
}

impl MPDAction for IdlePlayer {
    type Response = ();

    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<Self::Response>> {
        Box::pin(async move {
            tracing::debug!("Executing IdlePlayer command");
            let commands = (commands::Status, commands::CurrentSong);
            let rsp = mpd_client.command_list(commands).await.map_err(|e| {
                tracing::error!("IdlePlayer: Command list failed: {e}");
                anyhow!("{e}")
            })?;
            let play_state = QString::from(format!("{:#?}", rsp.0.state));
            let current_song = rsp.1.map(QSong::from).unwrap_or_default();
            let _ = self.qt_thread.queue(move |mut qobject| {
                let modified = qobject.active_song.0.send_if_modified(|song: &mut QSong| {
                    if song.file != current_song.file {
                        *song = current_song;
                        return true;
                    }
                    false
                });
                if modified {
                    qobject.as_mut().active_song_changed();
                };
                qobject.as_mut().play_state_changed(play_state);
            });
            Ok(())
        })
    }
}

/// Idle queue command
#[derive(Clone)]
pub struct IdleQueue {
    qt_thread: CxxQtThread<QMPDConnector>,
}

impl IdleQueue {
    pub fn new(qt_thread: CxxQtThread<QMPDConnector>) -> Self {
        Self { qt_thread }
    }
}

impl MPDAction for IdleQueue {
    type Response = ();

    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<Self::Response>> {
        Box::pin(async move {
            tracing::debug!("Executing IdleQueue command");
            let command = commands::Queue::all();
            let rsp = mpd_client.command(command).await.map_err(|e| {
                tracing::error!("IdleQueue: Queue command failed: {e}");
                anyhow!("{e}")
            })?;
            let songs: Vec<QSong> = rsp.into_iter().map(QSong::from).collect();
            let bcode: &[u8] = &wincode::serialize(&songs).unwrap();
            let bcode = QByteArray::from(bcode);
            let _ = self.qt_thread.queue(|mut qobject| {
                qobject.as_mut().stage_playlist_result(bcode);
            });
            Ok(())
        })
    }
}

/// Idle options command
#[derive(Clone)]
pub struct IdleOptions {
    qt_thread: CxxQtThread<QMPDConnector>,
}

impl IdleOptions {
    pub fn new(qt_thread: CxxQtThread<QMPDConnector>) -> Self {
        Self { qt_thread }
    }
}

impl MPDAction for IdleOptions {
    type Response = ();

    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<Self::Response>> {
        Box::pin(async move {
            tracing::debug!("Executing IdleOptions command");
            let command = commands::Status;
            let rsp = mpd_client.command(command).await.map_err(|e| {
                tracing::error!("IdleOptions: Status command failed: {e}");
                anyhow!("{e}")
            })?;
            let repeat = rsp.repeat;
            let shuffle = rsp.random;
            let single = !matches!(rsp.single, commands::SingleMode::Disabled);
            let _ = self.qt_thread.queue(move |mut qobject| {
                qobject.as_mut().rust_mut().repeat = repeat;
                qobject.as_mut().rust_mut().single = single;
                qobject.as_mut().rust_mut().shuffle = shuffle;
                qobject.update_options();
            });
            Ok(())
        })
    }
}

/// Idle timeline command
#[derive(Clone)]
pub struct IdleTimeline {
    qt_thread: CxxQtThread<QMPDConnector>,
}

impl IdleTimeline {
    pub fn new(qt_thread: CxxQtThread<QMPDConnector>) -> Self {
        Self { qt_thread }
    }
}

impl MPDAction for IdleTimeline {
    type Response = ();

    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<Self::Response>> {
        Box::pin(async move {
            tracing::trace!("Executing IdleTimeline command");
            let command = commands::Status;
            let rsp = mpd_client.command(command).await.map_err(|e| {
                tracing::error!("IdleTimeline: Status command failed: {e}");
                anyhow!("{e}")
            })?;
            let duration = rsp.duration.unwrap_or(Duration::new(0, 0));
            let elapsed = rsp.elapsed.unwrap_or(Duration::new(0, 0));
            let bitrate = rsp.bitrate.unwrap_or_default();
            let _ = self.qt_thread.queue(move |mut qobject| {
                qobject.as_mut().timeline_update(duration.as_secs(), elapsed.as_secs());
                qobject.as_mut().bitrate_update(bitrate);
            });
            Ok(())
        })
    }
}

/// Update art command
#[derive(Clone)]
pub struct UpdateArt {
    qt_thread: CxxQtThread<QMPDConnector>,
    song_watch: watch::Receiver<QSong>,
}

impl UpdateArt {
    pub fn new(qt_thread: CxxQtThread<QMPDConnector>, song_watch: watch::Receiver<QSong>) -> Self {
        Self { qt_thread, song_watch }
    }
}

impl MPDAction for UpdateArt {
    type Response = ();

    fn queue(
        mut self,
        mpd_client: ClientController,
    ) -> BoxSyncFuture<'static, Result<Self::Response>> {
        let globals = Globals::get();
        let covers = globals.app_cover_cache.clone();

        let ckey_fn = |song: &QSong, covers: &PathBuf, tmp: bool| {
            let safe_album =
                song.album.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
            let safe_artist =
                song.artist.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
            if tmp {
                covers.join(format!("{}_{}_{}.tmp", safe_album, safe_artist, song.disc))
            } else {
                covers.join(format!("{}_{}_{}", safe_album, safe_artist, song.disc))
            }
        };

        Box::pin(async move {
            let song = self.song_watch.borrow().clone();
            let ckey = ckey_fn(&song, &covers, false);
            let ckey_tmp = ckey_fn(&song, &covers, true);
            tracing::debug!("New album art request for \"{}\"", song.file);

            if ckey_tmp.exists() {
                let _ = self.qt_thread.queue(move |qobject| {
                    qobject.album_art_update(QString::default());
                });
                return Ok(());
            }

            if ckey.exists() {
                let _ = self.qt_thread.queue(move |qobject| {
                    qobject.album_art_update(QString::from(ckey.to_string_lossy().into_owned()));
                });
                return Ok(());
            }

            if [song.album, song.artist].iter().all(|x| x.is_empty()) {
                tracing::debug!("No album/artist metadata for cover cache \"{}\"", song.file);
                return Ok(());
            }

            if let Some(parent) = ckey_tmp.parent() {
                fs::create_dir_all(parent).await?;
            }

            if let Err(e) = fs::write(&ckey_tmp, vec![]).await {
                tracing::error!("Failed to write pending file {:?}: {}", ckey_tmp, e);
                return Ok(());
            }

            let cover = tokio::select!(
                Ok(Some((cover, Some(_)))) = mpd_client.album_art(&song.file) => {
                    tracing::debug!("Recieved album art for \"{}\"", song.file);
                    cover.freeze()
                },
                _ = self.song_watch.changed(), if ckey_fn(&self.song_watch.borrow().clone(), &covers, false) != ckey => {
                    tracing::debug!("Canceled album art request for \"{}\"", song.file);
                    fs::remove_file(&ckey_tmp).await.expect(&format!("Failed to delete pending file {:?}", ckey_tmp));
                    return Ok(());
                },
                else => {
                    fs::remove_file(&ckey_tmp).await.expect(&format!("Failed to delete pending file {:?}", ckey_tmp));
                    return Ok(());
                },
            );

            let ckey_clone = ckey.clone();
            let cover_processing = task::spawn_blocking(move || -> Result<PathBuf, String> {
                let image = image::load_from_memory(&cover)
                    .map_err(|e| format!("Failed to load image from memory: {}", e))?;
                image.save_with_format(&ckey_tmp, image::ImageFormat::Jpeg).map_err(|e| {
                    format!("Failed to write compressed image to {:?}: {}", ckey_tmp, e)
                })?;
                std::fs::rename(&ckey_tmp, &ckey_clone).map_err(|e| {
                    format!("Failed to rename {:?} to {:?}: {}", ckey_tmp, ckey_clone, e)
                })?;
                Ok(ckey_clone)
            });

            tokio::select!(
                res = cover_processing => {
                    match res.unwrap() {
                        Ok(cover_path) => {
                            let _ = self.qt_thread.queue(move |qobject| {
                                qobject.album_art_update(QString::from(cover_path.to_string_lossy().into_owned()));
                            });
                        },
                        Err(e) => {
                            tracing::error!("Album art processing error: {}", e);
                        },
                    }
                },
                _ = self.song_watch.changed(), if ckey_fn(&self.song_watch.borrow().clone(), &covers, false) != ckey => {
                    tracing::debug!("Canceled album art processing for \"{}\"", song.file);
                },
            );

            Ok(())
        })
    }
}

/// Set binary limit
#[derive(Debug, Clone)]
pub struct SetBinaryLimit(pub usize);

impl MPDAction for SetBinaryLimit {
    type Response = ();

    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<Self::Response>> {
        Box::pin(async move {
            tracing::debug!("Executing SetBinaryLimit command (limit={})", self.0);
            let command = commands::SetBinaryLimit(self.0);
            mpd_client.command(command).await.map_err(|e| {
                tracing::error!("SetBinaryLimit command failed: {e}");
                anyhow!("{e}")
            })
        })
    }
}
