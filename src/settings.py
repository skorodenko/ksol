from pydantic import BaseModel, Field
from pydantic_settings import BaseSettings
from xdg_base_dirs import xdg_config_home, xdg_data_home


APP_CONFIG = xdg_config_home() / "ksol"
APP_DATA = xdg_data_home() / "ksol"


class MPDSettings(BaseModel):
    #socket: str | None = Field(None)
    socket: str = Field(str(APP_DATA / "mpd.socket"))
    native_socket: str = Field(str(APP_DATA / "mpd.socket"))
    native_config: str = Field(str(APP_CONFIG / "mpd.conf"))


class Settings(BaseSettings):
    mpd: MPDSettings = MPDSettings()


settings = Settings()
