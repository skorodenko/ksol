use serde;
use std::fmt::{Display, Formatter, Result};

#[derive(serde::Deserialize, serde::Serialize, Copy, Clone, Debug)]
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
    Time = 13,
    Duration = 14,
    Directory = 15,
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
            Self::Time => write!(f, "Time"),
            Self::Duration => write!(f, "Duration"),
            Self::Directory => write!(f, "Directory"),
        }
    }
}
