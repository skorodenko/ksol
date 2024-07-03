import toml
from xdg_base_dirs import xdg_config_home, xdg_data_home
from pydantic import BaseModel
from pydantic_settings import (
    BaseSettings,
    SettingsConfigDict,
    PydanticBaseSettingsSource,
    TomlConfigSettingsSource,
)


APP_CONFIG = xdg_config_home() / "ksol"
APP_DATA = xdg_data_home() / "ksol"


SETTINGS_FIELS = [str(APP_DATA / "settings.default.toml")]


class MPDSettings(BaseModel):
    socket: str
    native_socket: str
    native_config: str


class Settings(BaseSettings):
    mpd: MPDSettings
    model_config = SettingsConfigDict(toml_file=SETTINGS_FIELS)

    def commit(self, output_file: str):
        parsed = self.model_dump()
        config_toml = toml.dumps(parsed)
        with open(output_file, "w") as f:
            f.write(config_toml)

    def rollback(self): ...

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
