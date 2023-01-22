// MIT/Apache2 License

//! Contains the `StorageMap`; a feature-gated map structure that alternates between stack and heap
//! storage depending on the `alloc` feature.

#[cfg(not(feature = "alloc"))]
use tinymap::{TinyMap, TinyMapIterator};

#[cfg(feature = "alloc")]
use core::marker::PhantomData;
#[cfg(feature = "alloc")]
use hashbrown::HashMap;

use core::{fmt, hash::Hash, iter};

/// A map object that with either use the tinymap `TinyMap` or the hashbrown `HashMap` as a
/// backing implementation. It will use the `alloc` feature to control this.
#[repr(transparent)]
#[deprecated = "This crate is now deprecated."]
pub struct StorageMap<K: Eq + Ord + Hash, V, const N: usize>(SMImpl<K, V, N>);

#[cfg(feature = "alloc")]
#[repr(transparent)]
struct SMImpl<K: Eq + Ord + Hash, V, const N: usize>(HashMap<K, V>, PhantomData<[V; N]>);

#[cfg(not(feature = "alloc"))]
#[repr(transparent)]
struct SMImpl<K: Eq + Ord + Hash, V, const N: usize>(TinyMap<K, V, N>);

impl<K: Eq + Ord + Hash, V, const N: usize> StorageMap<K, V, N> {
    /// Create a new, empty `StorageMap`.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::new_impl()
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn new_impl() -> Self {
        Self(SMImpl(HashMap::new(), PhantomData))
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn new_impl() -> Self {
        Self(SMImpl(TinyMap::new()))
    }

    /// Get the length of this storage map.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        (self.0).0.len()
    }

    /// Tell whether or not the storage map is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        (self.0).0.is_empty()
    }

    /// Get an element from this map by its key.
    #[inline]
    #[must_use]
    pub fn get(&self, key: &K) -> Option<&V> {
        (self.0).0.get(key)
    }

    /// Get a mutable reference to an element by its key.
    #[inline]
    #[must_use]
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        (self.0).0.get_mut(key)
    }

    /// Insert a new element into this map. If the key already exists in the map, it
    /// returns the value previously held in that slot. Otherwise, it will return None.
    ///
    /// # Errors
    ///
    /// It will return back the key-value pair if the insertion cannot be
    /// accomplished due to capacity overflow.
    #[inline]
    pub fn try_insert(&mut self, key: K, value: V) -> Result<Option<V>, (K, V)> {
        self.try_insert_impl(key, value)
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn try_insert_impl(&mut self, key: K, value: V) -> Result<Option<V>, (K, V)> {
        Ok((self.0).0.insert(key, value))
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn try_insert_impl(&mut self, key: K, value: V) -> Result<Option<V>, (K, V)> {
        (self.0).0.try_insert(key, value)
    }

    /// Insert a new element into this map.
    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        (self.0).0.insert(key, value)
    }

    /// Remove a key/value entry from this map.
    #[inline]
    pub fn remove_entry(&mut self, key: &K) -> Option<(K, V)> {
        (self.0).0.remove_entry(key)
    }

    /// Remove a value from this map.
    #[inline]
    pub fn remove(&mut self, key: &K) -> Option<V> {
        (self.0).0.remove(key)
    }

    /// Tell whether this map contains a certain key.
    #[inline]
    pub fn contains_key(&self, key: &K) -> bool {
        (self.0).0.contains_key(key)
    }

    /// Get an iterator that iterates over the key-value pairs in arbitrary order.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        (self.0).0.iter()
    }

    /// Get an iterator that iterates over the key-value pairs in arbitary order, mutably.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        (self.0).0.iter_mut()
    }

    /// Get an iterator that iterates over the keys in arbitrary order.
    #[inline]
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        (self.0).0.keys()
    }

    /// Get an iterator that iterates over the values in arbitrary order.
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &V> {
        (self.0).0.values()
    }

    /// Get an iterator that iterates over the values in arbitrary order, mutably.
    #[inline]
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        (self.0).0.values_mut()
    }
}

impl<K: Ord + Eq + Hash + fmt::Debug, V: fmt::Debug, const N: usize> fmt::Debug
    for StorageMap<K, V, N>
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&(self.0).0, f)
    }
}

impl<K: Ord + Eq + Hash + Clone, V: Clone, const N: usize> Clone for SMImpl<K, V, N> {
    #[cfg(feature = "alloc")]
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<K: Ord + Eq + Hash + Clone, V: Clone, const N: usize> Clone for StorageMap<K, V, N> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<K: Ord + Eq + Hash, V, const N: usize> iter::IntoIterator for StorageMap<K, V, N> {
    type Item = (K, V);
    #[cfg(feature = "alloc")]
    type IntoIter = hashbrown::hash_map::IntoIter<K, V>;
    #[cfg(not(feature = "alloc"))]
    type IntoIter = TinyMapIterator<K, V, N>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        (self.0).0.into_iter()
    }
}

impl<K: Ord + Eq + Hash, V, const N: usize> iter::Extend<(K, V)> for StorageMap<K, V, N> {
    #[inline]
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        (self.0).0.extend(iter);
    }
}

impl<K: Ord + Eq + Hash, V, const N: usize> iter::FromIterator<(K, V)> for StorageMap<K, V, N> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut map = Self::new();
        map.extend(iter);
        map
    }
}

impl<K: Ord + Eq + Hash, V, const N: usize> Default for StorageMap<K, V, N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
