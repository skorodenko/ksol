import toml
from xdg_base_dirs import xdg_config_home, xdg_data_home
from pydantic import BaseModel, Field
from pydantic_settings import (
    BaseSettings,
    SettingsConfigDict,
    PydanticBaseSettingsSource,
    TomlConfigSettingsSource,
)


APP_CONFIG = xdg_config_home() / "ksol"
APP_DATA = xdg_data_home() / "ksol"


class MPDSettings(BaseModel):
    # socket: str | None = Field(None)
    socket: str = Field(str(APP_DATA / "mpd.socket"))
    native_socket: str = Field(str(APP_DATA / "mpd.socket"))
    native_config: str = Field(str(APP_CONFIG / "mpd.conf"))


class Settings(BaseSettings):
    mpd: MPDSettings = MPDSettings()
    model_config = SettingsConfigDict(toml_file=["./settings.default.toml"])
    
    def commit(self, output_file: str):
        parsed = self.model_dump()
        config_toml = toml.dumps(parsed)
        with open(output_file, "w") as f:
            f.write(config_toml)

    def rollback(self):
        ...

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

