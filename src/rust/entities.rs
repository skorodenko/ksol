use core::time::Duration;
use mpd_client::{responses::SongInQueue, tag::Tag};
use num_derive::{FromPrimitive, ToPrimitive};
use serde;
use std::fmt::{Display, Formatter, Result};
use std::path::Path;
use strum::EnumIter;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default)]
pub struct QSong {
    pub id: u64,
    pub position: usize,
    pub track: i32,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub date: String,
    pub genre: String,
    pub disc: i32,
    pub composer: String,
    pub file: String,
    pub format: String,
    pub lastmodified: String,
    pub duration: Duration,
    pub directory: String,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, EnumIter, FromPrimitive, ToPrimitive, Copy, Clone, Debug)]
#[repr(i32)]
pub enum SongField {
    Track = 0,
    Title = 1,
    Artist = 2,
    Album = 3,
    Date = 4,
    Genre = 5,
    Disc = 6,
    Composer = 7,
    Albumartist = 8,
    File = 9,
    Format = 10,
    Lastmodified = 11,
    Duration = 12,
    Directory = 13,
}

#[derive(serde::Deserialize, serde::Serialize, Copy, Clone, Debug)]
#[repr(i32)]
pub enum ColumnSort {
    Inactive,
    Ascending(SongField),
    Descending(SongField),
}

impl From<SongInQueue> for QSong {
    fn from(value: SongInQueue) -> Self {
        Self {
            id: value.id.0,
            position: value.position.0,
            track: value.song.tags.get(&Tag::Track).unwrap_or(&vec![]).join(",").parse().unwrap_or(0),
            disc: value.song.tags.get(&Tag::Disc).unwrap_or(&vec![]).join(",").parse().unwrap_or(0),
            title: value.song.tags.get(&Tag::Title).unwrap_or(&vec![]).join(","),
            artist: value.song.tags.get(&Tag::Artist).unwrap_or(&vec![]).join(","),
            album: value.song.tags.get(&Tag::Album).unwrap_or(&vec![]).join(","),
            date: value.song.tags.get(&Tag::Date).unwrap_or(&vec![]).join(","),
            genre: value.song.tags.get(&Tag::Genre).unwrap_or(&vec![]).join(","),
            composer: value.song.tags.get(&Tag::Composer).unwrap_or(&vec![]).join(","),
            file: value.song.url.clone(),
            format: value.song.format.unwrap_or("".into()),
            lastmodified: "".into(),
            duration: value.song.duration.unwrap_or_default(),
            directory: Path::new(&value.song.url).parent().unwrap_or(Path::new("Root")).to_str().unwrap_or_default().to_string(),
        }
    }
}

impl From<(i32, SongField)> for ColumnSort {
    fn from(value: (i32, SongField)) -> Self {
        match value.0 {
            -1 => ColumnSort::Descending(value.1),
            1 => ColumnSort::Ascending(value.1),
            _ => ColumnSort::Inactive,
        }
    }
}

impl From<SongField> for Tag {
    fn from(val: SongField) -> Self {
        match val {
            SongField::Track => Tag::Track,
            SongField::Disc => Tag::Disc,
            SongField::Title => Tag::Title,
            SongField::Artist => Tag::Artist,
            SongField::Album => Tag::Album,
            SongField::Date => Tag::Date,
            SongField::Genre => Tag::Genre,
            SongField::Composer => Tag::Composer,
            SongField::Albumartist => Tag::AlbumArtist,
            SongField::File => Tag::Other("File".into()),
            SongField::Format => Tag::Other("Format".into()),
            SongField::Lastmodified => Tag::Other("Lastmodified".into()),
            SongField::Duration => Tag::Other("Duration".into()),
            SongField::Directory => Tag::Other("Directory".into()),
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
