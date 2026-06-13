use crate::utils::globals::Globals;
use crate::utils::persist::PersistentConfig;
use std::fs;
use std::io::Write;
use tinytemplate::TinyTemplate;
use tracing;
use xdg::BaseDirectories;

static MPD_CONFIG_TEMPLATE: &str = r#"music_directory "{pc.native_music_dir}"
sticker_file "{gs.app_data_dir}mpd/sticker.sql"
bind_to_address "{gs.app_data_dir}mpd/socket"
db_file "{gs.app_cache_dir}mpd/db"
pid_file "{gs.app_cache_dir}mpd/pid"
state_file "{gs.app_cache_dir}mpd/state"
audio_buffer_size "8192"
log_file "/dev/null"
restore_paused "yes"
audio_output \{
    type "pipewire"
    name "Ksol"
    dop "yes"
}
"#;

#[derive(serde::Serialize)]
struct TemplateData<'a> {
    pub pc: &'a PersistentConfig,
    pub gs: &'a Globals,
}

pub fn init_native_mpd_config(pc: &PersistentConfig, gs: &Globals) {
    tracing::debug!("Init native mpd config");
    let mut tt = TinyTemplate::new();
    tt.add_template("mpd_config", MPD_CONFIG_TEMPLATE).unwrap();
    let data = TemplateData { pc, gs };
    let render = tt.render("mpd_config", &data).unwrap();
    match fs::File::create_new(data.gs.native_config.clone()) {
        Ok(mut file) => {
            let _ = file.write_all(render.as_bytes());
        }
        Err(_) => tracing::debug!("MPD config file exists"),
    }
}

pub fn init_dirs() {
    let xdg_dirs = BaseDirectories::with_prefix("ksol");
    let app_config = xdg_dirs.get_config_home().unwrap();
    let app_data = xdg_dirs.get_data_home().unwrap();
    let app_cache = xdg_dirs.get_cache_home().unwrap();

    // Create all directories at once to avoid duplicate calls
    let dirs_to_create = [
        &app_config,
        &app_data,
        &app_cache,
        &app_config.join("mpd"),
        &app_data.join("mpd"),
        &app_cache.join("mpd"),
    ];

    for dir in dirs_to_create {
        let _ = fs::create_dir_all(dir);
    }
}
