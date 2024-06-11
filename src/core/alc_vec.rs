use std::{
    mem,
    ops::{Index, IndexMut, RangeBounds},
    slice::{Iter, IterMut, SliceIndex},
    sync::Arc,
};

use rand::Rng;
use serde::{Deserialize, Serialize};

/// Thread safe lazily cloned vector. The vector may be cloned only when you
/// actually need to mutate its data.
pub enum AlcVec<T>
where
    T: Clone,
{
    /// There is static immutable reference to the playlist
    Static(Arc<Vec<T>>),
    /// There is owned vector with the playlist
    Dynamic(Vec<T>),
}

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

impl<T> AlcVec<T>
where
    T: Clone,
{
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the number of items in the playlist
    pub fn len(&self) -> usize {
        match self {
            Self::Static(s) => s.len(),
            Self::Dynamic(d) => d.len(),
        }
    }

    /// Returns iterator over the items.
    pub fn iter(&self) -> Iter<'_, T> {
        self[..].iter()
    }

    /// This may clone the vector.
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.vec_mut()[..].iter_mut()
    }

    /// Gets/Creates arc from the vector.
    pub fn make_arc(&mut self) -> Arc<Vec<T>> {
        self.make_static().clone()
    }

    /// This doesn't invoke clone on the vector.
    pub fn _vec(&self) -> &Vec<T> {
        match self {
            Self::Static(s) => s,
            Self::Dynamic(d) => d,
        }
    }

    /// This may clone the vector.
    pub fn vec_mut(&mut self) -> &mut Vec<T> {
        self.make_dynamic()
    }

    /// Creates lazy clone of the vector.
    pub fn clone(&mut self) -> Self {
        Self::Static(self.make_arc())
    }

    /// Replaces the given range of values with the given value. This may clone
    /// the vector, but it will not clone the vector if the iterator of new
    /// values is empty.
    pub fn splice<R, I>(&mut self, range: R, replace_with: I)
    where
        R: RangeBounds<usize>,
        I: IntoIterator<Item = T>,
    {
        let mut iter = replace_with.into_iter();
        let i = iter.next();
        if i.is_none() {
            return;
        }

        self.make_dynamic().splice(range, i.into_iter().chain(iter));
    }

    /// Mix new values after the given index. This may clone the vector, but it
    /// will not clone the vector if the iterator of new values is empty.
    pub fn mix_after<I>(&mut self, after: usize, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        let mut it = iter.into_iter();
        let i = it.next();
        if i.is_none() {
            return;
        }

        let v = self.make_dynamic();
        let mut rng = rand::thread_rng();
        for s in i.into_iter().chain(it) {
            let idx = rng.gen_range(after + 1..=v.len());
            let itm = mem::replace(&mut v[idx], s);
            v.push(itm);
        }
    }
}

impl<T> Default for AlcVec<T>
where
    T: Clone,
{
    fn default() -> Self {
        Self::Dynamic(Default::default())
    }
}

impl<T, I: SliceIndex<[T]>> Index<I> for AlcVec<T>
where
    T: Clone,
{
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        match self {
            Self::Static(s) => s.index(index),
            Self::Dynamic(d) => d.index(index),
        }
    }
}

impl<T, I: SliceIndex<[T]>> IndexMut<I> for AlcVec<T>
where
    T: Clone,
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.make_dynamic().index_mut(index)
    }
}

impl<T> From<Vec<T>> for AlcVec<T>
where
    T: Clone,
{
    fn from(value: Vec<T>) -> Self {
        Self::Dynamic(value)
    }
}

impl<T> From<Arc<Vec<T>>> for AlcVec<T>
where
    T: Clone,
{
    fn from(value: Arc<Vec<T>>) -> Self {
        Self::Static(value)
    }
}

impl<T> Extend<T> for AlcVec<T>
where
    T: Clone,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let mut it = iter.into_iter();
        let i = it.next();
        if i.is_some() {
            self.make_dynamic().extend(i.into_iter().chain(it))
        }
    }
}

impl<'a, T> Extend<&'a T> for AlcVec<T>
where
    T: Copy,
{
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        let mut it = iter.into_iter();
        let i = it.next();
        if i.is_some() {
            self.make_dynamic().extend(i.into_iter().chain(it))
        }
    }
}

impl<T> Serialize for AlcVec<T>
where
    T: Clone + Serialize,
{
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Static(s) => s.as_ref().serialize(serializer),
            Self::Dynamic(v) => v.serialize(serializer),
        }
    }
}

impl<'de, T> Deserialize<'de> for AlcVec<T>
where
    T: Clone + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::Dynamic(Vec::deserialize(deserializer)?))
    }

    fn deserialize_in_place<D>(
        deserializer: D,
        place: &mut Self,
    ) -> std::result::Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match place {
            Self::Static(_) => {
                *place = Self::Dynamic(Vec::deserialize(deserializer)?);
                Ok(())
            }
            Self::Dynamic(v) => Vec::deserialize_in_place(deserializer, v),
        }
    }
}

impl<T> From<AlcVec<T>> for Vec<T>
where
    T: Clone,
{
    fn from(mut value: AlcVec<T>) -> Self {
        value.make_dynamic();
        let AlcVec::Dynamic(d) = value else {
            panic!();
        };
        d
    }
}

impl<T> From<AlcVec<T>> for Arc<Vec<T>>
where
    T: Clone,
{
    fn from(mut value: AlcVec<T>) -> Self {
        value.make_arc()
    }
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

impl<T> AlcVec<T>
where
    T: Clone,
{
    /// Clones the playlist and creates owned vector
    #[inline]
    fn make_dynamic(&mut self) -> &mut Vec<T> {
        match self {
            Self::Dynamic(d) => d,
            Self::Static(_) => {
                let Self::Static(v) = mem::take(self) else {
                    panic!();
                };
                match Arc::try_unwrap(v) {
                    Ok(v) => {
                        *self = Self::Dynamic(v);
                        let Self::Dynamic(d) = self else {
                            panic!();
                        };
                        d
                    }
                    Err(a) => {
                        *self = Self::Dynamic(a.as_ref().clone());
                        let Self::Dynamic(d) = self else {
                            panic!();
                        };
                        d
                    }
                }
            }
        }
    }

    #[inline]
    fn make_static(&mut self) -> &Arc<Vec<T>> {
        match self {
            Self::Dynamic(d) => {
                *self = Self::Static(Arc::new(mem::take(d)));
                let Self::Static(s) = self else {
                    panic!();
                };
                s
            }
            Self::Static(s) => s,
        }
    }
}
