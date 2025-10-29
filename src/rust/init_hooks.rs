use crate::rust::settings::InternalSettings;
use std::{fs::File, io::Write};
use log;
use tinytemplate::TinyTemplate;

static MPD_CONFIG_TEMPLATE: &'static str = r#"
music_directory "{native_music_dir}"
playlist_directory "{app_data_dir}mpd/playlists"
sticker_file "{app_data_dir}mpd/sticker.sql"
bind_to_address "{app_data_dir}mpd/socket"
db_file "{app_data_dir}mpd/tag_cache"
pid_file "{app_data_dir}mpd/pid"
state_file "{app_data_dir}mpd/state"
log_file "/dev/null"
zeroconf_enabled "no"
metadata_to_use	"artist,album,title,track,name,genre,date,disc,albumartist,composer,musicbrainz_albumid,originaldate,albumartistsort,artistsort,albumsort"
audio_output \{
    type "pipewire"
    name "ksol"
}
mixer_type "software"
audio_buffer_size "8192"
filesystem_charset "UTF-8"
id3v1_encoding "UTF-8"
"#;

pub fn init_configs(defaults: &InternalSettings) {
    log::debug!("Init config files");
    let mut tt = TinyTemplate::new();
    tt.add_template("mpd_config", MPD_CONFIG_TEMPLATE).unwrap();
    let render = tt.render("mpd_config", defaults).unwrap();
    match File::create_new(defaults.native_config.clone()) {
        Ok(mut file) => {
            file.write_all(render.as_bytes());
        }
        Err(_) => log::debug!("MPD config file exists")
    }
}
