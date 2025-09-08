#[cxx::bridge(namespace = "Qt")]
mod ffi {
    #[repr(i32)]
    enum Orientation {
        Horizontal = 0x1,
        Vertical = 0x2,
    }

    unsafe extern "C++" {
        include!("cxx-qt-lib/qt.h");
        type Orientation;
    }
}

pub use ffi::{Orientation};

