//! String pool for interning strings.
//! This module provides a simple string pool to avoid duplicating strings in memory.

use std::collections::HashMap;

#[derive(Debug)]
pub struct StringPool {
    pool: HashMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub struct StrDescriptor(usize);

impl StrDescriptor {
    pub fn new(index: usize) -> Self {
        StrDescriptor(index)
    }

    pub fn index(&self) -> usize {
        self.0
    }
}

impl StringPool {
    pub fn new() -> Self {
        Self {
            pool: HashMap::new(),
        }
    }

    pub fn intern(&mut self, string: String) -> StrDescriptor {
        if let Some(&index) = self.pool.get(&string) {
            StrDescriptor(index)
        } else {
            let index = self.pool.len();
            self.pool.insert(string, index);
            StrDescriptor(index)
        }
    }

    pub fn get(&self, descriptor: StrDescriptor) -> Option<&String> {
        self.pool.iter().find_map(|(key, &value)| {
            if value == descriptor.index() {
                Some(key)
            } else {
                None
            }
        })
    }

    pub fn dump(&self) -> String {
        let mut strings = self.pool
            .iter()
            .collect::<Vec<_>>();
        strings.sort_by(|a, b| a.1.cmp(b.1));
        strings.iter()
            .map(|(str, idx)| format!("[{}]: {}", idx, str))
            .collect::<Vec<_>>()
            .join("\n")
    }
}