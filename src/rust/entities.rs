use core::time::Duration;
use mpd_client::{responses::Song, tag::Tag};
use num_derive::FromPrimitive;
use serde;
use std::fmt::{Display, Formatter, Result};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct QSong {
    pub track: i32,
    pub disc: i32,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub date: String,
    pub genre: String,
    pub composer: String,
    pub file: String,
    pub format: String,
    pub lastmodified: String,
    pub duration: Duration,
    pub directory: String,
}

#[derive(serde::Deserialize, serde::Serialize, FromPrimitive, Copy, Clone, Debug)]
#[repr(i32)]
pub enum SongField {
    Track = 1,
    Disc = 2,
    Title = 3,
    Artist = 4,
    Album = 5,
    Date = 6,
    Genre = 7,
    Composer = 8,
    Albumartist = 9,
    File = 10,
    Format = 11,
    Lastmodified = 12,
    Duration = 13,
    Directory = 14,
}

impl From<Song> for QSong {
    fn from(value: Song) -> Self {
        Self {
            track: value
                .tags
                .get(&Tag::Track)
                .unwrap()
                .join(",")
                .parse()
                .unwrap_or(0),
            disc: value
                .tags
                .get(&Tag::Disc)
                .unwrap()
                .join(",")
                .parse()
                .unwrap_or(0),
            title: value.tags.get(&Tag::Title).unwrap().join(","),
            artist: value.tags.get(&Tag::Artist).unwrap().join(","),
            album: value.tags.get(&Tag::Album).unwrap().join(","),
            date: value.tags.get(&Tag::Date).unwrap().join(","),
            genre: value.tags.get(&Tag::Genre).unwrap().join(","),
            composer: value.tags.get(&Tag::Composer).unwrap().join(","),
            file: value.url,
            format: value.format.unwrap_or("".into()),
            lastmodified: "".into(),
            duration: value.duration.unwrap(),
            directory: "".into(),
        }
    }
}

impl Display for SongField {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Self::Track => write!(f, "Track"),
            Self::Disc => write!(f, "Disc"),
            Self::Title => write!(f, "Title"),
            Self::Artist => write!(f, "Artist"),
            Self::Album => write!(f, "Album"),
            Self::Date => write!(f, "Date"),
            Self::Genre => write!(f, "Genre"),
            Self::Composer => write!(f, "Composer"),
            Self::Albumartist => write!(f, "Album artist"),
            Self::File => write!(f, "File"),
            Self::Format => write!(f, "Format"),
            Self::Lastmodified => write!(f, "Last modified"),
            Self::Duration => write!(f, "Duration"),
            Self::Directory => write!(f, "Directory"),
        }
    }
}
