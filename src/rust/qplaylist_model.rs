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

        include!("cxx-qt-lib/qbytearray.h");
        type QByteArray = cxx_qt_lib::QByteArray;

        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;

        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
    }

    #[namespace = "Qt"]
    extern "C++" {
        include!("cxx-qt-lib/qt.h");
        type Orientation = crate::rust::qt::Orientation;
    }

    #[qenum(QPlaylistModel)]
    enum QPlaylistRoles {
        SongName,
        SongActive,
        ColumnWidth,
        ColumnName,
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut QPlaylistModel>);

        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut QPlaylistModel>);
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[base = QAbstractListModel]
        #[qproperty(QString, filter, READ = get_filter, WRITE = set_filter, NOTIFY = update)]
        type QPlaylistModel = super::PlaylistModel;

        #[qsignal]
        #[cxx_name = "update"]
        fn update(self: Pin<&mut QPlaylistModel>);

        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &QPlaylistModel) -> QHash_i32_QByteArray;

        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &QPlaylistModel, index: &QModelIndex) -> i32;

        #[cxx_override]
        #[cxx_name = "columnCount"]
        fn column_count(self: &QPlaylistModel, index: &QModelIndex) -> i32;

        #[cxx_override]
        fn data(self: &QPlaylistModel, index: &QModelIndex, role: i32) -> QVariant;

        #[cxx_override]
        #[cxx_name = "headerData"]
        fn header_data(
            self: &QPlaylistModel,
            section: i32,
            orientation: Orientation,
            role: i32,
        ) -> QVariant;

        #[qinvokable]
        #[cxx_name = "setQueue"]
        fn set_queue(self: Pin<&mut QPlaylistModel>, value: QByteArray);

        #[qinvokable]
        fn get_filter(self: &QPlaylistModel) -> QString;

        #[qinvokable]
        fn set_filter(self: Pin<&mut QPlaylistModel>, value: QString);
    }

    impl cxx_qt::Initialize for QPlaylistModel {}
}

use qobject::*;

use crate::rust::entities::{QSong, SongField};
use bincode::config;
use bincode::serde::{decode_from_slice, encode_to_vec};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use num_traits::FromPrimitive;
use regex::RegexBuilder;
use strum::IntoEnumIterator;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PlaylistModel {
    pub filter: String,
    pub queue: Vec<QSong>,
    queue_proxy: Vec<QSong>,
    pub column_width: Vec<f32>,
}

impl qobject::QPlaylistModel {
    pub fn role_names(&self) -> QHash_i32_QByteArray {
        let mut roles = QHash_i32_QByteArray::default();
        roles.insert(QPlaylistRoles::SongName.repr, "songName".into());
        roles.insert(QPlaylistRoles::SongActive.repr, "songActive".into());
        roles.insert(QPlaylistRoles::ColumnName.repr, "columnName".into());
        roles.insert(QPlaylistRoles::ColumnWidth.repr, "columnWidth".into());
        roles
    }

    pub fn row_count(&self, _index: &QModelIndex) -> i32 {
        self.queue_proxy.len() as i32
    }

    pub fn column_count(&self, _index: &QModelIndex) -> i32 {
        self.column_width
            .clone()
            .into_iter()
            .filter(|x| *x != 0.0)
            .count() as i32
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let role = QPlaylistRoles { repr: role };
        //let name = self.queue_proxy.get(index.row() as usize).unwrap();
        match role {
            _ => QVariant::from(&QString::from("TEST".to_string())),
        }
    }

    pub fn header_data(&self, section: i32, orientation: Orientation, role: i32) -> QVariant {
        let role = QPlaylistRoles { repr: role };
        //let name = self.queue_proxy.get(index.row() as usize).unwrap();
        match role {
            _ => QVariant::from(&QString::from("TEST".to_string())),
        }
    }

    pub fn set_queue(mut self: Pin<&mut QPlaylistModel>, value: QByteArray) {
        let (value, _): (Vec<QSong>, usize) =
            decode_from_slice(value.as_slice(), config::standard()).unwrap();
        self.as_mut().rust_mut().queue = value;
        //self.as_mut().update();
    }

    pub fn get_filter(self: &QPlaylistModel) -> QString {
        QString::from(&self.filter)
    }

    pub fn set_filter(mut self: Pin<&mut QPlaylistModel>, value: QString) {
        self.as_mut().rust_mut().filter = value.into();
        self.as_mut().update();
    }
}

impl Drop for PlaylistModel {
    fn drop(&mut self) {
        let db = match sled::open("db") {
            Ok(v) => v,
            Err(e) => {
                panic!("Failed to open/create state db {}", e);
            }
        };
        let bcode: &[u8] = &encode_to_vec(self, config::standard()).unwrap();
        let _ = db.insert(b"playlist_model", bcode);
    }
}

impl Default for PlaylistModel {
    fn default() -> Self {
        let db = match sled::open("db") {
            Ok(v) => v,
            Err(e) => {
                panic!("Failed to open/create state db {}", e);
            }
        };

        match db.get(b"playlist_model").unwrap() {
            Some(val) => {
                let (val, _): (Self, usize) =
                    decode_from_slice(val.as_ref(), config::standard()).unwrap();
                val
            }
            None => Self {
                filter: String::default(),
                queue: Vec::default(),
                queue_proxy: Vec::default(),
                column_width: SongField::iter().map(|_| 1.0).collect(),
            },
        }
    }
}

impl cxx_qt::Initialize for qobject::QPlaylistModel {
    fn initialize(self: Pin<&mut Self>) {
        self.on_update(|mut qobject| {
            let queue = qobject.as_ref().rust().queue.clone();
            //let filter = qobject.as_ref().rust().filter.clone();
            //let pattern = RegexBuilder::new(&filter)
            //    .case_insensitive(true)
            //    .build()
            //    .unwrap();
            qobject.as_mut().begin_reset_model();
            println!("{:?}", queue);
            qobject.as_mut().rust_mut().queue_proxy = queue;
            //qobject.as_mut().rust_mut().queue_proxy =
            //    queue.into_iter().filter(|x| pattern.is_match(x)).collect();
            qobject.as_mut().end_reset_model();
        })
        .release();
    }
}
