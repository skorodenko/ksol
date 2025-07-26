from PySide6.QtQml import QmlElement
from PySide6.QtCore import (
    Qt,
    QAbstractListModel,
    QAbstractTableModel,
    Slot,
    Signal,
    Property,
    QModelIndex,
)

import qasync
import logging
from db import state
from settings import settings
from pydantic import TypeAdapter
from entities import Queue, SongField, Song
from pyqml.mpd_connector import mpd_client


logger = logging.getLogger("models")


QML_IMPORT_NAME = "models"
QML_IMPORT_MAJOR_VERSION = 1
QML_IMPORT_MINOR_VERSION = 0


@QmlElement
class QPlaylistsGroupModel(QAbstractListModel):
    groupChanged: Signal = Signal()

    def __init__(self):
        super().__init__()
        self.search_groups = settings.app.search_groups

    def data(self, index, role):
        name = self.roleNames().get(role)
        if name == b"name":
            group = self.search_groups[index.row()]
            if group in self.search_groups:
                return group.name.capitalize()
        if name == b"value":
            group = self.search_groups[index.row()]
            if group in self.search_groups:
                return group

    @Slot(SongField)
    def setActive(self, group: SongField):
        state.playlists_group = group
        self.groupChanged.emit()

    @Property(SongField, notify=groupChanged)
    def active(self):
        return state.playlists_group

    def roleNames(self):
        return {0: b"name", 1: b"value"}

    def rowCount(self, index) -> int:
        return len(self.search_groups)


@QmlElement
class QPlaylistsList(QAbstractListModel):
    def __init__(self):
        super().__init__()
        self.playlists = []
        self.playlists_proxy = []
        self.mpd_client = mpd_client
        self._filter = ""

    async def _lsinfo(self, root: str):
        retval = []
        data = await self.mpd_client.lsinfo(root)
        playlists = map(lambda x: x.get("directory", ""), data)
        playlists = list(filter(lambda x: x != "", playlists))
        songs = map(lambda x: x.get("file", ""), data)
        songs = list(filter(lambda x: x != "", songs))
        if len(songs) > 0:
            retval.append(root)
        if len(playlists) > 0:
            for p in playlists:
                retval.extend(await self._lsinfo(p))
        return retval

    @Property(str)
    def filter(self):
        return self._filter

    @filter.setter
    def filter(self, text: str):
        self.layoutAboutToBeChanged.emit()
        self.playlists_proxy = list(filter(lambda x: text in x, self.playlists))
        self._filter = text
        self.layoutChanged.emit()

    @qasync.asyncSlot(SongField)
    async def refresh(self, group: SongField):
        self.layoutAboutToBeChanged.emit()
        if group == SongField.directory:
            self.playlists = await self._lsinfo("")
        else:
            data = await self.mpd_client.list(group.name)
            playlists = map(lambda x: x.get(group.name, ""), data)
            playlists = list(filter(lambda x: x != "", playlists))
            self.playlists = playlists
        self.playlists_proxy = list(filter(lambda x: self.filter in x, self.playlists))
        self.layoutChanged.emit()

    def data(self, index, role):
        name = self.roleNames().get(role)
        if name == b"name":
            return self.playlists_proxy[index.row()]

    def roleNames(self):
        return {0: b"name"}

    def rowCount(self, index) -> int:
        return len(self.playlists_proxy)


@QmlElement
class QQueue(QAbstractTableModel):
    def __init__(self):
        super().__init__()
        self._queue: Queue | None = None

    @Property(Queue)
    def queue(self):
        return self._queue

    @queue.setter
    def queue(self, value):
        self.beginResetModel()
        self._queue = value
        self.endResetModel()

    #    @Slot(QUuid)
    #    def setActiveSong(self, value: QUuid):
    #        self.layoutAboutToBeChanged.emit()
    #        self._activeUuid = value
    #        self.layoutChanged.emit()

    def rowCount(self, index):
        return len(self._queue.contents) if self._queue is not None else 0

    def columnCount(self, index):
        return len(settings.app.playlist_table_cols)

    def roleNames(self):
        return {
            0: b"display",
            1: b"activeSong",
        }

    def data(self, index: QModelIndex, role: int):
        name = self.roleNames().get(role)
        if name == b"display":
            return getattr(
                self._queue.contents[index.row()],
                settings.app.playlist_table_cols[index.column()],
            )
        if name == b"activeSong":
            # active = getattr(self, "_activeUuid", None)
            column = index.column()
            return column == 0  # and active == self._queue[index.row()].uuid

    def headerData(self, section: int, orientation: Qt.Orientation, role: int):
        if role == Qt.ItemDataRole.DisplayRole:
            if orientation == Qt.Orientation.Horizontal:
                name = settings.app.playlist_table_cols[section]
                return name.capitalize()
