/// The bridge definition for our QObject
#[cxx_qt::bridge]
pub mod qobject {

    extern "C++" {
        include!(<QAbstractListModel>);
        type QAbstractListModel;
        
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;

        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;

        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
    }

    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        type QPlaylistsGroupModel = super::PlaylistsGroupModel;

        #[qinvokable]
        #[cxx_override]
        fn data(self: &QPlaylistsGroupModel, index: &QModelIndex, role: i32) -> QVariant;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &QPlaylistsGroupModel) -> QHash_i32_QByteArray;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &QPlaylistsGroupModel, _parent: &QModelIndex) -> i32;
    }

    #[qenum(QPlaylistsGroupModel)]
    enum Roles {
        Name,
        Value,
    }
}

use crate::rust::settings::Settings;
use cxx_qt_lib::{QByteArray, QHash, QHashPair_i32_QByteArray, QModelIndex, QString, QVariant};
use log::debug;

#[derive(Default)]
pub struct PlaylistsGroupModel {}

impl qobject::QPlaylistsGroupModel {
    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut roles = QHash::<QHashPair_i32_QByteArray>::default();
        roles.insert(qobject::Roles::Name.repr, QByteArray::from("name"));
        roles.insert(qobject::Roles::Value.repr, QByteArray::from("value"));
        return roles;
    }

    pub fn row_count(&self, _index: &QModelIndex) -> i32 {
        let settings = Settings::load();
        return settings.app.search_groups.len() as i32;
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let settings = Settings::load();
        let role = qobject::Roles { repr: role };
        let sg = settings.app.search_groups.get(index.row() as usize);
        let sg_name = QString::from(sg.unwrap().to_string());
        let sg_value = *sg.unwrap() as i32;
        debug!("{} {}", sg_name, sg_value);
        return match role {
            qobject::Roles::Name => QVariant::from(&sg_name),
            qobject::Roles::Value => QVariant::from(&sg_value),
            _ => QVariant::default(),
        };
    }
}
