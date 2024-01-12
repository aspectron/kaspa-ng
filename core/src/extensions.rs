use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;

pub trait CompareHashMaps<K, V> {
    fn compare(&self, other: &HashMap<K, V>) -> bool;
}

impl<K, V> CompareHashMaps<K, V> for HashMap<K, V>
where
    K: Eq + Hash,
    V: Eq,
{
    fn compare(&self, other: &HashMap<K, V>) -> bool {
        if self.len() != other.len() {
            return false;
        }

        for (key, value) in self {
            if !other.contains_key(key) || other.get(key) != Some(value) {
                return false;
            }
        }

        true
    }
}
