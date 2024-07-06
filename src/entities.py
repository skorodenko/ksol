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
