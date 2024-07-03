from PySide6.QtQml import QmlElement
from PySide6.QtCore import Qt, QAbstractListModel, Signal

import qasync
from entities import PlaylistsGroup


QML_IMPORT_NAME = "models"
QML_IMPORT_MAJOR_VERSION = 1
QML_IMPORT_MINOR_VERSION = 0


@QmlElement
class Playlists(QAbstractListModel):
    def __init__(self):
        super().__init__()
        self.active = PlaylistsGroup.directory
        self.allowed_groups = []
    
    @qasync.asyncSlot(PlaylistsGroup)
    async def set_active(self, index: PlaylistsGroup):
        ...

    @qasync.asyncSlot(PlaylistsGroup)
    async def refresh(self, group: PlaylistsGroup):
        ...
