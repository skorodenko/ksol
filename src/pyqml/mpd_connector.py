import asyncio
import logging
import qasync
from PySide6.QtCore import QObject, Signal
from PySide6.QtQml import QmlElement, QmlSingleton


logger = logging.getLogger(__name__)


QML_IMPORT_NAME = "controllers"
QML_IMPORT_MAJOR_VERSION = 1
QML_IMPORT_MINOR_VERSION = 0 # Optional


@QmlElement
@QmlSingleton
class MainTest(QObject):
    connected: Signal = Signal()
    
    @qasync.asyncSlot()
    async def connect(self):
        SLEEP_TIME = 10
        for _ in range(SLEEP_TIME * 4):
            try:
                channel = Channel(path=config.default.grpc_host)
                service = TMpdServiceStub(channel)
                status = await service.connect(
                    ConnectionCredentials(
                        socket = config.default.native_socket
                    )
                )
                if status == ConnectionStatus.FailedToConnect:
                    continue
                break
            except ConnectionRefusedError:
                await asyncio.sleep(0.25)
        
        if status.status == ConnectionStatus.Connected:
            self.connected.emit()  
 
