import socket
import logging
import asyncio
import subprocess
from shutil import which
from subprocess import Popen
from pydantic import TypeAdapter
from mpd.asyncio import MPDClient
from deepdiff import DeepDiff, Delta
from PySide6.QtQml import QmlElement
from PySide6.QtCore import QObject, Signal, Slot, QUuid

import qasync
from db import state
from settings import settings
from entities import MPDStatus, Song


logger = logging.getLogger("mpd_connector")


QML_IMPORT_NAME = "controllers"
QML_IMPORT_MAJOR_VERSION = 1
QML_IMPORT_MINOR_VERSION = 0


mpd_client = MPDClient()


@QmlElement
class MPDConnector(QObject):
    connected: Signal = Signal(str)
    dbUpdated: Signal = Signal(bool)
    statePlay: Signal = Signal(str)
    songChange: Signal = Signal(QUuid, QUuid)

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
        self, mpd_socket: str, retcount: int = 3, timeout: float = 1.0
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
        self.connected.emit("connecting")
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
        connected = await self._connect_mpd_client(
            settings.mpd.socket, retcount=8, timeout=0.1
        )
        if connected:
            self.mpd_idle = asyncio.create_task(self._mpd_idle())
            self.connected.emit("connected")
        else:
            self.connected.emit("disconnected")

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
        status_dict = await self.mpd_client.status()
        state.mpd_status = MPDStatus(**status_dict)
        status_dict = state.mpd_status.dict()
        # Initial UI update
        for pair in status_dict.items():
            # Ignore updating_db status 'cause no change occurs
            if pair[0] == "updating_db":
                continue
            await self._idle_action_router(pair)
        logger.debug("Finished initial UI update")
        async for subsystem in self.mpd_client.idle():
            status_dict = await self.mpd_client.status()
            status = MPDStatus(**status_dict)
            ddiff = DeepDiff(state.mpd_status.dict(), status.dict())
            delta = {} + Delta(ddiff, force=True)
            state.mpd_status = status
            # UI update
            for pair in delta.items():
                await self._idle_action_router(pair)

    async def _idle_action_router(self, delta: tuple):
        logger.debug(f"State router: {delta}")
        match delta:
            case ("updating_db", value):
                if value is None:
                    self.dbUpdated.emit(True)
                else:
                    self.dbUpdated.emit(False)
            case ("state", value):
                self.statePlay.emit(value)
            case ("songid", value):
                song = await mpd_client.playlistid(value)
                song = Song(**song[0])
                # If no tile match queue signature
                # there is active tile -> turn into generic
                # no active tile -> turn 1st into generic
                # no tiles at all -> create generic tile
            case _:
                ...

#    @qasync.asyncSlot(QUuid, QUuid)
#    async def stagePlaylist(self, pl_uuid: QUuid, sg_uuid: QUuid):
#        logger.debug(f"Staging playlist: {pl_uuid}/{sg_uuid}")
#        tile = state.get_tile(pl_uuid)
#        playpos = None
#        for i, song in enumerate(tile.playlist):
#            if song.uuid == sg_uuid:
#                playpos = i
#            await mpd_client.addid(song.file, i)
#        # status = await mpd_client.status()
#        # status = MPDStatus(**status)
#        # await mpd_client.delete((i+1, status.playlistlength))
#        await mpd_client.delete((i + 1, 9999))
#        await mpd_client.play(playpos)

    @qasync.asyncSlot()
    async def playNext(self):
        logger.debug("Play next")
        self.mpd_client.next()

    @qasync.asyncSlot()
    async def playPrevious(self):
        logger.debug("Play previous")
        self.mpd_client.previous()

    @qasync.asyncSlot()
    async def playToggle(self):
        status = await self.mpd_client.status()
        status = MPDStatus(**status)
        logger.debug(f"Play toggle. Current state {status.state}")
        match status.state:
            case "play":
                await self.mpd_client.pause(1)
            case "pause":
                await self.mpd_client.pause(0)

    @qasync.asyncSlot()
    async def refreshDb(self):
        logger.debug("Starting db update")
        await self.mpd_client.update()
