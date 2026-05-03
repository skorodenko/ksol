use crate::utils::settings::Settings;
use crate::{QSong, SongField};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use num_traits::FromPrimitive;
use qobject::*;
use regex;

#[derive(Default)]
pub struct PlaylistModel {
    pub filter: String,
    pub queue: Vec<QSong>,
    queue_proxy: Vec<usize>,
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
        self.queue_proxy.len() as i32
    }

    pub fn column_count(&self, _index: &QModelIndex) -> i32 {
        let settings = Settings::load().blocking_read();
        settings.column_width.len() as i32
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
                let index = self.queue_proxy[row];
                let qsong = &self.queue[index];
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
                    SongField::Duration => &format!(
                        "{:0>2}:{:0>2}",
                        qsong.duration / 60,
                        qsong.duration % 60
                    ),
                    SongField::Directory => &qsong.directory,
                };
                QVariant::from(&QString::from(field))
            }
            QPlaylistRoles::SongId => {
                let index = self.queue_proxy[row];
                QVariant::from(&self.queue[index].id)
            }
            _ => QVariant::default(),
        }
    }

    pub fn header_data(
        &self,
        section: i32,
        orientation: Orientation,
        role: i32,
    ) -> QVariant {
        let role = QPlaylistRoles { repr: role };
        match role {
            QPlaylistRoles::ColumnName
                if orientation == Orientation::Horizontal =>
            {
                let sf = SongField::from_i32(section).unwrap();
                QVariant::from(&QString::from(format!("{}", sf)))
            }
            _ => QVariant::default(),
        }
    }

    fn set_queue(mut self: Pin<&mut QPlaylistModel>, value: QByteArray) {
        let value: Vec<QSong> = wincode::deserialize(value.as_slice()).unwrap();
        self.as_mut().begin_reset_model();
        self.as_mut().rust_mut().queue = value;
        self.as_mut().rust_mut().queue_proxy = (0..self.queue.len()).collect();
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
}

impl cxx_qt::Initialize for qobject::QPlaylistModel {
    fn initialize(self: Pin<&mut Self>) {
        self.on_update_filter(|mut qobject| {
            let filter = regex::escape(&qobject.filter);
            let pattern = regex::RegexBuilder::new(&filter)
                .case_insensitive(true)
                .build()
                .unwrap();
            let proxy: Vec<usize> = (0..qobject.queue.len()).collect();
            qobject.as_mut().layout_about_to_be_changed();
            qobject.as_mut().rust_mut().queue_proxy = proxy
                .into_iter()
                .filter(|&x| {
                    pattern.is_match(&qobject.queue[x].title)
                        || pattern.is_match(&qobject.queue[x].artist)
                })
                .collect();
            qobject.as_mut().layout_changed();
        })
        .release();
    }
}

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
        type QHash_i32_QByteArray =
            cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;

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
        #[qproperty(QString, filter, READ = get_filter, WRITE = set_filter, NOTIFY = update_filter)]
        type QPlaylistModel = super::PlaylistModel;

        #[qsignal]
        #[cxx_name = "updateFilter"]
        fn update_filter(self: Pin<&mut QPlaylistModel>);

        #[qsignal]
        #[cxx_name = "updateHeader"]
        fn update_header(self: Pin<&mut QPlaylistModel>);

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
        fn data(
            self: &QPlaylistModel,
            index: &QModelIndex,
            role: i32,
        ) -> QVariant;

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
        #[cxx_name = "setQueue"]
        fn set_queue(self: Pin<&mut QPlaylistModel>, value: QByteArray);

        #[qinvokable]
        fn get_filter(self: &QPlaylistModel) -> QString;

        #[qinvokable]
        fn set_filter(self: Pin<&mut QPlaylistModel>, value: QString);

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
