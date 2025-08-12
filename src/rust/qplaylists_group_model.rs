/// The bridge definition for our QObject
#[cxx_qt::bridge]
pub mod qobject {

    extern "C++Qt" {
        include!(<QtCore/QAbstractListModel>);
        #[qobject]
        type QAbstractListModel;
    }

    extern "C++" {
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        type QPlaylistsGroupModel = super::PlaylistsGroupModel;
    }
    
    #[qenum(QPlaylistsGroupModel)]
    enum Roles {
        name,
        value
    }
}

use log::debug;
use core::pin::Pin;
use std::path::PathBuf;
use subprocess::Popen;
use crate::rust::entities::{SongField};
use crate::rust::settings::{settings};
use which::which;

/// The Rust struct for the QObject
#[derive(Default)]
pub struct PlaylistsGroupModel {
    mpd_server: Option<Popen>,
}

impl qobject::QPlaylistsGroupModel {
}
