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
        #[qproperty(i32, activeGroup, READ = get_active_group, WRITE = set_active_group, NOTIFY = update)]
        //#[qproperty(QString, filter, READ = get_filter, WRITE = set_filter, NOTIFY = active_group_changed)]
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
        fn get_active_group(self: &QPlaylistsListModel) -> i32;

        #[qinvokable]
        fn set_active_group(self: Pin<&mut QPlaylistsListModel>, value: i32);
    }
}

use crate::rust::entities::SongField;
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
    pub active_group: SongField,
    pub filter: String,
}

impl qobject::QPlaylistsListModel {
    pub fn role_names(&self) -> QHash_i32_QByteArray {
        let mut roles = QHash_i32_QByteArray::default();
        roles.insert(QPlaylistsListRoles::Name.repr, "name".into());
        return roles;
    }

    pub fn row_count(&self, _index: &QModelIndex) -> i32 {
        let settings = Settings::load();
        return settings.search_groups.len() as i32;
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let settings = Settings::load();
        let role = QPlaylistsListRoles { repr: role };
        let sg = settings.search_groups.get(index.row() as usize);
        let sg_name = QString::from(&sg.expect("asdgasdfg").to_string());
        return match role {
            QPlaylistsListRoles::Name => (&sg_name).into(),
            _ => QVariant::default(),
        };
    }

    pub fn get_active_group(&self) -> i32 {
        self.active_group as i32
    }

    pub fn set_active_group(mut self: Pin<&mut Self>, value: i32) {
        let value = SongField::from_i32(value).unwrap();
        self.as_mut().rust_mut().active_group = value;
        self.update();
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
                active_group: SongField::Directory,
                filter: String::default(),
            },
        }
    }
}
