use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct AtomicF64Vec {
    data: Arc<[AtomicU64]>,
}

impl AtomicF64Vec {
    pub fn new(len: usize, default_value: f64) -> Self {
        let bits = f64::to_bits(default_value);
        let data: Vec<AtomicU64> = (0..len).map(|_| AtomicU64::new(bits)).collect();
        Self { data: Arc::from(data) }
    }

    pub fn set(&self, index: usize, value: f64) {
        self.data[index].store(f64::to_bits(value), Ordering::Relaxed);
    }

    pub fn get(&self, index: usize) -> f64 {
        f64::from_bits(self.data[index].load(Ordering::Relaxed))
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
