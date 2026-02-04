use std::{
    mem,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use rand::Rng;
use serde::{Deserialize, Serialize};

/// Thread safe lazily cloned value. The value may be cloned only when you
/// actually need to mutate its data.
///
/// May be in one of two states: [`AlcVec::Static`] or [`AlcVec::Dynamic`].
///
/// When in static state, the value is immutable and when mutation is necesary
/// it will have to be cloned, or if this is the only instance of the static
/// data, the data will be reclaimed.
///
/// In dynamic mode, this is same as the type itself.
#[derive(Debug)]
pub enum Alc<T> {
    Static(Arc<T>),
    Dynamic(T),
}

impl<T> Alc<T> {
    pub fn new(v: T) -> Self {
        Self::Dynamic(v)
    }
}

impl<T> Alc<T>
where
    T: Clone + Default,
{
    pub fn make_arc(this: &mut Self) -> Arc<T> {
        Self::make_static(this).clone()
    }

    pub fn clone(this: &mut Self) -> Self {
        Self::Static(Self::make_arc(this))
    }

    /// Extract value from the alc, potentionally cloning if shared.
    pub fn take(mut this: Self) -> T {
        Self::make_dynamic(&mut this);
        let Alc::Dynamic(d) = this else {
            unreachable!();
        };
        d
    }
}

impl<T> Alc<Vec<T>>
where
    T: Clone,
{
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

        let v = Self::make_dynamic(self);
        let mut rng = rand::rng();
        for s in i.into_iter().chain(it) {
            if after >= v.len() {
                v.push(s);
                continue;
            }
            let idx = rng.random_range(after + 1..=v.len());
            if idx == v.len() {
                v.push(s);
            } else {
                let itm = mem::replace(&mut v[idx], s);
                v.push(itm);
            }
        }
    }
}

impl<T: Default + Clone> Default for Alc<T> {
    fn default() -> Self {
        Self::Dynamic(T::default())
    }
}

impl<T> From<T> for Alc<T>
where
    T: Clone,
{
    fn from(value: T) -> Self {
        Self::Dynamic(value)
    }
}

impl<T> From<Arc<T>> for Alc<T>
where
    T: Clone,
{
    fn from(value: Arc<T>) -> Self {
        Self::Static(value)
    }
}

impl<T, V> Extend<V> for Alc<T>
where
    T: Clone + Extend<V> + Default,
{
    fn extend<I: IntoIterator<Item = V>>(&mut self, iter: I) {
        let mut it = iter.into_iter();
        let i = it.next();
        if i.is_some() {
            Self::make_dynamic(self).extend(i.into_iter().chain(it))
        }
    }
}

impl<T> Serialize for Alc<T>
where
    T: Serialize,
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

impl<'de, T> Deserialize<'de> for Alc<T>
where
    T: Clone + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::Dynamic(T::deserialize(deserializer)?))
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
                *place = Self::Dynamic(T::deserialize(deserializer)?);
                Ok(())
            }
            Self::Dynamic(v) => T::deserialize_in_place(deserializer, v),
        }
    }
}

impl<T> From<Alc<T>> for Arc<T>
where
    T: Clone + Default,
{
    fn from(mut value: Alc<T>) -> Self {
        Alc::make_arc(&mut value)
    }
}

impl<T, V> FromIterator<V> for Alc<T>
where
    T: Clone + FromIterator<V>,
{
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
        Self::Dynamic(T::from_iter(iter))
    }
}

impl<T> AsRef<T> for Alc<T> {
    fn as_ref(&self) -> &T {
        match self {
            Self::Dynamic(d) => d,
            Self::Static(s) => s,
        }
    }
}

impl<T> AsMut<T> for Alc<T>
where
    T: Clone + Default,
{
    fn as_mut(&mut self) -> &mut T {
        Self::make_dynamic(self)
    }
}

impl<'a, T> IntoIterator for &'a mut Alc<T>
where
    T: Clone + Default,
    &'a mut T: IntoIterator,
{
    type Item = <&'a mut T as IntoIterator>::Item;

    type IntoIter = <&'a mut T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        Alc::make_dynamic(self).into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Alc<T>
where
    T: Clone,
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;

    type IntoIter = <&'a T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.as_ref().into_iter()
    }
}

impl<T> DerefMut for Alc<T>
where
    T: Clone + Default,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T> Deref for Alc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

impl<T> Alc<T>
where
    T: Clone + Default,
{
    /// Clones the playlist and creates owned vector
    #[inline]
    fn make_dynamic(this: &mut Self) -> &mut T {
        match this {
            Self::Dynamic(d) => d,
            Self::Static(_) => {
                let Self::Static(v) = mem::take(this) else {
                    panic!();
                };
                match Arc::try_unwrap(v) {
                    Ok(v) => {
                        *this = Self::Dynamic(v);
                        let Self::Dynamic(d) = this else {
                            panic!();
                        };
                        d
                    }
                    Err(a) => {
                        *this = Self::Dynamic(a.as_ref().clone());
                        let Self::Dynamic(d) = this else {
                            panic!();
                        };
                        d
                    }
                }
            }
        }
    }

    #[inline]
    fn make_static(this: &mut Self) -> &Arc<T> {
        match this {
            Self::Dynamic(d) => {
                *this = Self::Static(Arc::new(mem::take(d)));
                let Self::Static(s) = this else {
                    panic!();
                };
                s
            }
            Self::Static(s) => s,
        }
    }
}
