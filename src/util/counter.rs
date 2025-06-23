use std::{cell::RefCell, rc::Rc};

pub struct Counter {
    // Use a vector instead of hashmap as lookup will be faster with low number of entries
    pub counts: Rc<RefCell<Vec<(String, usize)>>>,
    pub enabled: bool,
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

impl Counter {
    pub fn new() -> Self {
        Self {
            counts: Default::default(),
            enabled: true,
        }
    }

    /// Increment a value, or insert a new entry
    pub fn increment(&self, key: &str) {
        if !self.enabled {
            return;
        }

        let mut counts = self.counts.borrow_mut();
        let entry = counts.iter_mut().find(|(k, _)| k == key);
        if let Some((_, count)) = entry {
            *count += 1
        } else {
            counts.push((key.to_string(), 1));
        }
    }

    pub fn get_debug_strings(&self) -> Vec<String> {
        self.counts
            .borrow()
            .iter()
            .map(|(k, v)| format!("{k}: {v}"))
            .collect()
    }
}
