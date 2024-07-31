from pydantic import BaseModel
from enum import IntEnum


class PlaylistsGroup(IntEnum):
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
    time: int = None
    elapsed: int = None
    duration: int = None
    bitrate: str = None
    xfade: int = None
    mixrampdb: int = None
    mixrampdelay: int = None
    audio: str = None
    updating_db: int = None
    error: str = None
