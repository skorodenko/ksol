import logging
from shutil import which
from subprocess import Popen
from settings import settings
from PySide6.QtQml import QmlElement
from PySide6.QtCore import QObject, Signal, Slot, QThread, QRunnable, QThreadPool

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
        self.thread_pool = QThreadPool()
        logger.debug(f"Starting threadpool ({self.thread_pool.maxThreadCount()})")
        self.mpd_server = MPDServer()
    
    @qasync.asyncSlot()
    async def connect(self):
        logger.debug("Connecting to mpd server")
        self.thread_pool.start(self.mpd_server)
        self.connected.emit()

    @Slot()
    def disconnect(self):
        logger.debug("Disconnecting from mpd server")
        self.mpd_server.stop()


class MPDServer(QRunnable, QThread):

    def __init__(self):
        super().__init__()
        self.mpd_binary = which("mpd")
        self.server_subproc = None

    @Slot()
    def run(self):
        if settings.mpd.socket == settings.mpd.native_socket:
            logger.debug("Connecting to native server")
            if self.mpd_binary:
                logger.debug(f"Found mpd binary: {self.mpd_binary}")
                self.server_subproc = Popen(
                    [self.mpd_binary, settings.mpd.native_config, "--no-daemon"]
                )
            else:
                logger.warning("No mpd binary found")

    def stop(self):
        if self.server_subproc:
            logger.debug("Terminating mpd server")
            self.server_subproc.terminate()

