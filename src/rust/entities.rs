use core::time::Duration;
use mpd_client::{responses::Song, tag::Tag};
use num_derive::FromPrimitive;
use serde;
use std::fmt::{Display, Formatter, Result};
use std::path::Path;
use strum::EnumIter;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct QSong {
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

#[derive(serde::Deserialize, serde::Serialize, EnumIter, FromPrimitive, Copy, Clone, Debug)]
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

impl From<Song> for QSong {
    fn from(value: Song) -> Self {
        Self {
            track: value
                .tags
                .get(&Tag::Track)
                .unwrap_or(&vec![])
                .join(",")
                .parse()
                .unwrap_or(0),
            disc: value
                .tags
                .get(&Tag::Disc)
                .unwrap_or(&vec![])
                .join(",")
                .parse()
                .unwrap_or(0),
            title: value.tags.get(&Tag::Title).unwrap_or(&vec![]).join(","),
            artist: value.tags.get(&Tag::Artist).unwrap_or(&vec![]).join(","),
            album: value.tags.get(&Tag::Album).unwrap_or(&vec![]).join(","),
            date: value.tags.get(&Tag::Date).unwrap_or(&vec![]).join(","),
            genre: value.tags.get(&Tag::Genre).unwrap_or(&vec![]).join(","),
            composer: value.tags.get(&Tag::Composer).unwrap_or(&vec![]).join(","),
            file: value.url.clone(),
            format: value.format.unwrap_or("".into()),
            lastmodified: "".into(),
            duration: value.duration.unwrap_or_default(),
            directory: Path::new(&value.url).parent().unwrap_or(Path::new("Root")).to_str().unwrap().to_string(),
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
