import toml
from deepdiff import DeepDiff, Delta
from xdg_base_dirs import xdg_config_home, xdg_data_home
from pydantic import BaseModel
from pydantic_settings import (
    BaseSettings,
    PydanticBaseSettingsSource,
    TomlConfigSettingsSource,
)


from entities import PlaylistsGroup


APP_CONFIG = xdg_config_home() / "ksol"
APP_DATA = xdg_data_home() / "ksol"


SETTINGS_FIELS = [
    str(APP_DATA / "settings.default.toml"),
    str(APP_CONFIG / "settings.toml")
]


class MPDSettings(BaseModel):
    socket: str
    native_socket: str
    native_config: str

class AppSettings(BaseModel):
    disabled_groups: list[PlaylistsGroup]

class CoreSettings(BaseModel):
    config_location: str
    state_db_location: str

class Settings(BaseSettings):
    mpd: MPDSettings
    core: CoreSettings
    app: AppSettings

    class Config:  
        toml_file = SETTINGS_FIELS
        use_enum_values = True

    def commit(self):
        parsed_config = self.model_dump()
        with open(APP_DATA / "settings.default.toml", "r") as f:
            default_config = toml.loads(f.read())
        ddiff = DeepDiff(default_config, parsed_config, ignore_numeric_type_changes=True)
        delta = {} + Delta(ddiff, force=True)
        ddiff_config_toml = toml.dumps(delta)
        with open(self.core.config_location, "w") as f:
            f.write(ddiff_config_toml)

    def rollback(self):
        self.__init__()

    @classmethod
    def settings_customise_sources(
        cls,
        settings_cls: type[BaseSettings],
        init_settings: PydanticBaseSettingsSource,
        env_settings: PydanticBaseSettingsSource,
        dotenv_settings: PydanticBaseSettingsSource,
        file_secret_settings: PydanticBaseSettingsSource,
    ) -> tuple[PydanticBaseSettingsSource, ...]:
        return [TomlConfigSettingsSource(settings_cls)]


settings = Settings()

