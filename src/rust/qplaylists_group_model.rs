#[cxx_qt::bridge]
mod qobject {

    unsafe extern "C++" {
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
    enum Roles {
        Name,
        Value,
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[base = QAbstractListModel]
        type QPlaylistsGroupModel = super::PlaylistsGroupModel;

        #[cxx_override]
        #[rust_name = "role_names"]
        fn roleNames(self: &QPlaylistsGroupModel) -> QHash_i32_QByteArray;

        #[cxx_override]
        #[rust_name = "row_count"]
        fn rowCount(self: &QPlaylistsGroupModel, index: &QModelIndex) -> i32;

        #[cxx_override]
        fn data(self: &QPlaylistsGroupModel, index: &QModelIndex, role: i32) -> QVariant;
    }
}

use crate::rust::settings::Settings;
use qobject::*;

#[derive(Default)]
pub struct PlaylistsGroupModel {}

impl qobject::QPlaylistsGroupModel {
    pub fn role_names(&self) -> QHash_i32_QByteArray {
        let mut roles = QHash_i32_QByteArray::default();
        roles.insert(Roles::Name.repr, "name".into());
        roles.insert(Roles::Value.repr, "value".into());
        return roles;
    }

    pub fn row_count(&self, _index: &QModelIndex) -> i32 {
        let settings = Settings::load();
        return settings.app.search_groups.len() as i32;
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let settings = Settings::load();
        let role = Roles { repr: role };
        let sg = settings.app.search_groups.get(index.row() as usize);
        let sg_name = QString::from(&sg.expect("asdgasdfg").to_string());
        let sg_value = *sg.unwrap() as i32;
        return match role {
            Roles::Name => (&sg_name).into(),
            Roles::Value => (&sg_value).into(),
            _ => QVariant::default(),
        };
    }
}
