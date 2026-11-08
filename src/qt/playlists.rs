use qtbridge::qobject;

#[qobject(Base = QListModel)]
mod model {
    use qtbridge::{QListModel, QListModelBase};

    #[derive(Default)]
    pub struct QPlaylistsListModel {
        data: Vec<String>,
    }

    impl QListModel for QPlaylistsListModel {
        type Item = String;

        fn len(&self) -> usize {
            self.data.len()
        }

        fn get(&self, index: usize) -> Option<&Self::Item> {
            self.data.get(index)
        }
    }

    impl QPlaylistsListModel {
        fn set_data(&mut self, value: Vec<String>) {
            self.data = value;
            self.reset();
        }
    }
}
