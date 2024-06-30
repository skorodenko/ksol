import os
import sys
import yaml
import signal
import asyncio
import logging.config
from pathlib import Path
from PySide6.QtCore import QUrl
from PySide6.QtGui import QGuiApplication
from PySide6.QtQml import QQmlApplicationEngine

import pyqml
import qasync

base_path = Path(".")

with open(base_path / "logger.yml", "rt") as f:
    config = yaml.safe_load(f.read())

logging.config.dictConfig(config)
logger = logging.getLogger("root")


def main():
    logger.debug("Starting app")

    # Initializes and manages the application execution
    app = QGuiApplication(sys.argv)
    engine = QQmlApplicationEngine()
    loop = qasync.QEventLoop(app)
    asyncio.set_event_loop(loop)
    
    # Needed to close the app with Ctrl+C
    signal.signal(signal.SIGINT, signal.SIG_DFL)

    # Needed to get proper KDE style outside of Plasma
    if not os.environ.get("QT_QUICK_CONTROLS_STYLE"):
        os.environ["QT_QUICK_CONTROLS_STYLE"] = "org.kde.desktop"
    
    # Clean app stop
    app_close_event = asyncio.Event()
    app.aboutToQuit.connect(engine.deleteLater)
    engine.quit.connect(app.quit)
    app.aboutToQuit.connect(app_close_event.set)
    engine.quit.connect(app_close_event.set)

    # Load qml files
    url = QUrl(str("file:" / base_path.absolute() / "src/qml/main.qml"))
    engine.load(url)

    with loop:
        loop.run_forever()
    logger.debug("Quitting app")


if __name__ == "__main__":
    main()
