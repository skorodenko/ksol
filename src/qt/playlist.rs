use qtbridge::qobject;

#[qobject(Base = QListModel)]
mod model {

    use crate::{QSong, SongField};
    use qtbridge::{QListModel, QListModelBase};

    #[derive(Default)]
    pub struct QPlaylistModel {
        data: Vec<QSong>,
    }

    impl QListModel for QPlaylistModel {
        type Item = QSong;

        fn len(&self) -> usize {
            self.data.len()
        }

        fn get(&self, index: usize) -> Option<&Self::Item> {
            self.data.get(index)
        }
    }

    impl QPlaylistModel {
        fn set_data(&mut self, value: Vec<QSong>) {
            self.data = value;
            self.reset();
        }
    }
}
