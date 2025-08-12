use serde;

#[derive(serde::Deserialize, Debug)]
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
    Directory = 15
}

