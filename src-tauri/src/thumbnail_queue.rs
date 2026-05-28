use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct ThumbnailRuntime {
    running: Arc<Mutex<bool>>,
}

impl ThumbnailRuntime {
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }

    pub fn set_running(&self, value: bool) {
        *self.running.lock().unwrap() = value;
    }
}
