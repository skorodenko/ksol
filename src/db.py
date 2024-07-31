from peewee import SqliteDatabase, Model, IntegerField
from playhouse.kv import KeyValue, PickleField

from settings import settings
from entities import PlaylistsGroup, MPDStatus


db = SqliteDatabase(
    settings.core.state_db_location,
    pragmas={
        "journal_mode": "wal",
        "synchronous": "normal",
        "journal_size_limit": "6144000",
    },
)

mem_db = SqliteDatabase(
    ":memory:",
    pragmas={
        "journal_mode": "wal",
        "synchronous": "normal",
        "journal_size_limit": "6144000",
    },
)


class State:
    def __init__(self):
        self._kv_int = KeyValue(value_field=IntegerField(), database=db)
        self._kvmem_pickle = KeyValue(value_field=PickleField(), database=mem_db)

    @property
    def playlists_group(self) -> PlaylistsGroup:
        group = self._kv_int.get("playlists_group", 0)
        return PlaylistsGroup(group)

    @playlists_group.setter
    def playlists_group(self, group: PlaylistsGroup):
        self._kv_int["playlists_group"] = group.value

    @property
    def mpd_status(self):
        return self._kvmem_pickle.get("mpd_status", None)

    @mpd_status.setter
    def mpd_status(self, status: MPDStatus):
        self._kvmem_pickle["mpd_status"] = status


class BaseModel(Model):
    class Meta:
        database = db


state = State()
