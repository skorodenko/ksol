use crate::rust::entities::{ColumnSort, QSong, SongField};
use crate::rust::qmpd_connector::qobject::QMPDConnector;
use crate::rust::services::BoxSyncFuture;
use base64::prelude::*;
use bincode::config;
use bincode::serde::encode_to_vec;
use cxx_qt::{CxxQtThread, CxxQtType};
use cxx_qt_lib::{QByteArray, QString};
use mpd_client::{ClientController, commands, filter::Filter, responses::PlayState, tag::Tag};
use tokio::time::Duration;
use tracing;

pub trait MPDAction {
    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<(), MPDActionError>>;
}

#[derive(Debug)]
pub enum MPDActionError {
    MPDClientError(String),
}

/// Next command
#[derive(Default, Debug, Clone)]
pub struct Next;

impl MPDAction for Next {
    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<(), MPDActionError>> {
        Box::pin(async move {
            let command = commands::Next;
            mpd_client.command(command).await.map_err(|x| MPDActionError::MPDClientError(x.to_string()))
        })
    }
}

/// Previous command
#[derive(Default, Debug, Clone)]
pub struct Previous;

impl MPDAction for Previous {
    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<(), MPDActionError>> {
        Box::pin(async move {
            let command = commands::Previous;
            mpd_client.command(command).await.map_err(|x| MPDActionError::MPDClientError(x.to_string()))
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
    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<(), MPDActionError>> {
        Box::pin(async move {
            let command = commands::Play::song(commands::SongId::from(self.id));
            mpd_client.command(command).await.map_err(|x| MPDActionError::MPDClientError(x.to_string()))
        })
    }
}

/// Play toggle command
#[derive(Default, Debug, Clone)]
pub struct PlayToggle;

impl MPDAction for PlayToggle {
    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<(), MPDActionError>> {
        Box::pin(async move {
            let command = commands::Status;
            let rsp = mpd_client.command(command).await.map_err(|x| MPDActionError::MPDClientError(x.to_string()))?;
            match rsp.state {
                PlayState::Paused => {
                    let command = commands::SetPause(false);
                    mpd_client.command(command).await.map_err(|x| MPDActionError::MPDClientError(x.to_string()))
                }
                PlayState::Playing => {
                    let command = commands::SetPause(true);
                    mpd_client.command(command).await.map_err(|x| MPDActionError::MPDClientError(x.to_string()))
                }
                PlayState::Stopped => {
                    let command = commands::Play::current();
                    mpd_client.command(command).await.map_err(|x| MPDActionError::MPDClientError(x.to_string()))
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
    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<(), MPDActionError>> {
        Box::pin(async move {
            let command = commands::Update::new();
            let _ = mpd_client.command(command).await.map_err(|x| MPDActionError::MPDClientError(x.to_string()))?;
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
    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<(), MPDActionError>> {
        Box::pin(async move {
            let group = Tag::from(self.group);
            let mut result: Vec<String> = match group {
                Tag::Other(value) if value == "Directory".into() => {
                    let command = commands::ListDirs::root();
                    mpd_client.command(command).await.map_err(|x| MPDActionError::MPDClientError(x.to_string()))?
                }
                _ => {
                    let command = commands::List::new(group);
                    let rsp = mpd_client.command(command).await.map_err(|x| MPDActionError::MPDClientError(x.to_string()))?;
                    rsp.values().map(|x| x.to_string()).collect()
                }
            };
            result.sort();
            let bcode: &[u8] = &encode_to_vec(result, config::standard()).expect("failed to encode to bcode");
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
    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<(), MPDActionError>> {
        Box::pin(async move {
            let command = commands::Queue::all();
            let mut songs: Vec<QSong> = mpd_client
                .command(command)
                .await
                .map_err(|x| MPDActionError::MPDClientError(x.to_string()))?
                .into_iter()
                .map(QSong::from)
                .collect();
            match self.order {
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
            let move_commands: Vec<commands::Move> =
                songs.iter().enumerate().map(|(i, x)| commands::Move::id(x.id.into()).to_position(i.into())).collect();
            mpd_client.command_list(move_commands).await.map_err(|x| MPDActionError::MPDClientError(x.to_string()))?;
            Ok(())
        })
    }
}

/// Stage playlist command
#[derive(Debug, Clone)]
pub struct StagePlaylist {
    name: String,
    group: SongField,
}

impl StagePlaylist {
    pub fn new(name: String, group: SongField) -> StagePlaylist {
        StagePlaylist { name, group }
    }
}

impl MPDAction for StagePlaylist {
    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<(), MPDActionError>> {
        Box::pin(async move {
            let tag = Tag::from(self.group);
            // Query playlist
            let songs = match tag {
                Tag::Other(value) if value == "Directory".into() => {
                    let command = commands::ListAllIn::directory(&self.name);
                    mpd_client.command(command).await.map_err(|x| MPDActionError::MPDClientError(x.to_string()))?
                }
                _ => {
                    let filter = Filter::tag(tag, &self.name);
                    let command = commands::Find::new(filter);
                    mpd_client.command(command).await.map_err(|x| MPDActionError::MPDClientError(x.to_string()))?
                }
            };
            // Clear current queue
            let clear_command = commands::ClearQueue;
            mpd_client.command(clear_command).await.map_err(|x| MPDActionError::MPDClientError(x.to_string()))?;
            // Populate new queue
            let add_commands: Vec<commands::Add> = songs.iter().map(|x| commands::Add::uri(x.url.as_str())).collect();
            mpd_client.command_list(add_commands).await.map_err(|x| MPDActionError::MPDClientError(x.to_string()))?;
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
    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<(), MPDActionError>> {
        Box::pin(async move {
            let command = commands::Seek(commands::SeekMode::Absolute(self.to));
            mpd_client.command(command).await.map_err(|x| MPDActionError::MPDClientError(x.to_string()))
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
    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<(), MPDActionError>> {
        let current = self.current;
        Box::pin(async move {
            let command = commands::SetRandom(!current);
            mpd_client.command(command).await.map_err(|x| MPDActionError::MPDClientError(x.to_string()))
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
    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<(), MPDActionError>> {
        Box::pin(async move {
            let commands = match (self.repeat, self.single) {
                (false, false) | (false, true) => {
                    (commands::SetRepeat(true), commands::SetSingle(commands::SingleMode::Disabled))
                }
                (true, false) => (commands::SetRepeat(true), commands::SetSingle(commands::SingleMode::Enabled)),
                (true, true) => (commands::SetRepeat(false), commands::SetSingle(commands::SingleMode::Disabled)),
            };
            mpd_client.command_list(commands).await.map_err(|x| MPDActionError::MPDClientError(x.to_string()))?;
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
    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<(), MPDActionError>> {
        Box::pin(async move {
            let commands = (commands::Status, commands::CurrentSong);
            let rsp = mpd_client.command_list(commands).await.map_err(|x| MPDActionError::MPDClientError(x.to_string()))?;
            let play_state = QString::from(format!("{:#?}", rsp.0.state));
            let current_song = rsp.1.map(QSong::from);
            let _ = self.qt_thread.queue(move |mut qobject| {
                qobject.as_mut().rust_mut().active_song = current_song;
                qobject.as_mut().play_state_changed(play_state);
                qobject.as_mut().active_song_changed();
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
    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<(), MPDActionError>> {
        Box::pin(async move {
            // Propagate playlist to other components
            let command = commands::Queue::all();
            let rsp = mpd_client.command(command).await.map_err(|x| MPDActionError::MPDClientError(x.to_string()))?;
            let songs: Vec<QSong> = rsp.into_iter().map(QSong::from).collect();
            let bcode: &[u8] = &encode_to_vec(songs, config::standard()).expect("failed to encode to bcode");
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
    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<(), MPDActionError>> {
        Box::pin(async move {
            let command = commands::Status;
            let rsp = mpd_client.command(command).await.map_err(|x| MPDActionError::MPDClientError(x.to_string()))?;
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
    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<(), MPDActionError>> {
        Box::pin(async move {
            let command = commands::Status;
            let rsp = mpd_client.command(command).await.map_err(|x| MPDActionError::MPDClientError(x.to_string()))?;
            let duration = rsp.duration.unwrap_or(Duration::new(0, 0));
            let elapsed = rsp.elapsed.unwrap_or(Duration::new(0, 0));
            let _ = self.qt_thread.queue(move |mut qobject| {
                qobject.as_mut().timeline_update(duration.as_secs(), elapsed.as_secs());
            });
            Ok(())
        })
    }
}

/// Update art command
#[derive(Clone)]
pub struct UpdateArt {
    qt_thread: CxxQtThread<QMPDConnector>,
}

impl UpdateArt {
    pub fn new(qt_thread: CxxQtThread<QMPDConnector>) -> Self {
        Self { qt_thread }
    }
}

impl MPDAction for UpdateArt {
    fn queue(self, mpd_client: ClientController) -> BoxSyncFuture<'static, Result<(), MPDActionError>> {
        Box::pin(async move {
            let command = commands::CurrentSong;
            if let Ok(Some(song)) = mpd_client.command(command).await {
                let sign = mpd_client
                    .album_art_signature(&song.song.url)
                    .await
                    .map_err(|x| MPDActionError::MPDClientError(x.to_string()))?;
                //                    if let Some(image) = art_url_cache.get(&sign) {
                //                        tracing::debug!("Using cached album art for {}", &song.song.url);
                //                        let image = QString::from(image);
                //                        let _ = qt_thread.queue(move |qobject| {
                //                            qobject.album_art_update(image);
                //                        });
                //                    } else {
                tracing::debug!("New album art request for {}", &song.song.url);
                if let Ok(Some((image, Some(mime)))) = mpd_client.album_art(&song.song.url).await {
                    tracing::debug!("Recieved album art for {}", &song.song.url);
                    let image = format!("data:{};base64,{}", mime, BASE64_STANDARD.encode(image));
                    //art_url_cache.insert(sign, image.clone());
                    let _ = self.qt_thread.queue(move |qobject| {
                        qobject.album_art_update(QString::from(image));
                    });
                };
                //                   };
            } else {
                let _ = self.qt_thread.queue(move |qobject| {
                    qobject.album_art_update(QString::from(""));
                });
            }
            Ok(())
        })
    }
}
