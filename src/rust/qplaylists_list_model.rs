#[cxx_qt::bridge]
mod qobject {
    extern "C++Qt" {
        include!(<QAbstractListModel>);
        #[qobject]
        type QAbstractListModel;
    }

    extern "C++" {
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;

        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        //include!("cxx-qt-lib/qbytearray.h");
        //type QByteArray = cxx_qt_lib::QByteArray;

        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;

        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
    }

    #[qenum(QPlaylistsListModel)]
    enum QPlaylistsListRoles {
        Name,
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[base = QAbstractListModel]
        #[qproperty(QString, filter, READ = get_filter, WRITE = set_filter, NOTIFY = update)]
        type QPlaylistsListModel = super::PlaylistsListModel;

        #[qsignal]
        #[cxx_name = "update"]
        fn update(self: Pin<&mut QPlaylistsListModel>);

        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &QPlaylistsListModel) -> QHash_i32_QByteArray;

        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &QPlaylistsListModel, index: &QModelIndex) -> i32;

        #[cxx_override]
        fn data(self: &QPlaylistsListModel, index: &QModelIndex, role: i32) -> QVariant;

        #[qinvokable]
        fn get_filter(self: &QPlaylistsListModel) -> QString;

        #[qinvokable]
        fn set_filter(self: Pin<&mut QPlaylistsListModel>, value: QString);
    }
}

use crate::rust::entities::{QSong, SongField};
use crate::rust::settings::Settings;
use bincode::config;
use bincode::serde::{decode_from_slice, encode_to_vec};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use num_traits::FromPrimitive;
use serde;

use qobject::*;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PlaylistsListModel {
    pub filter: String,
    //pub queue: Vec<QSong>,
}

impl qobject::QPlaylistsListModel {
    pub fn role_names(&self) -> QHash_i32_QByteArray {
        let mut roles = QHash_i32_QByteArray::default();
        roles.insert(QPlaylistsListRoles::Name.repr, "name".into());
        roles
    }

    pub fn row_count(&self, _index: &QModelIndex) -> i32 {
        let settings = Settings::load();
        settings.search_groups.len() as i32
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let settings = Settings::load();
        let role = QPlaylistsListRoles { repr: role };
        let sg = settings.search_groups.get(index.row() as usize);
        let sg_name = QString::from(&sg.expect("asdgasdfg").to_string());
        match role {
            QPlaylistsListRoles::Name => (&sg_name).into(),
            _ => QVariant::default(),
        }
    }

    //fn set_queue(self: Pin<&mut QPlaylistsListModel>, value: QByteArray) {
    //}

    fn get_filter(self: &QPlaylistsListModel) -> QString {
        QString::from(&self.filter)
    }

    fn set_filter(mut self: Pin<&mut QPlaylistsListModel>, value: QString) {
        self.as_mut().rust_mut().filter = value.into();
    }
}

impl Drop for PlaylistsListModel {
    fn drop(&mut self) {
        let db = match sled::open("db") {
            Ok(v) => v,
            Err(e) => {
                panic!("Failed to open/create state db {}", e);
            }
        };
        let bcode: &[u8] = &encode_to_vec(self, config::standard()).unwrap();
        let _ = db.insert(b"playlists_list_model", bcode);
    }
}

impl Default for PlaylistsListModel {
    fn default() -> Self {
        let db = match sled::open("db") {
            Ok(v) => v,
            Err(e) => {
                panic!("Failed to open/create state db {}", e);
            }
        };

        match db.get(b"playlists_list_model").unwrap() {
            Some(val) => {
                let (val, _): (Self, usize) =
                    decode_from_slice(val.as_ref(), config::standard()).unwrap();
                val
            }
            None => Self {
                filter: String::default(),
                //queue: Vec::default(),
            },
        }
    }
}
