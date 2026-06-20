pub mod qt;
pub mod service;
pub mod utils;

use mpd_client::{responses::SongInQueue, tag::Tag};
use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::path::Path;
use strum_macros::{Display, EnumCount, EnumString};
use wincode::{SchemaRead, SchemaWrite};

#[derive(Serialize, Deserialize, SchemaWrite, SchemaRead, Debug, Clone, Default)]
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
    pub duration: u64,
    pub directory: String,
}

#[derive(
    Deserialize,
    Serialize,
    SchemaRead,
    SchemaWrite,
    PartialEq,
    FromPrimitive,
    ToPrimitive,
    Display,
    EnumString,
    EnumCount,
    Copy,
    Clone,
    Debug,
)]
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

#[derive(Deserialize, Serialize, SchemaRead, SchemaWrite, PartialEq, Copy, Clone, Debug)]
#[repr(i32)]
pub enum ColumnSort {
    Inactive,
    Ascending(SongField),
    Descending(SongField),
}

#[derive(SchemaRead, SchemaWrite, Debug)]
pub struct HeaderColumn {
    name: String,
    width: f64,
    hidden: bool,
}

impl From<SongInQueue> for QSong {
    fn from(value: SongInQueue) -> Self {
        Self {
            id: value.id.0,
            position: value.position.0,
            track: value
                .song
                .tags
                .get(&Tag::Track)
                .unwrap_or(&vec![])
                .join(",")
                .parse()
                .unwrap_or(0),
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
            duration: value.song.duration.unwrap_or_default().as_secs(),
            directory: Path::new(&value.song.url)
                .parent()
                .unwrap_or(Path::new("Root"))
                .to_str()
                .unwrap_or_default()
                .to_string(),
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
