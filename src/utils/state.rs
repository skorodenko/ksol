use super::misc::AtomicF64Vec;
use super::settings::Settings;
use crate::{ColumnSort, SongField};

use arc_swap::ArcSwap;
use std::sync::OnceLock;
use tokio::sync::RwLock;

static INSTANCE: OnceLock<Globals> = OnceLock::new();

pub struct Globals {
    pub state: State,
    pub settings: ArcSwap<Settings>,
    settings_edit: RwLock<Settings>,
}

impl Globals {
    pub fn get() -> &'static Self {
        INSTANCE.get_or_init(|| Self::load())
    }

    pub fn with_edit_setings<F>(&self, f: F) -> bool
    where
        F: FnOnce(&mut Settings),
    {
        let mut edit = self.settings_edit.blocking_write();
        let settings = self.settings.load();
        f(&mut *edit);
        *edit == **settings
    }

    pub fn with_state<F>(&self, f: F)
    where
        F: FnOnce(&State),
    {
        f(&self.state);
    }

    pub fn load() -> Self {
        let state = State::load();
        let settings = Settings::load();
        let settings_edit = settings.clone();

        Self {
            state,
            settings: ArcSwap::from_pointee(settings),
            settings_edit: RwLock::new(settings_edit),
        }
    }
}

pub struct State {
    pub column_width: AtomicF64Vec,
    pub column_sort: ArcSwap<ColumnSort>,
    pub active_group: ArcSwap<SongField>,
}

impl State {
    pub fn load() -> Self {
        let column_width = AtomicF64Vec::new(14, 1_f64 / 14_f64);
        let column_sort =
            ArcSwap::from_pointee(ColumnSort::Ascending(SongField::Track));
        let active_group = ArcSwap::from_pointee(SongField::Directory);

        Self {
            column_width,
            column_sort,
            active_group,
        }
    }
}
