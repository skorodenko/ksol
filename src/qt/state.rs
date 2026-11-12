use crate::utils::globals::Globals;
use crate::utils::persist::StateFile;
use crate::{ColumnSort, HeaderColumn, QSong, SongField};
use num_traits::FromPrimitive;
use qtbridge::qobject;
use qtbridge::{QListModel, QListModelBase};
use serde_json::Value;
use std::pin::Pin;
use std::str::FromStr;

pub struct QState {
    pub active_group: String,
    pub column_sort: ColumnSort,
    pub(crate) header_columns: Vec<HeaderColumn>,
}

#[qobject(Singleton)]
impl QState {
    #[qsignal(qml_name = "updateSort")]
    pub fn update_sort(&mut self);

    #[qslot(qml_name = "setSort")]
    fn set_sort(&mut self, col: i32) {
        let col = SongField::from_i32(col).unwrap();
        let sort = match self.column_sort {
            ColumnSort::Inactive => ColumnSort::Descending(col),
            ColumnSort::Descending(field) if field == col => ColumnSort::Ascending(col),
            ColumnSort::Descending(field) if field != col => ColumnSort::Descending(col),
            ColumnSort::Ascending(field) if field == col => ColumnSort::Descending(col),
            ColumnSort::Ascending(field) if field != col => ColumnSort::Descending(col),
            _ => unreachable!(),
        };
        self.column_sort = sort;
        self.update_sort();
    }

    #[qslot(qml_name = "getSortColumn")]
    fn get_sort_column(&self) -> i32 {
        match self.column_sort {
            ColumnSort::Inactive => -1,
            ColumnSort::Ascending(song_field) => song_field as i32,
            ColumnSort::Descending(song_field) => song_field as i32,
        }
    }

    #[qslot(qml_name = "getSortOrder")]
    fn get_sort_order(&self) -> i32 {
        match self.column_sort {
            ColumnSort::Inactive => 0,
            ColumnSort::Ascending(_) => 1 as i32,
            ColumnSort::Descending(_) => -1 as i32,
        }
    }

    #[qslot(qml_name = "getHeaderColumn")]
    fn get_header_column(&self, col: usize) -> Value {
        let item = self.header_columns.get(col).expect("Out of bounds");
        serde_json::to_value(item).unwrap_or_else(|e| {
            //LOG
            Value::Null
        })
    }

    #[qslot(qml_name = "setHeaderColumn")]
    fn set_header_column(&mut self, col: usize, value: Value) {
        match serde_json::from_value(value) {
            Ok(new_data) => self.header_columns[col] = new_data,
            Err(e) => tracing::error!("Failed to update header_column: {}", e),
        }
    }

    #[qslot(qml_name = "mpdBinaryAvailable")]
    fn mpd_binary_available(&self) -> bool {
        let globals = Globals::get();
        globals.mpd_binary.exists()
    }

    #[qslot]
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

impl Default for QState {
    fn default() -> Self {
        let state = StateFile::load();
        let active_group = &state.active_group.to_string();

        tracing::debug!("Loaded state file: {:?}", state);

        Self {
            header_columns: state.header_columns,
            active_group: String::from(active_group),
            column_sort: state.column_sort,
        }
    }
}
