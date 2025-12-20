use crate::rust::mpd_actions::{self, Seek};
use crate::rust::services::MPDActionService;
use foyer::HybridCache;
use mpd_client::commands::SingleMode;
use mpd_client::responses::PlayState;
use mpris_server::{
    LoopStatus, Metadata, PlaybackRate, PlaybackStatus, PlayerInterface, RootInterface, Time, TrackId, Volume,
    zbus::{Result, fdo},
};
use std::time::Duration;
use tower::Service;
use zvariant::ObjectPath;

pub struct Player {
    pub mpd_service: MPDActionService,
    pub cover_cache: HybridCache<String, String>,
}

impl RootInterface for Player {
    async fn raise(&self) -> fdo::Result<()> {
        Ok(())
    }

    async fn quit(&self) -> fdo::Result<()> {
        Ok(())
    }

    async fn can_quit(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn fullscreen(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn set_fullscreen(&self, _fullscreen: bool) -> Result<()> {
        Ok(())
    }

    async fn can_set_fullscreen(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn can_raise(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn has_track_list(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn identity(&self) -> fdo::Result<String> {
        Ok("Ksol".to_string())
    }

    async fn desktop_entry(&self) -> fdo::Result<String> {
        Ok("Ksol".to_string())
    }

    async fn supported_uri_schemes(&self) -> fdo::Result<Vec<String>> {
        Ok(vec![])
    }

    async fn supported_mime_types(&self) -> fdo::Result<Vec<String>> {
        Ok(vec![])
    }
}

impl PlayerInterface for Player {
    async fn next(&self) -> fdo::Result<()> {
        let mut service = self.mpd_service.clone();
        service.call(mpd_actions::Next).await.map_err(|_| mpris_server::zbus::Error::InvalidReply)?;
        Ok(())
    }

    async fn previous(&self) -> fdo::Result<()> {
        let mut service = self.mpd_service.clone();
        service.call(mpd_actions::Previous).await.map_err(|_| mpris_server::zbus::Error::InvalidReply)?;
        Ok(())
    }

    async fn pause(&self) -> fdo::Result<()> {
        let mut service = self.mpd_service.clone();
        service.call(mpd_actions::PlayToggle).await.map_err(|_| mpris_server::zbus::Error::InvalidReply)?;
        Ok(())
    }

    async fn play_pause(&self) -> fdo::Result<()> {
        let mut service = self.mpd_service.clone();
        let _ = service.call(mpd_actions::PlayToggle).await.map_err(|_| mpris_server::zbus::Error::InvalidReply)?;
        Ok(())
    }

    async fn stop(&self) -> fdo::Result<()> {
        let mut service = self.mpd_service.clone();
        service.call(mpd_actions::PlayToggle).await.map_err(|_| mpris_server::zbus::Error::InvalidReply)?;
        Ok(())
    }

    async fn play(&self) -> fdo::Result<()> {
        let mut service = self.mpd_service.clone();
        service.call(mpd_actions::PlayToggle).await.map_err(|_| mpris_server::zbus::Error::InvalidReply)?;
        Ok(())
    }

    async fn seek(&self, offset: Time) -> fdo::Result<()> {
        let mut service = self.mpd_service.clone();
        service
            .call(Seek::new(Duration::from_secs(offset.as_secs() as u64)))
            .await
            .map_err(|_| mpris_server::zbus::Error::InvalidReply)?;
        Ok(())
    }

    async fn set_position(&self, _track_id: TrackId, position: Time) -> fdo::Result<()> {
        let mut service = self.mpd_service.clone();
        service
            .call(Seek::new(Duration::from_secs(position.as_secs() as u64)))
            .await
            .map_err(|_| mpris_server::zbus::Error::InvalidReply)?;
        Ok(())
    }

    async fn open_uri(&self, _uri: String) -> fdo::Result<()> {
        Ok(())
    }

    async fn playback_status(&self) -> fdo::Result<PlaybackStatus> {
        let mut service = self.mpd_service.clone();
        let status = service.call(mpd_actions::Status).await.map_err(|_| mpris_server::zbus::Error::InvalidReply)?;
        match status.state {
            PlayState::Paused => Ok(PlaybackStatus::Paused),
            PlayState::Stopped => Ok(PlaybackStatus::Stopped),
            PlayState::Playing => Ok(PlaybackStatus::Playing),
        }
    }

    async fn loop_status(&self) -> fdo::Result<LoopStatus> {
        let mut service = self.mpd_service.clone();
        let status = service.call(mpd_actions::Status).await.map_err(|_| mpris_server::zbus::Error::InvalidReply)?;
        match (status.repeat, status.single) {
            (true, SingleMode::Enabled) | (true, SingleMode::Oneshot) => Ok(LoopStatus::Track),
            (true, SingleMode::Disabled) => Ok(LoopStatus::Playlist),
            _ => Ok(LoopStatus::None),
        }
    }

    async fn set_loop_status(&self, loop_status: LoopStatus) -> Result<()> {
        let mut service = self.mpd_service.clone();
        let (repeat, single) = match loop_status {
            LoopStatus::None => (false, false),
            LoopStatus::Track => (true, true),
            LoopStatus::Playlist => (true, false),
        };
        let _ = service.call(mpd_actions::RepeatToggle::new(repeat, single)).await;
        Ok(())
    }

    async fn rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(PlaybackRate::default())
    }

    async fn set_rate(&self, _rate: PlaybackRate) -> Result<()> {
        Ok(())
    }

    async fn shuffle(&self) -> fdo::Result<bool> {
        let mut service = self.mpd_service.clone();
        let status = service.call(mpd_actions::Status).await.map_err(|_| mpris_server::zbus::Error::InvalidReply)?;
        Ok(status.random)
    }

    async fn set_shuffle(&self, shuffle: bool) -> Result<()> {
        let mut service = self.mpd_service.clone();
        service.call(mpd_actions::ShuffleToggle::new(!shuffle)).await.map_err(|_| mpris_server::zbus::Error::InvalidReply)?;
        Ok(())
    }

    async fn metadata(&self) -> fdo::Result<Metadata> {
        let mut service = self.mpd_service.clone();
        let song = service.call(mpd_actions::CurrentSong).await.map_err(|_| mpris_server::zbus::Error::InvalidReply)?;
        let ckey = format!("{}/{}", song.artist, song.album);
        let cover = self.cover_cache.get(&ckey).await.map_err(|_| mpris_server::zbus::Error::InvalidReply)?;
        let trackid: TrackId = ObjectPath::try_from(format!("{}", song.id)).unwrap_or_default().into();
        let metadata = match cover {
            Some(cover) => Metadata::builder()
                .title(&song.title)
                .artist([&song.artist])
                .album(&song.album)
                .trackid(trackid)
                .length(Time::from_secs(song.duration.as_secs() as i64))
                .art_url(String::from(cover.value()))
                .build(),
            None => Metadata::builder()
                .title(&song.title)
                .artist([&song.artist])
                .album(&song.album)
                .trackid(trackid)
                .length(Time::from_secs(song.duration.as_secs() as i64))
                .build(),
        };
        Ok(metadata)
    }

    async fn volume(&self) -> fdo::Result<Volume> {
        Ok(Volume::default())
    }

    async fn set_volume(&self, _volume: Volume) -> Result<()> {
        Ok(())
    }

    async fn position(&self) -> fdo::Result<Time> {
        let mut service = self.mpd_service.clone();
        let status = service.call(mpd_actions::Status).await.map_err(|_| mpris_server::zbus::Error::InvalidReply)?;
        match status.elapsed {
            Some(t) => Ok(Time::from_secs(t.as_secs().try_into().unwrap_or_default())),
            None => Ok(Time::ZERO),
        }
    }

    async fn minimum_rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(1.0)
    }

    async fn maximum_rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(1.0)
    }

    async fn can_go_next(&self) -> fdo::Result<bool> {
        let mut service = self.mpd_service.clone();
        let status = service.call(mpd_actions::Status).await.map_err(|_| mpris_server::zbus::Error::InvalidReply)?;
        match status.state {
            PlayState::Stopped => Ok(false),
            PlayState::Playing | PlayState::Paused => Ok(true),
        }
    }

    async fn can_go_previous(&self) -> fdo::Result<bool> {
        let mut service = self.mpd_service.clone();
        let status = service.call(mpd_actions::Status).await.map_err(|_| mpris_server::zbus::Error::InvalidReply)?;
        match status.state {
            PlayState::Stopped => Ok(false),
            PlayState::Playing | PlayState::Paused => Ok(true),
        }
    }

    async fn can_play(&self) -> fdo::Result<bool> {
        let mut service = self.mpd_service.clone();
        let status = service.call(mpd_actions::Status).await.map_err(|_| mpris_server::zbus::Error::InvalidReply)?;
        match status.state {
            PlayState::Stopped => Ok(false),
            PlayState::Playing | PlayState::Paused => Ok(true),
        }
    }

    async fn can_pause(&self) -> fdo::Result<bool> {
        let mut service = self.mpd_service.clone();
        let status = service.call(mpd_actions::Status).await.map_err(|_| mpris_server::zbus::Error::InvalidReply)?;
        match status.state {
            PlayState::Stopped => Ok(false),
            PlayState::Playing | PlayState::Paused => Ok(true),
        }
    }

    async fn can_seek(&self) -> fdo::Result<bool> {
        let mut service = self.mpd_service.clone();
        let status = service.call(mpd_actions::Status).await.map_err(|_| mpris_server::zbus::Error::InvalidReply)?;
        match status.state {
            PlayState::Stopped => Ok(false),
            PlayState::Playing | PlayState::Paused => Ok(true),
        }
    }

    async fn can_control(&self) -> fdo::Result<bool> {
        Ok(true)
    }
}
