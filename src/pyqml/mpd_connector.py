import logging
from shutil import which
from subprocess import Popen
from settings import settings
from PySide6.QtQml import QmlElement
from PySide6.QtCore import QObject, Signal, Slot

import qasync

logger = logging.getLogger("root")


QML_IMPORT_NAME = "controllers"
QML_IMPORT_MAJOR_VERSION = 1
QML_IMPORT_MINOR_VERSION = 0


@QmlElement
class MPDConnector(QObject):
    connected: Signal = Signal()

    def __init__(self):
        super().__init__()
        self.mpd_binary = which("mpd")
        self.mpd_server: Popen | None = None
    
    @qasync.asyncSlot()
    async def connect(self):
        logger.debug("Establishing connection to mpd server")
        if settings.mpd.socket == settings.mpd.native_socket:
            logger.debug("Using native mpd server")
            if self.mpd_binary:
                logger.debug(f"Found mpd binary: {self.mpd_binary}")
                self.mpd_server = Popen(
                    [self.mpd_binary, settings.mpd.native_config, "--no-daemon"]
                )
            else:
                logger.warning("No mpd binary found")
        self.connected.emit()

    @Slot()
    def disconnect(self):
        logger.debug("Disconnecting from mpd server")
        if self.mpd_server:
            logger.debug("Stopping native server")
            self.mpd_server.terminate()
            self.mpd_server.wait(3.0)

