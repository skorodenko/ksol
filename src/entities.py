from enum import IntEnum
from PySide6.QtCore import QUuid
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
    volume: int | None = None
    state: str | None = None
    repeat: bool | None = None
    random: bool | None = None
    single: bool | None = None
    consume: bool | None = None
    partition: str | None = None
    playlist: int | None = None
    playlistlength: int | None = None
    song: int | None = None
    songid: int | None = None
    nextsong: int | None = None
    nextsongid: int | None = None
    time: str | None = None
    elapsed: float | None = None
    duration: float | None = None
    bitrate: str | None = None
    xfade: int | None = None
    mixrampdb: int | None = None
    mixrampdelay: int | None = None
    audio: str | None = None
    updating_db: int | None = None
    error: str | None = None


class Song(BaseModel):
    uuid: QUuid = Field(default_factory=QUuid.createUuid)
    id: int | None = None
    pos: int | None = None
    file: str
    time: int
    duration: float
    lastmodified: datetime = Field(alias="last-modified")
    format: str = ""
    artist: str = ""
    albumartist: str = ""
    title: str = ""
    album: str = ""
    track: int | None = None
    date: int = 0
    genre: str = ""
    composer: str = ""
    disc: int = 0

    class Config:
        arbitrary_types_allowed = True

    @field_validator("artist", "albumartist", "genre", "composer", mode="before")
    @classmethod
    def _list_of_x_to_str(cls, val: str | list, info):
        if isinstance(val, list):
            return ", ".join(val)
        return val


class MetaTile(BaseModel):
    name: str
    pl_uuid: QUuid = Field(default_factory=QUuid.createUuid)
    locked: bool = False
    plgroup: SongField
    playlist: list[Song] = []
    
    class Config:
        arbitrary_types_allowed = True

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
        if isinstance(id, int):
            return self.playlist[id]
        if isinstance(id, QUuid):
            for song in self.playlist:
                if song.uuid == id:
                    return song
        raise NotImplementedError(f"Value {id} not allowed")

