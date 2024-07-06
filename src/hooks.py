import toml
import hashlib
import logging
from pathlib import Path
from xdg_base_dirs import xdg_data_home, xdg_config_home, xdg_cache_home


logger = logging.getLogger("app")


APP_CONFIG = xdg_config_home() / "ksol"
APP_DATA = xdg_data_home() / "ksol"
APP_CACHE = xdg_cache_home() / "ksol"
DIRECTORIES = [
    APP_DATA / "mpd" / "playlists",
    APP_CACHE / "mpd",
]


def create_directory_structure(dirs: list[Path]):
    result = False
    for dir in dirs:
        if not dir.exists():
            dir.mkdir(parents = True)
            result = True
    return result


def init_default_settings_file(path: Path) -> bool:
    default_settings = {
        "mpd": {
            "socket": str(APP_DATA / "mpd.socket"),
            "native_socket": str(APP_DATA / "mpd.socket"),
            "native_config": str(APP_CONFIG / "mpd.conf"),
        },
        "core": {
            "config_location": str(APP_CONFIG / "settings.toml"),
            "state_db_location": str(APP_DATA / "state.db"),
        },
        "app": {
            "disabled_groups": [],
        },
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


def init_native_mpd_conf(path: Path) -> bool:
    default_config = {
        "music_directory": "/home/rinkuro/Music",
        "bind_to_address": str(APP_DATA / "mpd.socket"),
        "playlist_directory": str(APP_DATA / "mpd" / "playlists"),
        "sticker_file": str(APP_DATA / "mpd" / "sticker.sql"),
        "db_file": str(APP_CACHE / "mpd" / "mpd.db"),
        "pid_file": str(APP_CACHE / "mpd" / "pid"),
        "state_file": str(APP_CACHE / "mpd" / "state"),
        "log_file": "/dev/null",
        "zeroconf_enabled": "no",
        "audio_output": {
            "name": "MPD Piepwire output",
            "type": "pipewire",
        },
        "mixer_type": "software",
        "audio_buffer_size": "8192",
        "filesystem_charset": "UTF-8",
    }

    # Parse python dict to mpd.conf
    def parse(value):
        retval = []
        if not isinstance(value, dict):
            return f"{value}"
        for param, value in value.items():
            if isinstance(value, dict):
                retval.append(f"{param} {{\n{parse(value)}\n}}\n")
            else:
                retval.append(f"{param} \"{parse(value)}\"\n")
        return "".join(retval)

    default_config = parse(default_config)
    
    if not path.exists():
        with open(path, "w") as f:
            f.write(default_config)
        return True
    return False


USE_HOOK_LIST = [
    (create_directory_structure, [DIRECTORIES]),
    (init_default_settings_file, [APP_DATA / "settings.default.toml"]),
    (init_native_mpd_conf, [APP_CONFIG / "mpd.conf"]),
]


def use_hooks():
    logger.debug("Firing init hooks")
    for hook, args in USE_HOOK_LIST:
        state = hook(*args)
        logger.debug(f"Hook -> {state}: {hook}, {args}")


if __name__ == "__main__":
    use_hooks()
