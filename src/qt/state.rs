use cxx_qt::CxxQtType;
use qobject::*;

#[cxx_qt::bridge]
mod qobject {

    extern "C++" {
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;

        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

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
        #[qproperty(i32, sortOrder, READ = get_sort_order, NOTIFY = update_sort_column)]
        #[qproperty(i32, sortColumn, READ = get_sort_column, NOTIFY = update_sort_column)]
        #[qproperty(i32, activeGroup, READ = get_active_group, WRITE = set_active_group, NOTIFY = update_active_group)]
        type QState = super::State;

        #[qsignal]
        #[cxx_name = "updateSortColumn"]
        fn update_sort_column(self: Pin<&mut QState>);

        #[qsignal]
        #[cxx_name = "updateActiveGroup"]
        fn update_active_group(self: Pin<&mut QState>);

        #[qinvokable]
        fn get_sort_order(self: &QState) -> i32;

        #[qinvokable]
        fn get_sort_column(self: &QState) -> i32;

        #[qinvokable]
        fn get_active_group(self: &QState) -> i32;

        #[qinvokable]
        fn set_active_group(self: Pin<&mut QState>, value: i32);

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
    }
}

use crate::utils::persist::StateFile;
use crate::{ColumnSort, HeaderColumn, SongField};
use num_traits::{FromPrimitive, ToPrimitive};
use std::pin::Pin;

pub struct State {
    pub header_columns: Vec<HeaderColumn>,
    pub column_sort: ColumnSort,
    pub active_group: SongField,
}

impl QState {
    fn get_sort_order(&self) -> i32 {
        match self.column_sort {
            ColumnSort::Inactive => 0,
            ColumnSort::Ascending(_) => 1,
            ColumnSort::Descending(_) => -1,
        }
    }

    fn get_sort_column(&self) -> i32 {
        match self.column_sort {
            ColumnSort::Inactive => -1,
            ColumnSort::Ascending(col) => col.to_i32().unwrap_or(-1),
            ColumnSort::Descending(col) => col.to_i32().unwrap_or(-1),
        }
    }

    fn get_active_group(&self) -> i32 {
        self.active_group.to_i32().expect("Bad value")
    }

    fn set_active_group(mut self: Pin<&mut Self>, value: i32) {
        self.as_mut().rust_mut().active_group = SongField::from_i32(value).expect("Bad value");
    }

    fn get_column_width(&self, col: usize) -> f64 {
        let item = self.header_columns.get(col).expect("Out of bounds");
        item.width
    }

    fn set_column_width(mut self: Pin<&mut Self>, col: usize, width: f64) {
        let mut this = self.as_mut().rust_mut();
        if let Some(item) = this.header_columns.get_mut(col) {
            item.width = width;
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
                ColumnType::Width => item.width = value.value_or_default(),
                ColumnType::Hidden => item.hidden = value.value_or_default(),
                _ => unreachable!(),
            }
        }
    }
}

impl Default for State {
    fn default() -> Self {
        let state = StateFile::load();

        Self {
            header_columns: state.header_columns,
            column_sort: state.column_sort,
            active_group: state.active_group,
        }
    }
}
