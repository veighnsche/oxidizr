pub mod audit;
pub mod command;
pub mod worker;

// Distribution metadata used by experiments for compatibility gating
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Distribution {
    pub id: String,
    pub release: String,
}

use std::collections::HashSet;
use std::hash::Hash;

// Unordered equality for vectors (ignores ordering)
pub fn vecs_eq<T: Eq + Hash + Clone>(v1: &[T], v2: &[T]) -> bool {
    if v1.len() != v2.len() {
        return false;
    }
    let set: HashSet<T> = v1.iter().cloned().collect();
    v2.iter().all(|x| set.contains(x))
}

// test worker mock can be introduced later as needed
