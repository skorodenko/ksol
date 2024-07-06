import toml
import hashlib
import logging
from pathlib import Path
from xdg_base_dirs import xdg_data_home, xdg_config_home


logger = logging.getLogger("app")


APP_CONFIG = xdg_config_home() / "ksol"
APP_DATA = xdg_data_home() / "ksol"


def init_default_settings_file(path: Path) -> bool:
    default_settings = {
        "mpd": {
            "socket": str(APP_DATA / "mpd.socket"),
            "native_socket": str(APP_DATA / "mpd.socket"),
            "native_config": str(APP_CONFIG / "mpd.conf"),
        },
        "core": {
            "config_location": str(APP_CONFIG / "mpd.conf")
        }
    }
    
    default_settings = toml.dumps(default_settings)
    
    # If file doesn't exist -> create file
    if not path.exists():
        with open(path, "w") as f:
            f.write(default_settings)
        return True

    # File exists -> check if hashes are identical
    default_hash = hashlib.sha256(default_settings.encode()).hexdigest()
    file_hash = hashlib.sha256(open(path, "rb").read()).hexdigest()
    if default_hash != file_hash:
        with open(path, "w") as f:
            f.write(default_settings)
        return True

    return False


USE_HOOK_LIST = [
    (init_default_settings_file, [APP_DATA / "settings.default.toml"])
]


def use_hooks():
    logger.debug("Firing init hooks")
    for (hook, args) in USE_HOOK_LIST:
        state = hook(*args)
        logger.debug(f"Hook -> {state}: {hook}, {args}")
