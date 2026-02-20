use std::{ops::Deref, slice};

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OneOrVec<T> {
    One(T),
    Vec(Vec<T>),
}

impl<T> OneOrVec<T> {
    pub fn as_slice(&self) -> &[T] {
        match self {
            Self::One(o) => slice::from_ref(o),
            Self::Vec(v) => v.as_slice(),
        }
    }
}

impl<T> Deref for OneOrVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> IntoIterator for OneOrVec<T> {
    type Item = T;

    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        <Self as Into<Vec<T>>>::into(self).into_iter()
    }
}

impl<'a, T> IntoIterator for &'a OneOrVec<T> {
    type Item = &'a T;

    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl<T> From<OneOrVec<T>> for Vec<T> {
    fn from(value: OneOrVec<T>) -> Self {
        match value {
            OneOrVec::One(v) => vec![v],
            OneOrVec::Vec(v) => v,
        }
    }
}
