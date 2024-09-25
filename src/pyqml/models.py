from PySide6.QtQml import QmlElement
from PySide6.QtCore import (
    Qt,
    QAbstractListModel,
    QAbstractTableModel,
    Slot,
    Signal,
    Property,
    QModelIndex,
    QUuid,
)

import qasync
import logging
from db import state
from settings import settings
from pydantic import TypeAdapter
from entities import SongField, MetaTile, Song
from pyqml.mpd_connector import mpd_client


logger = logging.getLogger("models")


QML_IMPORT_NAME = "models"
QML_IMPORT_MAJOR_VERSION = 1
QML_IMPORT_MINOR_VERSION = 0


@QmlElement
class QPlaylistsGroupModel(QAbstractListModel):
    groupChanged: Signal = Signal(SongField)

    def __init__(self):
        super().__init__()
        self.disabled_groups = settings.app.disabled_groups
        self.groups = [group for group in SongField]

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

    @Slot(SongField)
    def setActive(self, group: SongField):
        state.playlists_group = group
        self.groupChanged.emit(group)

    @Property(SongField)
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

    @qasync.asyncSlot(SongField)
    async def refresh(self, group: SongField):
        self.layoutAboutToBeChanged.emit()
        if group == SongField.directory:
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
    tileGridUpdate = Signal(list)

    def __init__(self):
        super().__init__()

    def _add_tile(self, tile: MetaTile):
        self.tileGridUpdate.emit(self.tiling_struct(self.size + 1))
        self.beginInsertRows(QModelIndex(), self.rowCount(), self.rowCount())
        with state.etile_stack as stack:
            stack.append(tile)
        self.endInsertRows()

    def _subst_tile(self, old: MetaTile, new: MetaTile):
        with state.etile_stack as stack:
            tile_index = stack.index(old)
            stack.pop(tile_index)
            stack.insert(tile_index, new)
        start = self.createIndex(tile_index, 0)
        stop = self.createIndex(self.size, 0)
        self.dataChanged.emit(start, stop)

    async def _populate_playlist(self, tile: MetaTile):
        query = tile.mpd_playlist_query()
        songs = await mpd_client.find(*query)
        if not songs:
            return []
        ta = TypeAdapter(list[Song])
        songs = ta.validate_python(songs)
        tile.playlist = songs
        return tile

    @qasync.asyncSlot(str)
    async def addTile(self, strid: str) -> bool:
        tile = MetaTile(name=strid, plgroup=state.playlists_group)
        logger.debug(f"Trying to add tile: '{tile}'")
        if self.size < settings.app.max_tiles:
            logger.debug(f"Adding tile: '{tile}'")
            tile = await self._populate_playlist(tile)
            self._add_tile(tile)
            return True
        if old_tile := self.first_unlocked:
            logger.debug(f"Changing '{old_tile}' to '{tile}'")
            tile = await self._populate_playlist(tile)
            self._subst_tile(old_tile, tile)
            return True
        logger.debug(f"Not enough place to add: '{tile}'")
        return False

    @qasync.asyncSlot(int)
    async def deleteTile(self, pos: int):
        logger.debug(f"Deleting tile at index: {pos}")
        self.beginRemoveRows(QModelIndex(), pos, pos)
        with state.etile_stack as stack:
            stack.pop(pos)
        self.endRemoveRows()
        start = self.createIndex(0, 0)
        stop = self.createIndex(self.size, 0)
        self.dataChanged.emit(start, stop)
        self.tileGridUpdate.emit(self.tiling_struct(self.size))

    @Property(int)
    def size(self):
        return len(state.tile_stack)

    @Property(MetaTile)
    def first_unlocked(self):
        for tile in state.tile_stack:
            if not tile.locked:
                return tile
        return None

    def tiling_struct(self, size: int):
        if size == 0:
            return []
        if size == 1:
            return [[2, 2]]
        if size == 2:
            return [[1, 2], [1, 2]]
        if size == 3:
            return [[1, 1], [1, 1], [2, 1]]
        if size == 4:
            return [[1, 1], [1, 1], [1, 1], [1, 1]]

    def data(self, index, role):
        name = self.roleNames().get(role)
        if name == b"pl_uuid":
            return state.tile_stack[index.row()].pl_uuid
        if name == b"name":
            return state.tile_stack[index.row()].name
        if name == b"playlist":
            return state.tile_stack[index.row()].playlist
        if name == b"tileIndex":
            return index.row()
        if name == b"tilingStruct":
            return self.tiling_struct(self.size)[index.row()]

    def roleNames(self):
        return {
            0: b"pl_uuid",
            1: b"name",
            2: b"playlist",
            3: b"tileIndex",
            4: b"tilingStruct",
        }

    def rowCount(self, index: QModelIndex = QModelIndex()) -> int:
        return self.size


@QmlElement
class QPlaylist(QAbstractTableModel):
    def __init__(self):
        super().__init__()

    @Property(list)
    def playlist(self):
        return self._playlist

    @playlist.setter
    def playlist(self, value):
        self._playlist = value

    @Slot(QUuid)
    def setActiveSong(self, value: QUuid):
        self.layoutAboutToBeChanged.emit()
        self._activeUuid = value
        self.layoutChanged.emit()

    def rowCount(self, index):
        return len(self._playlist)

    def columnCount(self, index):
        return len(settings.app.playlist_table_cols)

    def roleNames(self):
        return {
            0: b"display",
            1: b"sgUuid",
            2: b"activeSong",
        }

    def data(self, index: QModelIndex, role: int):
        name = self.roleNames().get(role)
        if name == b"display":
            return getattr(
                self._playlist[index.row()],
                settings.app.playlist_table_cols[index.column()],
            )
        if name == b"sgUuid":
            return self._playlist[index.row()].uuid
        if name == b"activeSong":
            active = getattr(self, "_activeUuid", None)
            column = index.column()
            return column == 0 and active == self._playlist[index.row()].uuid

    def headerData(self, section: int, orientation: Qt.Orientation, role: int):
        if role == Qt.ItemDataRole.DisplayRole:
            if orientation == Qt.Orientation.Horizontal:
                name = settings.app.playlist_table_cols[section]
                return name.capitalize()
