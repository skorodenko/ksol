use crate::rust::entities::SongField;
use once_cell::sync::OnceCell;
use bincode;
use sled;
use log::debug;

pub struct State {
    pub group: SongField,
}

impl State {
    pub fn get_group(&self) -> SongField {
        return self.group;
    }

    pub fn set_group(&mut self, new_value: SongField) {
        self.group = new_value;
    }

    pub fn load() -> &'static Self {
        static INSTANCE: OnceCell<State> = OnceCell::new();
        INSTANCE.get_or_init(State::default)
    }

    pub fn get_state_db() -> &'static sled::Db {
        static INSTANCE: OnceCell<sled::Db> = OnceCell::new();
        INSTANCE.get_or_init(State::init_state_db)
    }

    fn init_state_db() -> sled::Db {
        let db = match sled::open("db") {
            Ok(v) => v,
            Err(e) => {
                panic!("Failed to open/create state db {}", e);
            }
        };
        return db;
    }
}

impl Default for State {
    fn default() -> Self {
        let db = Self::get_state_db();

        let group = match db.get(b"group").unwrap() {
            Some(v) => {
                let (val, _bytes_read): (SongField, usize) = bincode::serde::decode_from_slice(v.as_ref(), bincode::config::standard()).unwrap();
                val
            },
            None => SongField::Directory,
        };

        Self { group: group }
    }
}
