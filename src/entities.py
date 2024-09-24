from enum import IntEnum
from uuid import UUID, uuid4
from datetime import datetime
from pydantic import BaseModel, Field, field_validator


class SongField(IntEnum):
    directory = 0
    file = 1
    time = 2
    duration = 3
    lastmodified = 4
    format = 5
    artist = 6
    albumartist = 7
    title = 8
    album = 9
    track = 10
    date = 11
    genre = 12
    composer = 13
    disc = 14

    def __str__(self) -> str:
        return str(self.value)

    def __repr__(self) -> str:
        return str(self.value)


class MPDStatus(BaseModel):
    volume: int = None
    state: str = None
    repeat: bool = None
    random: bool = None
    single: bool = None
    consume: bool = None
    partition: str = None
    playlist: int = None
    playlistlength: int = None
    song: int = None
    songid: int = None
    nextsong: int = None
    nextsongid: int = None
    time: str = None
    elapsed: float = None
    duration: float = None
    bitrate: str = None
    xfade: int = None
    mixrampdb: int = None
    mixrampdelay: int = None
    audio: str = None
    updating_db: int = None
    error: str = None


class Song(BaseModel):
    uuid: UUID = Field(default_factory=uuid4)
    songid: int = None
    file: str
    time: int
    duration: float
    lastmodified: datetime = Field(alias="last-modified")
    format: str = ""
    artist: str = ""
    albumartist: str = ""
    title: str = ""
    album: str = ""
    track: int = ""
    date: int = 0
    genre: str = ""
    composer: str = ""
    disc: int = 0

    @field_validator("artist", "albumartist", "genre", "composer", mode="before")
    @classmethod
    def _list_of_x_to_str(cls, val: str | list, info):
        if isinstance(val, list):
            return ", ".join(val)
        return val


class MetaTile(BaseModel):
    name: str
    sg_uuid: UUID = None
    pl_uuid: UUID = Field(default_factory=uuid4)
    locked: bool = False
    plgroup: SongField
    playlist: list[Song] = []

    def mpd_playlist_query(self):
        match self.name, self.plgroup:
            case name, SongField.directory:
                return [
                    f"(base '{name}')",
                ]
            case name, group:
                return [
                    f"({group.name} == '{name}')",
                ]

    def get_song(self, id):
        # If id is songid
        if isinstance(id, int):
            return self.playlist[id]

