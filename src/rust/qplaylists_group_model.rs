#[cxx_qt::bridge]
mod qobject {
    extern "C++Qt" {
        include!(<QAbstractListModel>);
        #[qobject]
        type QAbstractListModel;
    }

    #[namespace = "Qt"]
    extern "C++" {
        type ItemFlags = cxx_qt_lib::ItemFlags;
        type ItemFlag = cxx_qt_lib::ItemFlag;
    }

    extern "C++" {
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;

        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;

        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
    }

    #[qenum(QPlaylistsGroupModel)]
    enum Roles {
        Name,
        Value,
        ActiveGroup,
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[base = QAbstractListModel]
        type QPlaylistsGroupModel = super::PlaylistsGroupModel;

        #[cxx_override]
        #[rust_name = "role_names"]
        fn roleNames(self: &QPlaylistsGroupModel) -> QHash_i32_QByteArray;

        #[cxx_override]
        #[rust_name = "row_count"]
        fn rowCount(self: &QPlaylistsGroupModel, index: &QModelIndex) -> i32;

        #[cxx_override]
        fn data(self: &QPlaylistsGroupModel, index: &QModelIndex, role: i32) -> QVariant;

        #[cxx_override]
        fn flags(self: &QPlaylistsGroupModel, index: &QModelIndex) -> ItemFlags;

        #[qinvokable]
        #[rust_name = "set_active"]
        fn setActive(self: &QPlaylistsGroupModel, value: &QVariant);
    }
}

use crate::rust::entities::SongField;
use crate::rust::settings::Settings;
use crate::rust::state::State;
use core::pin::Pin;
use log::error;

use qobject::*;

#[derive(Default)]
pub struct PlaylistsGroupModel {}

impl qobject::QPlaylistsGroupModel {
    pub fn role_names(&self) -> QHash_i32_QByteArray {
        let mut roles = QHash_i32_QByteArray::default();
        roles.insert(Roles::Name.repr, "name".into());
        roles.insert(Roles::Value.repr, "value".into());
        roles.insert(Roles::ActiveGroup.repr, "activeGroup".into());
        return roles;
    }

    pub fn row_count(&self, _index: &QModelIndex) -> i32 {
        let settings = Settings::load();
        return settings.app.search_groups.len() as i32;
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let settings = Settings::load();
        let state = State::load();
        let role = Roles { repr: role };
        let sg = settings.app.search_groups.get(index.row() as usize);
        let sg_name = QString::from(&sg.expect("asdgasdfg").to_string());
        let sg_value = *sg.unwrap() as i32;
        let sg_active = state.group as i32;
        return match role {
            Roles::Name => (&sg_name).into(),
            Roles::Value => (&sg_value).into(),
            Roles::ActiveGroup => (&sg_active).into(),
            _ => QVariant::default(),
        };
    }

    pub fn flags(&self, _index: &QModelIndex) -> ItemFlags {
        return ItemFlag::ItemIsEditable.into();
    }

    pub fn set_active(&self, variant: &QVariant) {
        error!("TEST");
        //let &mut state = State::load();
        //        match role {
        //            Roles::ActiveGroup => {
        //                if let Some(value) = variant.value::<i32>() {
        //                    //self.set_boolean(boolean);
        //                    true
        //                } else {
        //                    false
        //                }
        //            },
        //            _ => { false }
        //        }
        //        if let (Some(value), role) = (variant.value::<i32>(), Roles::ActiveGroup) {
        //            //state.set_group(SongField::Directory);
        //            error!("Test");
        //            true
        //        } else {
        //            error!("Bad operation");
        //            false
        //        }
    }
}
