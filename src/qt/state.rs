use crate::utils::globals::Globals;
use crate::utils::persist::StateFile;
use crate::{ColumnSort, HeaderColumn, SongField};
use num_traits::FromPrimitive;
use qtbridge::qobject;
use std::pin::Pin;
use std::str::FromStr;

#[qobject(Singleton, Base = QListModel)]
mod model {

    use crate::{ColumnSort, HeaderColumn, QSong, SongField};
    use qtbridge::{QListModel, QListModelBase};

    #[derive(Default)]
    pub struct QState {
        pub active_group: String,
        pub column_sort: ColumnSort,
        pub(crate) data: Vec<HeaderColumn>,
    }

    impl QListModel for QState {
        type Item = HeaderColumn;

        fn len(&self) -> usize {
            self.data.len()
        }

        fn get(&self, index: usize) -> Option<&Self::Item> {
            self.data.get(index)
        }
    }

    impl QState {}

    impl Default for QState {
        fn default() -> Self {
            let state = StateFile::load();
            let active_group = &state.active_group.to_string();

            tracing::debug!("Loaded state file: {:?}", state);

            Self {
                active_group: String::from(active_group),
                header_columns: state.header_columns,
                column_sort: state.column_sort,
            }
        }
    }
}

pub struct QState {
    pub active_group: String,
    pub column_sort: ColumnSort,
    pub(crate) header_columns: Vec<HeaderColumn>,
}

#[qobject(Singleton)]
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
