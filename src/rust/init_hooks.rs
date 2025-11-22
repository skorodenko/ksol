use crate::rust::settings::{InternalSettings, Settings};
use std::{fs::File, io::Write};
use tinytemplate::TinyTemplate;
use tracing;

static MPD_CONFIG_TEMPLATE: &str = r#"
music_directory "{s.native_music_dir}"
sticker_file "{is.app_data_dir}mpd/sticker.sql"
bind_to_address "{is.app_data_dir}mpd/socket"
db_file "{is.app_cache_dir}mpd/db"
pid_file "{is.app_cache_dir}mpd/pid"
state_file "{is.app_cache_dir}mpd/state"
audio_buffer_size "8192"
log_file "/dev/null"
restore_paused "yes"
audio_output \{
    type "pipewire"
    name "Ksol"
}
"#;

#[derive(serde::Serialize, serde::Deserialize)]
struct TemplateData {
    pub s: Settings,
    pub is: InternalSettings,
}

pub fn init_native_mpd_config(s: Settings, is: InternalSettings) {
    tracing::debug!("Init native mpd config");
    let mut tt = TinyTemplate::new();
    tt.add_template("mpd_config", MPD_CONFIG_TEMPLATE).unwrap();
    let data = TemplateData { s, is };
    let render = tt.render("mpd_config", &data).unwrap();
    match File::create_new(data.is.native_config.clone()) {
        Ok(mut file) => {
            let _ = file.write_all(render.as_bytes());
        }
        Err(_) => tracing::debug!("MPD config file exists"),
    }
}
