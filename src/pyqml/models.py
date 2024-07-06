from PySide6.QtQml import QmlElement
from PySide6.QtCore import QAbstractListModel, Slot, Property

import qasync
from db import state
from settings import settings
from entities import PlaylistsGroup


QML_IMPORT_NAME = "models"
QML_IMPORT_MAJOR_VERSION = 1
QML_IMPORT_MINOR_VERSION = 0


@QmlElement
class QPlaylistsGroupModel(QAbstractListModel):
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
    
    @Property(int)
    def active(self) -> int:
        return state.playlists_group

    def roleNames(self):
        return {
            0: b"name",
            1: b"value",
        }

    def rowCount(self, index) -> int:
        return len(self.groups)

    @qasync.asyncSlot(PlaylistsGroup)
    async def refresh(self, group: PlaylistsGroup): ...
