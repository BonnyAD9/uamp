use std::{
    mem,
    ops::{Bound, Index, IndexMut, RangeBounds},
    slice::{Iter, IterMut, SliceIndex},
    sync::Arc,
};

use rand::Rng;
use serde::{Deserialize, Serialize};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Thread safe lazily cloned vector. The vector may be cloned only when you
/// actually need to mutate its data.
///
/// May be in one of two states: [`AlcVec::Static`] or [`AlcVec::Dynamic`].
///
/// When in static state, the vector is immutable and when mutation is necesary
/// it will have to be cloned, or if this is the only instance of the static
/// data, the data will be reclaimed.
///
/// In dynamic mode, this is same as vector.
#[derive(Debug)]
pub enum AlcVec<T>
where
    T: Clone,
{
    /// There is static immutable reference to the playlist
    Static(Arc<Vec<T>>),
    /// There is owned vector with the playlist
    Dynamic(Vec<T>),
}

impl<T> AlcVec<T>
where
    T: Clone,
{
    /// Creates new [`AlcVec`]
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

    /// Gets a mutable iterator for the data.
    ///
    /// - This may clone the vector.
    /// - The vector will be in dynamic state after this.
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.vec_mut()[..].iter_mut()
    }

    /// Gets/Creates arc from the vector.
    ///
    /// - The vector wil be in static state after this.
    pub fn make_arc(&mut self) -> Arc<Vec<T>> {
        self.make_static().clone()
    }

    /// Gets mutable reference to the vector.
    ///
    /// - This may clone the vector.
    /// - The vector will be in dynamic state after this.
    pub fn vec_mut(&mut self) -> &mut Vec<T> {
        self.make_dynamic()
    }

    /// Creates lazy clone of the vector.
    ///
    /// - The vector will be in static state after this.
    pub fn clone(&mut self) -> Self {
        Self::Static(self.make_arc())
    }

    /// Replaces the given range of values with the given value.
    ///
    /// - If the range or iterator is not empty, this may clone the vector.
    /// - If the range or iterator is not empty, the vector will be dynamic.
    pub fn splice<R, I>(&mut self, range: R, replace_with: I)
    where
        R: RangeBounds<usize>,
        I: IntoIterator<Item = T>,
    {
        let mut iter = replace_with.into_iter();
        let i = iter.next();
        if i.is_none() && is_range_empty(&range) {
            return;
        }

        self.make_dynamic().splice(range, i.into_iter().chain(iter));
    }

    /// Mix new values after the given index.
    ///
    /// - If the iterator is not empty, this may clone the vector.
    /// - If the iterator is not empty, the vector will be dynamic.
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
            if idx == v.len() {
                v.push(s);
            } else {
                let itm = mem::replace(&mut v[idx], s);
                v.push(itm);
            }
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

impl<T> FromIterator<T> for AlcVec<T>
where
    T: Clone,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::Dynamic(Vec::from_iter(iter))
    }
}

impl<'a, T> IntoIterator for &'a mut AlcVec<T>
where
    T: Clone,
{
    type Item = &'a mut T;

    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, T> IntoIterator for &'a AlcVec<T>
where
    T: Clone,
{
    type Item = &'a T;

    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
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

fn is_range_empty<R, T>(range: &R) -> bool
where
    R: RangeBounds<T>,
    T: PartialOrd,
{
    match (range.start_bound(), range.end_bound()) {
        (Bound::Unbounded, _) => false,
        (_, Bound::Unbounded) => false,
        (Bound::Included(a), _) => range.contains(a),
        (_, Bound::Included(a)) => range.contains(a),
        (Bound::Excluded(a), Bound::Excluded(b)) => a >= b,
    }
}
