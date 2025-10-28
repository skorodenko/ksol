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
        #[qproperty(QString, filter, READ = get_filter, WRITE = set_filter, NOTIFY = update)]
        type QPlaylistsListModel = super::PlaylistsListModel;

        #[qsignal]
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

        #[qinvokable]
        #[cxx_name = "setQueue"]
        fn set_queue(self: Pin<&mut QPlaylistsListModel>, value: QByteArray);
    }

    impl cxx_qt::Initialize for QPlaylistsListModel {}
}

use bincode::config;
use bincode::serde::{decode_from_slice, encode_to_vec};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use regex;

use qobject::*;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PlaylistsListModel {
    pub filter: String,
    pub queue: Vec<String>,
    queue_proxy: Vec<String>,
}

impl qobject::QPlaylistsListModel {
    pub fn role_names(&self) -> QHash_i32_QByteArray {
        let mut roles = QHash_i32_QByteArray::default();
        roles.insert(QPlaylistsListRoles::Name.repr, "name".into());
        roles
    }

    pub fn row_count(&self, _index: &QModelIndex) -> i32 {
        self.queue_proxy.len() as i32
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let role = QPlaylistsListRoles { repr: role };
        let name = self.queue_proxy.get(index.row() as usize).unwrap();
        match role {
            QPlaylistsListRoles::Name => (&QString::from(name)).into(),
            _ => QVariant::default(),
        }
    }

    pub fn set_queue(mut self: Pin<&mut QPlaylistsListModel>, value: QByteArray) {
        let (value, _): (Vec<String>, usize) = decode_from_slice(value.as_slice(), config::standard()).unwrap();
        self.as_mut().rust_mut().queue = value;
        self.as_mut().update();
    }

    pub fn get_filter(self: &QPlaylistsListModel) -> QString {
        QString::from(&self.filter)
    }

    pub fn set_filter(mut self: Pin<&mut QPlaylistsListModel>, value: QString) {
        self.as_mut().rust_mut().filter = value.into();
        self.as_mut().update();
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
        self.queue = vec![];
        self.queue_proxy = vec![];
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
                let (val, _): (Self, usize) = decode_from_slice(val.as_ref(), config::standard()).unwrap();
                val
            }
            None => Self { filter: String::default(), queue: Vec::default(), queue_proxy: Vec::default() },
        }
    }
}

impl cxx_qt::Initialize for qobject::QPlaylistsListModel {
    fn initialize(self: Pin<&mut Self>) {
        self.on_update(|mut qobject| {
            let queue = qobject.as_ref().rust().queue.clone();
            let filter = qobject.as_ref().rust().filter.clone();
            let filter = regex::escape(&filter);
            let pattern = regex::RegexBuilder::new(&filter).case_insensitive(true).build().unwrap();
            qobject.as_mut().layout_about_to_be_changed();
            qobject.as_mut().rust_mut().queue_proxy = queue.into_iter().filter(|x| pattern.is_match(x)).collect();
            qobject.as_mut().layout_changed();
        })
        .release();
    }
}
