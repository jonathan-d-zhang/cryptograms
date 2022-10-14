use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type StdoutMap = Arc<Mutex<HashMap<String, String>>>;

/// Abstraction over writing to static stdout map
///
/// Writes data when dropped.
pub struct Stdout {
    name: String,
    lines: Vec<String>,
    map: StdoutMap,
}

impl Stdout {
    pub fn new(name: String, map: StdoutMap) -> Self {
        Self {
            name,
            lines: Vec::new(),
            map,
        }
    }

    pub fn push(&mut self, msg: String) {
        self.lines.push(msg);
    }
}

impl Drop for Stdout {
    fn drop(&mut self) {
        self.map
            .lock()
            .unwrap()
            .insert(self.name.clone(), self.lines.join("\n"));
    }
}
