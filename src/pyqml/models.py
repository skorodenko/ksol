from PySide6.QtQml import QmlElement
from PySide6.QtCore import QAbstractListModel, Slot, Signal, Property, QModelIndex

import qasync
from db import state
from settings import settings
from entities import PlaylistsGroup
from pyqml.mpd_connector import mpd_client


QML_IMPORT_NAME = "models"
QML_IMPORT_MAJOR_VERSION = 1
QML_IMPORT_MINOR_VERSION = 0


@QmlElement
class QPlaylistsGroupModel(QAbstractListModel):
    groupChanged: Signal = Signal(PlaylistsGroup)

    def __init__(self):
        super().__init__()
        self.disabled_groups = settings.app.disabled_groups
        self.groups = [group for group in PlaylistsGroup]

    def data(self, index, role):
        name = self.roleNames().get(role)
        if name == b"name":
            group = self.groups[index.row()]
            if group not in self.disabled_groups:
                return group.name.capitalize()
        if name == b"value":
            group = self.groups[index.row()]
            if group not in self.disabled_groups:
                return group

    @Slot(PlaylistsGroup)
    def setActive(self, group: PlaylistsGroup):
        state.playlists_group = group
        self.groupChanged.emit(group)

    @Property(PlaylistsGroup)
    def active(self):
        return state.playlists_group

    def roleNames(self):
        return {0: b"name", 1: b"value"}

    def rowCount(self, index) -> int:
        return len(self.groups)


@QmlElement
class QPlaylistsList(QAbstractListModel):
    def __init__(self):
        super().__init__()
        self.playlists = []
        self.mpd_client = mpd_client

    @qasync.asyncSlot(PlaylistsGroup)
    async def refresh(self, group: PlaylistsGroup):
        self.layoutAboutToBeChanged.emit()
        if group == PlaylistsGroup.directory:
            data = await self.mpd_client.lsinfo("")
            self.playlists = list(map(lambda x: x.get("directory", ""), data))
        else:
            data = await self.mpd_client.list(group.name)
            self.playlists = list(map(lambda x: x.get(group.name, ""), data))
        self.layoutChanged.emit()

    def data(self, index, role):
        name = self.roleNames().get(role)
        if name == b"name":
            return self.playlists[index.row()]

    def roleNames(self):
        return {0: b"name"}

    def rowCount(self, index) -> int:
        return len(self.playlists)


@QmlElement
class QTilingStack(QAbstractListModel):
    tileAddStart: Signal = Signal(list)    
    tileAddEnd: Signal = Signal()

    def __init__(self):
        super().__init__()
        self.stack = []

    @qasync.asyncSlot(str)
    async def addTile(self, strid: str):
        self.tileAddStart.emit(self.tiling_struct(self.size + 1))
        self.beginInsertRows(QModelIndex(), self.rowCount(), self.rowCount())
        self.stack.append(strid)
        self.endInsertRows()
        self.tileAddEnd.emit()

    @Property(int)
    def size(self):
        return len(self.stack)

    def tiling_struct(self, size: int):
        if size == 0:
            return []
        if size == 1:
            return [[2,2]]
        if size == 2:
            return [[1,2], [1,2]]
        if size == 3:
            return [[1,1], [1,1], [2,1]]
        if size == 4:
            return [[1,1], [1,1], [1,1], [1,1]]

    def data(self, index, role):
        name = self.roleNames().get(role)
        if name == b"name":
            return self.stack[index.row()]
        if name == b"tileIndex":
            return index.row()
        if name == b"tilingStruct":
            return self.tiling_struct(self.size)[index.row()]

    def roleNames(self):
        return {0: b"name", 1: b"tileIndex", 2: b"tilingStruct"}

    def rowCount(self, index: QModelIndex = QModelIndex()) -> int:
        return len(self.stack)
