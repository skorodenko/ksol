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

    #[qenum(QPlaylistsGroupModel)]
    enum Roles {
        Name,
        Value,
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[base = QAbstractListModel]
        #[qproperty(i32, activeGroup, READ = get_active_group, WRITE = set_active_group, NOTIFY = active_group_changed)]
        type QPlaylistsGroupModel = super::PlaylistsGroupModel;

        #[qsignal]
        #[cxx_name = "activeGroupChanged"]
        fn active_group_changed(self: Pin<&mut QPlaylistsGroupModel>);

        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &QPlaylistsGroupModel) -> QHash_i32_QByteArray;

        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &QPlaylistsGroupModel, index: &QModelIndex) -> i32;

        #[cxx_override]
        fn data(self: &QPlaylistsGroupModel, index: &QModelIndex, role: i32) -> QVariant;

        #[qinvokable]
        fn get_active_group(self: &QPlaylistsGroupModel) -> i32;

        #[qinvokable]
        fn set_active_group(self: Pin<&mut QPlaylistsGroupModel>, value: &QVariant);
    }
}

use crate::rust::entities::SongField;
use crate::rust::settings::Settings;
use bincode::config;
use bincode::serde::{decode_from_slice, encode_to_vec};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use log::error;
use num_traits::FromPrimitive;
use serde;

use qobject::*;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PlaylistsGroupModel {
    pub active_group: SongField,
}

impl qobject::QPlaylistsGroupModel {
    pub fn role_names(&self) -> QHash_i32_QByteArray {
        let mut roles = QHash_i32_QByteArray::default();
        roles.insert(Roles::Name.repr, "name".into());
        roles.insert(Roles::Value.repr, "value".into());
        return roles;
    }

    pub fn row_count(&self, _index: &QModelIndex) -> i32 {
        let settings = Settings::load();
        return settings.search_groups.len() as i32;
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let settings = Settings::load();
        let role = Roles { repr: role };
        let sg = settings.search_groups.get(index.row() as usize);
        let sg_name = QString::from(&sg.expect("asdgasdfg").to_string());
        let sg_value = *sg.unwrap() as i32;
        return match role {
            Roles::Name => (&sg_name).into(),
            Roles::Value => (&sg_value).into(),
            _ => QVariant::default(),
        };
    }

    pub fn get_active_group(&self) -> i32 {
        self.active_group as i32
    }

    pub fn set_active_group(mut self: Pin<&mut Self>, variant: &QVariant) {
        if let Some(value) = variant.value::<i32>() {
            let value = SongField::from_i32(value).unwrap();
            self.as_mut().rust_mut().active_group = value;
            self.active_group_changed();
        } else {
            error!("Bad operation");
        }
    }
}

impl Drop for PlaylistsGroupModel {
    fn drop(&mut self) {
        let db = match sled::open("db") {
            Ok(v) => v,
            Err(e) => {
                panic!("Failed to open/create state db {}", e);
            }
        };
        let bcode: &[u8] = &encode_to_vec(self, config::standard()).unwrap();
        let _ = db.insert(b"playlists_group_model", bcode);
    }
}

impl Default for PlaylistsGroupModel {
    fn default() -> Self {
        let db = match sled::open("db") {
            Ok(v) => v,
            Err(e) => {
                panic!("Failed to open/create state db {}", e);
            }
        };

        match db.get(b"playlists_group_model").unwrap() {
            Some(val) => {
                let (val, _): (Self, usize) =
                    decode_from_slice(val.as_ref(), config::standard()).unwrap();
                val
            }
            None => Self {
                active_group: SongField::Directory,
            },
        }
    }
}
