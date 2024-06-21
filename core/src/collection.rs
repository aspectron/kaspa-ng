use crate::imports::*;
use kaspa_utils::hex::ToHex;
use std::fmt::Debug;
use std::hash::Hash;

pub struct Collection<Id, T> {
    list: Vec<T>,
    map: HashMap<Id, T>,
}

impl<Id, T> Default for Collection<Id, T>
where
    Id: Copy + Eq + Hash + Debug + ToHex,
    T: Clone + IdT<Id = Id> + Debug,
{
    fn default() -> Self {
        Self {
            list: Vec::new(),
            map: HashMap::new(),
        }
    }
}

impl<Id, T> Collection<Id, T>
where
    Id: Copy + Eq + Hash + Debug + ToHex,
    T: Clone + IdT<Id = Id> + Debug,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn push_unchecked(&mut self, v: T) {
        self.map.insert(*v.id(), v.clone());
        self.list.push(v);
    }

    pub fn replace_or_insert(&mut self, v: T) -> Option<T> {
        if self.map.insert(*v.id(), v.clone()).is_some() {
            let id = v.id();
            let index = self.list.iter().position(|item| item.id() == id).unwrap_or_else(|| {
                panic!("Collection::replace_or_insert(): failed to find index for id: {} while inserting: {:?}", id.to_hex(), v)
            });
            let t = std::mem::replace(&mut self.list[index], v);
            Some(t)
        } else {
            self.list.insert(0, v);
            None
        }
    }

    pub fn replace_or_push(&mut self, v: T) -> Option<T> {
        if self.map.insert(*v.id(), v.clone()).is_some() {
            let id = v.id();
            let index = self.list.iter().position(|item| item.id() == id).unwrap_or_else(|| {
                panic!("Collection::replace_or_insert(): failed to find index for id: {} while inserting: {:?}", id.to_hex(), v)
            });
            let t = std::mem::replace(&mut self.list[index], v);
            Some(t)
        } else {
            self.list.push(v);
            None
        }
    }

    pub fn first(&self) -> Option<&T> {
        self.list.first()
    }

    pub fn get(&self, id: &Id) -> Option<&T> {
        self.map.get(id)
    }

    pub fn pop(&mut self) -> Option<T> {
        if let Some(item) = self.list.pop() {
            self.map.remove(item.id());
            return Some(item);
        }
        None
    }

    pub fn list(&self) -> &Vec<T> {
        &self.list
    }

    pub fn list_mut(&mut self) -> &mut Vec<T> {
        &mut self.list
    }

    pub fn remove(&mut self, id: &Id) -> Option<T> {
        if let Some(v) = self.map.remove(id) {
            self.list.retain(|a| a.id() != id);
            Some(v)
        } else {
            None
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.list.iter()
    }

    pub fn reverse_iter(&self) -> impl Iterator<Item = &T> {
        self.list.iter().rev()
    }

    pub fn clear(&mut self) {
        self.list.clear();
        self.map.clear();
    }

    pub fn extend_unchecked(&mut self, iter: impl IntoIterator<Item = T>) {
        for v in iter {
            self.push_unchecked(v);
        }
    }

    pub fn load(&mut self, iter: impl IntoIterator<Item = T>) {
        self.clear();
        self.extend_unchecked(iter);
    }
}

impl<Id, T> From<Vec<T>> for Collection<Id, T>
where
    Id: Copy + Eq + Hash + Debug + ToHex,
    T: Clone + IdT<Id = Id>,
{
    fn from(list: Vec<T>) -> Self {
        Self {
            map: list
                .clone()
                .into_iter()
                .map(|v| (*v.id(), v))
                .collect::<HashMap<Id, T>>(),
            list,
        }
    }
}
