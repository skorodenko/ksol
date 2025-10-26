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
        type Orientation = crate::rust::qt::Orientation;
    }

    #[qenum(QPlaylistModel)]
    enum QPlaylistRoles {
        SongId,
        SongDisplay,
        SongActive,
        ColumnWidth,
        ColumnName,
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[base = QAbstractTableModel]
        #[qproperty(QString, filter, READ = get_filter, WRITE = set_filter, NOTIFY = update_filter)]
        #[qproperty(u64, active_song_id, cxx_name="activeSongId", READ, WRITE, NOTIFY = update_info)]
        #[qproperty(usize, last_visible_column, cxx_name="lastVisibleColumn", READ = get_last_visible_column, NOTIFY = update_header)]
        #[qproperty(QString, activeSongTitle, READ = get_active_song_title, NOTIFY = update_info)]
        #[qproperty(QString, activeSongArtist, READ = get_active_song_artist, NOTIFY = update_info)]
        #[qproperty(i32, sortOrder, READ = get_sort_order, NOTIFY = update_sort)]
        #[qproperty(i32, sortColumn, READ = get_sort_column, NOTIFY = update_sort)]
        type QPlaylistModel = super::PlaylistModel;

        #[qsignal]
        #[cxx_name = "updateFilter"]
        fn update_filter(self: Pin<&mut QPlaylistModel>);

        #[qsignal]
        #[cxx_name = "updateInfo"]
        fn update_info(self: Pin<&mut QPlaylistModel>);

        #[qsignal]
        #[cxx_name = "updateHeader"]
        fn update_header(self: Pin<&mut QPlaylistModel>);

        #[qsignal]
        #[cxx_name = "updateSort"]
        fn update_sort(self: Pin<&mut QPlaylistModel>);

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
        #[cxx_virtual]
        fn sort(self: Pin<&mut QPlaylistModel>, column: i32);

        #[qinvokable]
        #[cxx_virtual]
        #[cxx_name = "setQueue"]
        fn set_queue(self: Pin<&mut QPlaylistModel>, value: QByteArray);

        #[qinvokable]
        fn get_filter(self: &QPlaylistModel) -> QString;

        #[qinvokable]
        fn set_filter(self: Pin<&mut QPlaylistModel>, value: QString);

        #[qinvokable]
        fn get_active_song_title(self: &QPlaylistModel) -> QString;

        #[qinvokable]
        fn get_active_song_artist(self: &QPlaylistModel) -> QString;

        #[qinvokable]
        fn get_sort_order(self: &QPlaylistModel) -> i32;

        #[qinvokable]
        fn get_sort_column(self: &QPlaylistModel) -> i32;

        #[qinvokable]
        fn get_last_visible_column(self: &QPlaylistModel) -> usize;

        #[qinvokable]
        #[cxx_name = "updateColumnWidth"]
        pub fn update_column_width(self: Pin<&mut QPlaylistModel>, section: i32, width: f64);

        #[inherit]
        #[cxx_name = "headerDataChanged"]
        fn header_data_changed(
            self: Pin<&mut QPlaylistModel>,
            orientation: Orientation,
            start: i32,
            end: i32,
        );

        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut QPlaylistModel>);

        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut QPlaylistModel>);

        #[inherit]
        #[cxx_name = "layoutAboutToBeChanged"]
        fn layout_about_to_be_changed(self: Pin<&mut QPlaylistModel>);

        #[inherit]
        #[cxx_name = "layoutChanged"]
        fn layout_changed(self: Pin<&mut QPlaylistModel>);
    }

    impl cxx_qt::Initialize for QPlaylistModel {}
    impl cxx_qt::Threading for QPlaylistModel {}
}

use qobject::*;

use crate::rust::entities::{ColumnSort, QSong, SongField};
use bincode::config;
use bincode::serde::{decode_from_slice, encode_to_vec};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use num_traits::{FromPrimitive, ToPrimitive};
use regex::RegexBuilder;
use strum::IntoEnumIterator;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PlaylistModel {
    pub filter: String,
    pub queue: Vec<QSong>,
    queue_proxy: Vec<QSong>,
    pub column_width: Vec<f64>,
    pub active_song_id: u64,
    pub column_sort: ColumnSort,
}

impl qobject::QPlaylistModel {
    pub fn role_names(&self) -> QHash_i32_QByteArray {
        let mut roles = QHash_i32_QByteArray::default();
        roles.insert(QPlaylistRoles::SongId.repr, "songId".into());
        roles.insert(QPlaylistRoles::SongDisplay.repr, "songDisplay".into());
        roles.insert(QPlaylistRoles::SongActive.repr, "songActive".into());
        roles.insert(QPlaylistRoles::ColumnName.repr, "columnName".into());
        roles.insert(QPlaylistRoles::ColumnWidth.repr, "columnWidth".into());
        roles
    }

    pub fn row_count(&self, _index: &QModelIndex) -> i32 {
        self.queue_proxy.len() as i32
    }

