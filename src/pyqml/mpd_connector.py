import socket
import logging
import asyncio
import subprocess
from shutil import which
from subprocess import Popen
from mpd.asyncio import MPDClient
from deepdiff import DeepDiff, Delta
from PySide6.QtQml import QmlElement
from PySide6.QtCore import QObject, Signal, Slot

import qasync
from db import state
from settings import settings
from entities import MPDStatus


logger = logging.getLogger("app")


QML_IMPORT_NAME = "controllers"
QML_IMPORT_MAJOR_VERSION = 1
QML_IMPORT_MINOR_VERSION = 0


mpd_client = MPDClient()


@QmlElement
class MPDConnector(QObject):
    connected: Signal = Signal(bool)
    dbUpdated: Signal = Signal(bool)

    def __init__(self):
        super().__init__()
        self.mpd_idle = None
        self.mpd_binary = which("mpd")
        self.mpd_server: Popen | None = None
        self.mpd_client: MPDClient = mpd_client

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
        if settings.mpd.socket == settings.mpd.native_socket:
            logger.debug("Using native mpd server")
            if self.mpd_binary:
                logger.debug(f"Found mpd binary: {self.mpd_binary}")
                self.mpd_server = self._start_native_server(
                    self.mpd_binary, settings.mpd.native_config
                )
            else:
                logger.warning("No mpd binary found")
        connected = await self._connect_mpd_client(settings.mpd.socket)
        if connected:
            self.mpd_idle = asyncio.create_task(self._mpd_idle())
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

    async def _mpd_idle(self):
        logger.debug("Starting idle task")
        state.mpd_status = MPDStatus()
        async for subsystem in self.mpd_client.idle():
            status_dict = await self.mpd_client.status()
            status = MPDStatus(**status_dict)
            ddiff = DeepDiff(state.mpd_status.dict(), status.dict())
            delta = {} + Delta(ddiff, force=True)
            state.mpd_status = status
            for pair in delta.items():
                self._action_router(pair)

    def _action_router(self, delta: tuple):
        match delta:
            case ("updating_db", state):
                if state is None:
                    self.dbUpdated.emit(True)
                else:
                    self.dbUpdated.emit(False)
            case _:
                ...

    @qasync.asyncSlot()
    async def refresh_db(self):
        await self.mpd_client.update()
