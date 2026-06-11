use qobject::*;

#[cxx_qt::bridge]
mod qobject {
    extern "C++" {
        include!(<QAbstractListModel>);
        type QAbstractListModel;

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
    enum QPlaylistsGroupRoles {
        Name,
        Value,
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[base = QAbstractListModel]
        type QPlaylistsGroupModel = super::PlaylistsGroupModel;

        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &QPlaylistsGroupModel) -> QHash_i32_QByteArray;

        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &QPlaylistsGroupModel, index: &QModelIndex) -> i32;

        #[cxx_override]
        fn data(self: &QPlaylistsGroupModel, index: &QModelIndex, role: i32) -> QVariant;
    }
}

use crate::SongField;
use core::pin::Pin;
use num_traits::{FromPrimitive, ToPrimitive};
use std::sync::Arc;

#[derive(Default)]
pub struct PlaylistsGroupModel {}

impl qobject::QPlaylistsGroupModel {
    pub fn role_names(&self) -> QHash_i32_QByteArray {
        let mut roles = QHash_i32_QByteArray::default();
        roles.insert(QPlaylistsGroupRoles::Name.repr, "name".into());
        roles.insert(QPlaylistsGroupRoles::Value.repr, "value".into());
        roles
    }

    pub fn row_count(&self, _index: &QModelIndex) -> i32 {
        14
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let role = QPlaylistsGroupRoles { repr: role };
        let Ok(row) = usize::try_from(index.row()) else {
            return QVariant::default();
        };
        match role {
            //            QPlaylistsGroupRoles::Name => {
            //                if let Some(sg) = settings.search_groups.get(row) {
            //                    let sg_name = QString::from(sg.to_string());
            //                    QVariant::from(&sg_name)
            //                } else {
            //                    QVariant::default()
            //                }
            //            }
            //            QPlaylistsGroupRoles::Value => {
            //                if let Some(sg) = settings.search_groups.get(row) {
            //                    let sg_value = sg.to_i32().expect("Failed to cast to i32");
            //                    QVariant::from(&sg_value)
            //                } else {
            //                    QVariant::default()
            //                }
            //            }
            _ => QVariant::default(),
        }
    }
}
