use qobject::*;

#[cxx_qt::bridge]
mod qobject {
    unsafe extern "C++" {
        include!(<QAbstractTableModel>);
        type QAbstractTableModel;

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
    unsafe extern "C++" {
        include!("cxx-qt-lib/qt.h");
        type Orientation = crate::qt::qt::Orientation;
    }

    #[qenum(QPlaylistModel)]
    enum QPlaylistRoles {
        SongId,
        SongDisplay,
        SongActive,
        ColumnName,
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[base = QAbstractTableModel]
        type QPlaylistModel = super::PlaylistModel;

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

        #[qinvokable]
        #[cxx_virtual]
        #[cxx_name = "setData"]
        fn set_data(self: Pin<&mut QPlaylistModel>, value: QByteArray);

        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut QPlaylistModel>);

        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut QPlaylistModel>);
    }

    impl cxx_qt::Threading for QPlaylistModel {}
}

use crate::{QSong, SongField};
use cxx_qt::CxxQtType;
use num_traits::FromPrimitive;
use std::pin::Pin;
use strum::EnumCount;

#[derive(Default)]
pub struct PlaylistModel {
    data: Vec<QSong>,
}

impl qobject::QPlaylistModel {
    pub fn role_names(&self) -> QHash_i32_QByteArray {
        let mut roles = QHash_i32_QByteArray::default();
        roles.insert(QPlaylistRoles::SongId.repr, "songId".into());
        roles.insert(QPlaylistRoles::SongDisplay.repr, "songDisplay".into());
        roles.insert(QPlaylistRoles::SongActive.repr, "songActive".into());
        roles.insert(QPlaylistRoles::ColumnName.repr, "columnName".into());
        roles
    }

    pub fn row_count(&self, _index: &QModelIndex) -> i32 {
        self.data.len() as i32
    }

    pub fn column_count(&self, _index: &QModelIndex) -> i32 {
        SongField::COUNT as i32
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let role = QPlaylistRoles { repr: role };
        let Ok(row) = usize::try_from(index.row()) else {
            return QVariant::default();
        };
        match role {
            QPlaylistRoles::SongDisplay => {
                let column = index.column();
                let sf = SongField::from_i32(column).unwrap();
                let qsong = &self.data[row];
                let field = match sf {
                    SongField::Track => &format!("{}", qsong.track),
                    SongField::Title => &qsong.title,
                    SongField::Artist => &qsong.artist,
                    SongField::Album => &qsong.album,
                    SongField::Date => &qsong.date.to_string(),
                    SongField::Genre => &qsong.genre,
                    SongField::Disc => &format!("{}", qsong.disc),
                    SongField::Composer => &qsong.composer,
                    SongField::Albumartist => &String::default(),
                    SongField::File => &qsong.file,
                    SongField::Format => &qsong.format,
                    SongField::Lastmodified => &qsong.lastmodified,
                    SongField::Duration => {
                        &format!("{:0>2}:{:0>2}", qsong.duration / 60, qsong.duration % 60)
                    }
                    SongField::Directory => &qsong.directory,
                };
                QVariant::from(&QString::from(field))
            }
            QPlaylistRoles::SongId => QVariant::from(&self.data[row].id),
            _ => QVariant::default(),
        }
    }

    fn set_data(mut self: Pin<&mut QPlaylistModel>, value: QByteArray) {
        let value: Vec<QSong> = wincode::deserialize(value.as_slice()).unwrap();
        self.as_mut().begin_reset_model();
        self.as_mut().rust_mut().data = value;
        self.as_mut().end_reset_model();
    }
}
