import socket
import logging
import asyncio
import subprocess
from shutil import which
from subprocess import Popen
from mpd.asyncio import MPDClient
from PySide6.QtQml import QmlElement
from PySide6.QtCore import QObject, Signal, Slot

import qasync
from settings import settings


logger = logging.getLogger("app")


QML_IMPORT_NAME = "controllers"
QML_IMPORT_MAJOR_VERSION = 1
QML_IMPORT_MINOR_VERSION = 0


@QmlElement
class MPDConnector(QObject):
    connected: Signal = Signal(bool)

    def __init__(self):
        super().__init__()
        self.mpd_binary = which("mpd")
        self.mpd_server: Popen | None = None
        self.mpd_client: MPDClient = MPDClient()

    def _start_native_server(self, mpd_binary: str, mpd_native_config: str):
        args = [mpd_binary, "--no-daemon", mpd_native_config]
        logger.debug(f"Starting mpd server: {args}")
        return Popen(args)

    async def _connect_mpd_client(
        self, mpd_socket: str, retcount: int = 3, timeout: int = 1
    ):
        if retcount == 0:
            return False
        try:
            await self.mpd_client.connect(mpd_socket)
            logger.debug(f"Successfuly connected to mpd: {mpd_socket}")
            return True
        except (socket.gaierror, ConnectionRefusedError, FileNotFoundError):
            logger.warning(f"Failed to connect to mpd: {mpd_socket}")
            await asyncio.sleep(timeout)
            return await self._connect_mpd_client(mpd_socket, retcount - 1, timeout)

    @qasync.asyncSlot()
    async def connect(self):
        logger.debug("Establishing connection to mpd server")
        # Native server
        if settings.mpd.socket == settings.mpd.native_socket:
            logger.debug("Using native mpd server")
            if self.mpd_binary:
                logger.debug(f"Found mpd binary: {self.mpd_binary}")
                self.mpd_server = self._start_native_server(
                    self.mpd_binary, settings.mpd.native_config
                )
            else:
                logger.warning("No mpd binary found")
        # MPD Client
        connected = await self._connect_mpd_client(settings.mpd.socket)
        self.connected.emit(connected)

    @Slot()
    def disconnect(self):
        logger.debug("Gracefull close")
        if self.mpd_client.connected:
            logger.debug("Disconnecting from mpd server")
            self.mpd_client.disconnect()
        if self.mpd_server:
            logger.debug("Stopping native server")
            try:
                self.mpd_server.terminate()
                self.mpd_server.wait(3.0)
            except subprocess.TimeoutExpired:
                logger.warning(
                    "MPD server didn't terminate timeout. Killing MPD server"
                )
                self.mpd_server.kill()
