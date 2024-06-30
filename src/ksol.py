import os
import sys
import yaml
import pyqml
import signal
import logging.config
from pathlib import Path

from PyQt6.QtCore import QUrl
from PyQt6.QtGui import QGuiApplication
from PyQt6.QtQml import QQmlApplicationEngine, qmlRegisterType


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

    # Needed to close the app with Ctrl+C
    signal.signal(signal.SIGINT, signal.SIG_DFL)

    # Needed to get proper KDE style outside of Plasma
    if not os.environ.get("QT_QUICK_CONTROLS_STYLE"):
        os.environ["QT_QUICK_CONTROLS_STYLE"] = "org.kde.desktop"
    
    # Load qml files
    url = QUrl(str("file:" / base_path.absolute() / "src/qml/main.qml"))
    engine.load(url)

    if len(engine.rootObjects()) == 0:
        quit()

    app.exec()
    #with loop:
    #    loop.run_forever()
    logger.debug("Quitting app")


if __name__ == "__main__":
    main()
