use qobject::*;

//use crate::utils::settings::{InternalSettings, Settings};
use crate::utils::state::Globals;
use crate::{ColumnSort, SongField};
use core::pin::Pin;
use num_traits::{FromPrimitive, ToPrimitive};
use std::net::TcpStream;
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use tracing;

#[derive(Default)]
pub struct Settings {
    init_wizard: bool,
    mpd_socket: QString,
    native_music_dir: QString,
    native_output_plugin: QString,
    background_blur: usize,
    background_colorization: usize,
}

impl qobject::QSettings {}

#[cxx_qt::bridge]
mod qobject {
    extern "C++" {
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;

        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(bool, init_wizard)]
        #[qproperty(QString, mpd_socket, cxx_name = "mpdSocket")]
        #[qproperty(QString, native_music_dir, cxx_name = "nativeMusicDir")]
        #[qproperty(QString, native_output_plugin)]
        #[qproperty(usize, background_blur)]
        #[qproperty(usize, background_colorization)]
        //        #[qproperty(i32, sortOrder, READ = get_sort_order, NOTIFY = update_sort_column)]
        //        #[qproperty(i32, sortColumn, READ = get_sort_column, NOTIFY = update_sort_column)]
        type QSettings = super::Settings;
    }
}