    pub fn column_count(&self, _index: &QModelIndex) -> i32 {
        self.column_width.len() as i32
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let role = QPlaylistRoles { repr: role };
        match role {
            QPlaylistRoles::SongDisplay => {
                let row = index.row() as usize;
                let column = index.column();
                let sf = SongField::from_i32(column).unwrap();
                let qsong: QSong = self.queue_proxy.get(row).unwrap().clone();
                let field: String = match sf {
                    SongField::Track => format!("{}", qsong.track),
                    SongField::Title => qsong.title,
                    SongField::Artist => qsong.artist,
                    SongField::Album => qsong.album,
                    SongField::Date => qsong.date.to_string(),
                    SongField::Genre => qsong.genre,
                    SongField::Disc => format!("{}", qsong.disc),
                    SongField::Composer => qsong.composer,
                    SongField::Albumartist => String::default(),
                    SongField::File => qsong.file,
                    SongField::Format => qsong.format,
                    SongField::Lastmodified => qsong.lastmodified,
                    SongField::Duration => format!(
                        "{:0>2}:{:0>2}",
                        qsong.duration.as_secs() / 60,
                        qsong.duration.as_secs() % 60
                    ),
                    SongField::Directory => qsong.directory,
                };
                QVariant::from(&QString::from(field))
            }
            QPlaylistRoles::SongId => {
                let row = index.row() as usize;
                let qsong: QSong = self.queue_proxy.get(row).unwrap().clone();
                QVariant::from(&qsong.id)
            }
            _ => QVariant::default(),
        }
    }

    pub fn header_data(&self, section: i32, orientation: Orientation, role: i32) -> QVariant {
        let role = QPlaylistRoles { repr: role };
        match role {
            QPlaylistRoles::ColumnName if orientation == Orientation::Horizontal => {
                let sf = SongField::from_i32(section).unwrap();
                QVariant::from(&QString::from(format!("{}", sf)))
            }
            QPlaylistRoles::ColumnWidth => {
                let column = section as usize;
                QVariant::from(&self.column_width[column])
            }
            _ => QVariant::default(),
        }
    }

    pub fn sort(mut self: Pin<&mut QPlaylistModel>, column: i32) {
        let column = SongField::from_i32(column).unwrap();
        match self.column_sort {
            ColumnSort::Inactive => {
                self.as_mut().rust_mut().column_sort = ColumnSort::Ascending(column);
            }
            ColumnSort::Ascending(old_column) => {
                if old_column != column {
                    self.as_mut().rust_mut().column_sort = ColumnSort::Ascending(column);
                } else {
                    self.as_mut().rust_mut().column_sort = ColumnSort::Descending(column);
                }
            }
            ColumnSort::Descending(_) => {
                self.as_mut().rust_mut().column_sort = ColumnSort::Ascending(column);
            }
        };
        self.update_sort();
    }

    pub fn update_column_width(mut self: Pin<&mut QPlaylistModel>, section: i32, width: f64) {
        self.as_mut().rust_mut().column_width[section as usize] = width;
        let column_count = self.as_mut().rust_mut().column_width.len() as i32;
        self.header_data_changed(Orientation::Horizontal, 0, column_count);
    }

    fn set_queue(mut self: Pin<&mut QPlaylistModel>, value: QByteArray) {
        let (value, _): (Vec<QSong>, usize) =
            decode_from_slice(value.as_slice(), config::standard()).unwrap();
        self.as_mut().begin_reset_model();
        self.as_mut().rust_mut().queue = value.clone();
        self.as_mut().rust_mut().queue_proxy = value;
        self.as_mut().end_reset_model();
        self.as_mut().update_filter();
    }

    fn get_filter(self: &QPlaylistModel) -> QString {
        QString::from(&self.filter)
    }

    fn set_filter(mut self: Pin<&mut QPlaylistModel>, value: QString) {
        self.as_mut().rust_mut().filter = value.into();
        self.as_mut().update_filter();
    }

    fn get_active_song_title(self: &QPlaylistModel) -> QString {
        let song = &self.queue.iter().find(|&x| x.id == self.active_song_id);
        match song {
            Some(v) => QString::from(&v.title),
            None => QString::from("Title"),
        }
    }

    fn get_active_song_artist(self: &QPlaylistModel) -> QString {
        let song = &self.queue.iter().find(|&x| x.id == self.active_song_id);
        match song {
            Some(v) => QString::from(&v.artist),
            None => QString::from("Artist"),
        }
    }

    fn get_sort_order(self: &QPlaylistModel) -> i32 {
        match self.column_sort {
            ColumnSort::Inactive => 0,
            ColumnSort::Ascending(_) => 1,
            ColumnSort::Descending(_) => -1,
        }
    }

    fn get_sort_column(self: &QPlaylistModel) -> i32 {
        match self.column_sort {
            ColumnSort::Inactive => -1,
            ColumnSort::Ascending(col) => col.to_i32().unwrap_or(-1),
            ColumnSort::Descending(col) => col.to_i32().unwrap_or(-1),
        }
    }

    fn get_last_visible_column(self: &QPlaylistModel) -> usize {
        self.column_width
            .iter()
            .rposition(|&x| x != 0.0)
            .unwrap_or(0)
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
        self.active_song_id = 0;
        self.queue = vec![];
        self.queue_proxy = vec![];
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
                column_width: SongField::iter().map(|_| 1_f64 / 14_f64).collect(), //FIX calculated max enum
                active_song_id: 0,
                column_sort: ColumnSort::Ascending(SongField::Track),
            },
        }
    }
}

impl cxx_qt::Initialize for qobject::QPlaylistModel {
    fn initialize(self: Pin<&mut Self>) {
        self.on_update_filter(|mut qobject| {
            let queue = qobject.as_ref().rust().queue.clone();
            let filter = qobject.as_ref().rust().filter.clone();
            let pattern = RegexBuilder::new(&filter)
                .case_insensitive(true)
                .build()
                .unwrap();
            qobject.as_mut().layout_about_to_be_changed();
            qobject.as_mut().rust_mut().queue_proxy = queue
                .clone()
                .into_iter()
                .filter(|x| pattern.is_match(&x.title) || pattern.is_match(&x.artist))
                .collect();
            qobject.as_mut().layout_changed();
        })
        .release();
    }
}
