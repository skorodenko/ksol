use cxx_qt::CxxQtType;
use qobject::*;

#[cxx_qt::bridge]
mod qobject {

    extern "C++" {
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;

        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("cxx-qt-lib/qbytearray.h");
        type QByteArray = cxx_qt_lib::QByteArray;
    }

    #[qml_element]
    qnamespace!("ColumnType");

    #[qenum]
    #[namespace = "ColumnType"]
    enum ColumnType {
        Width,
        Hidden,
        Name,
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(QString, active_group, cxx_name = "activeGroup")]
        #[qproperty(i32, sortColumn, READ = get_sort_column, NOTIFY = update_sort)]
        #[qproperty(i32, sortOrder, READ = get_sort_order, NOTIFY = update_sort)]
        type QState = super::State;

        #[qinvokable]
        #[cxx_name = "setSort"]
        fn set_sort(self: Pin<&mut QState>, col: i32);

        #[qinvokable]
        #[cxx_name = "getSortColumn"]
        fn get_sort_column(self: &QState) -> i32;

        #[qinvokable]
        #[cxx_name = "getSortOrder"]
        fn get_sort_order(self: &QState) -> i32;

        #[qsignal]
        #[cxx_name = "updateSort"]
        fn update_sort(self: Pin<&mut QState>);

        #[qinvokable]
        #[cxx_name = "getHeaderColumn"]
        fn get_header_column(self: &QState, col: usize, col_type: ColumnType) -> QVariant;

        #[qinvokable]
        #[cxx_name = "setHeaderColumn"]
        fn set_header_column(
            self: Pin<&mut QState>,
            col: usize,
            col_type: ColumnType,
            value: QVariant,
        );

        #[qinvokable]
        #[cxx_name = "mpdBinaryAvailable"]
        fn mpd_binary_available(self: &QState) -> bool;

        #[qinvokable]
        fn dump(self: &QState);
    }
}

use crate::utils::globals::Globals;
use crate::utils::persist::StateFile;
use crate::{ColumnSort, HeaderColumn, SongField};
use num_traits::FromPrimitive;
use std::pin::Pin;
use std::str::FromStr;

pub struct State {
    pub active_group: QString,
    pub column_sort: ColumnSort,
    pub(crate) header_columns: Vec<HeaderColumn>,
}

impl QState {
    fn set_sort(mut self: Pin<&mut Self>, col: i32) {
        let col = SongField::from_i32(col).unwrap();
        let sort = match self.column_sort {
            ColumnSort::Inactive => ColumnSort::Descending(col),
            ColumnSort::Descending(field) if field == col => ColumnSort::Ascending(col),
            ColumnSort::Descending(field) if field != col => ColumnSort::Descending(col),
            ColumnSort::Ascending(field) if field == col => ColumnSort::Descending(col),
            ColumnSort::Ascending(field) if field != col => ColumnSort::Descending(col),
            _ => unreachable!(),
        };
        self.as_mut().rust_mut().column_sort = sort;
        self.update_sort();
    }

    fn get_sort_column(&self) -> i32 {
        match self.column_sort {
            ColumnSort::Inactive => -1,
            ColumnSort::Ascending(song_field) => song_field as i32,
            ColumnSort::Descending(song_field) => song_field as i32,
        }
    }

    fn get_sort_order(&self) -> i32 {
        match self.column_sort {
            ColumnSort::Inactive => 0,
            ColumnSort::Ascending(_) => 1 as i32,
            ColumnSort::Descending(_) => -1 as i32,
        }
    }

    fn get_header_column(&self, col: usize, col_type: ColumnType) -> QVariant {
        let item = self.header_columns.get(col).expect("Out of bounds");
        match col_type {
            ColumnType::Width => QVariant::from(&item.width),
            ColumnType::Hidden => QVariant::from(&item.hidden),
            ColumnType::Name => QVariant::from(&QString::from(item.name.clone())),
            _ => unreachable!(),
        }
    }

    fn set_header_column(
        mut self: Pin<&mut Self>,
        col: usize,
        col_type: ColumnType,
        value: QVariant,
    ) {
        let mut this = self.as_mut().rust_mut();
        if let Some(item) = this.header_columns.get_mut(col) {
            match col_type {
                ColumnType::Width => {
                    let width: f64 = value.value_or_default();
                    if !width.is_nan() {
                        item.width = width;
                    }
                }
                ColumnType::Hidden => item.hidden = value.value_or_default(),
                _ => unreachable!(),
            }
        }
    }

    fn mpd_binary_available(&self) -> bool {
        let globals = Globals::get();
        globals.mpd_binary.exists()
    }

    fn dump(&self) {
        let state_file = StateFile {
            header_columns: self.header_columns.clone(),
            column_sort: self.column_sort,
            active_group: SongField::from_str(self.active_group.to_string().as_str()).unwrap(),
        };
        tracing::debug!("Dumping state file: {:?}", state_file);
        state_file.dump();
    }
}

impl Default for State {
    fn default() -> Self {
        let state = StateFile::load();
        let active_group = &state.active_group.to_string();

        tracing::debug!("Loaded state file: {:?}", state);

        Self {
            active_group: QString::from(active_group),
            header_columns: state.header_columns,
            column_sort: state.column_sort,
        }
    }
}
