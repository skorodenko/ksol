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

        include!("cxx-qt-lib/qbytearray.h");
        type QByteArray = cxx_qt_lib::QByteArray;

        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;

        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
    }

    #[qenum(QPlaylistsListModel)]
    enum QPlaylistsListRoles {
        Name,
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "layoutAboutToBeChanged"]
        fn layout_about_to_be_changed(self: Pin<&mut QPlaylistsListModel>);

        #[inherit]
        #[cxx_name = "layoutChanged"]
        fn layout_changed(self: Pin<&mut QPlaylistsListModel>);
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[base = QAbstractListModel]
        type QPlaylistsListModel = super::PlaylistsListModel;

        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &QPlaylistsListModel) -> QHash_i32_QByteArray;

        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &QPlaylistsListModel, index: &QModelIndex) -> i32;

        #[cxx_override]
        fn data(self: &QPlaylistsListModel, index: &QModelIndex, role: i32) -> QVariant;

        #[qinvokable]
        #[cxx_name = "setData"]
        fn set_data(self: Pin<&mut QPlaylistsListModel>, value: QByteArray);
    }
}

use core::pin::Pin;
use cxx_qt::CxxQtType;

#[derive(Default)]
pub struct PlaylistsListModel {
    data: Vec<String>,
}

impl qobject::QPlaylistsListModel {
    pub fn role_names(&self) -> QHash_i32_QByteArray {
        let mut roles = QHash_i32_QByteArray::default();
        roles.insert(QPlaylistsListRoles::Name.repr, "name".into());
        roles
    }

    pub fn row_count(&self, _index: &QModelIndex) -> i32 {
        self.data.len() as i32
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let role = QPlaylistsListRoles { repr: role };
        let Ok(row) = usize::try_from(index.row()) else {
            return QVariant::default();
        };
        match role {
            QPlaylistsListRoles::Name => (&QString::from(&self.data[row])).into(),
            _ => QVariant::default(),
        }
    }

    pub fn set_data(mut self: Pin<&mut QPlaylistsListModel>, value: QByteArray) {
        let value: Vec<String> = wincode::deserialize(value.as_slice()).unwrap();
        self.as_mut().layout_about_to_be_changed();
        self.as_mut().rust_mut().data = value;
        self.as_mut().layout_changed();
    }
}
