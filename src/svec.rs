// MIT/Apache2 License

//! Contains the `StorageVec`; a feature-gated vector structure that alternates between stack and heap
//! storage depending on the `alloc` feature.

#[cfg(not(feature = "alloc"))]
use tinyvec::{ArrayVec, ArrayVecIterator};

#[cfg(all(feature = "alloc", not(feature = "stack")))]
use alloc::vec::{self, Vec};
#[cfg(all(feature = "alloc", not(feature = "stack")))]
use core::marker::PhantomData;

#[cfg(all(feature = "alloc", feature = "stack"))]
use tinyvec::{TinyVec, TinyVecIterator};

use core::{
    fmt, iter,
    ops::{self, RangeBounds},
};

/// A list-like object that will either use the tinyvec `ArrayVec`, the standard library `Vec`,
/// or the tinyvec `TinyVec` as a backing implementation. It will use the `alloc` and `stack`
/// features to control this.
#[repr(transparent)]
pub struct StorageVec<T: Default, const N: usize>(SVImpl<T, N>);

#[cfg(not(feature = "alloc"))]
#[repr(transparent)]
struct SVImpl<T: Default, const N: usize>(ArrayVec<[T; N]>);

#[cfg(all(feature = "alloc", not(feature = "stack")))]
#[repr(transparent)]
struct SVImpl<T: Default, const N: usize>(Vec<T>, PhantomData<[T; N]>);

#[cfg(all(feature = "alloc", feature = "stack"))]
#[repr(transparent)]
struct SVImpl<T: Default, const N: usize>(TinyVec<[T; N]>);

impl<T: Default, const N: usize> StorageVec<T, N> {
    /// Create a new `StorageVec`.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::new_impl()
    }

    #[cfg(all(feature = "alloc", not(feature = "stack")))]
    #[inline]
    fn new_impl() -> Self {
        Self(SVImpl(Vec::new(), PhantomData))
    }

    #[cfg(all(feature = "alloc", feature = "stack"))]
    #[inline]
    fn new_impl() -> Self {
        Self(SVImpl(TinyVec::new()))
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn new_impl() -> Self {
        Self(SVImpl(ArrayVec::new()))
    }

    #[inline]
    fn deref_impl(&self) -> &[T] {
        &(self.0).0
    }

    #[inline]
    fn deref_mut_impl(&mut self) -> &mut [T] {
        &mut (self.0).0
    }

    /// Try to push an item onto this list.
    ///
    /// # Errors
    ///
    /// If the push operation fails due to capacity overflow, the element is returned back
    /// in an `Err`.
    #[inline]
    pub fn try_push(&mut self, item: T) -> Result<(), T> {
        self.try_push_impl(item)
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn try_push_impl(&mut self, item: T) -> Result<(), T> {
        (self.0).0.push(item);
        Ok(())
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn try_push_impl(&mut self, item: T) -> Result<(), T> {
        match (self.0).0.try_push(item) {
            None => Ok(()),
            Some(reject) => Err(reject),
        }
    }

    /// Push an item onto this list, and panic if the push operation failed.
    #[inline]
    pub fn push(&mut self, item: T) {
        if let Err(_) = self.try_push(item) {
            panic!("<StorageVec> Failed to push item onto list due to capacity overflow");
        }
    }

    /// Pop an item from the back of this list.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.pop_impl()
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn pop_impl(&mut self) -> Option<T> {
        (self.0).0.pop()
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn pop_impl(&mut self) -> Option<T> {
        match (self.0).0.pop() {
            Some(p) => Some(p),
            None => None,
        }
    }

    /// Try to insert an item into this list.
    ///
    /// # Errors
    ///
    /// If the element cannot be inserted into the list due to capacity overflow,
    /// the element is returned back in an `Err`.
    #[inline]
    pub fn try_insert(&mut self, item: T, index: usize) -> Result<(), T> {
        self.try_insert_impl(item, index)
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn try_insert_impl(&mut self, item: T, index: usize) -> Result<(), T> {
        (self.0).0.insert(index, item);
        Ok(())
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn try_insert_impl(&mut self, item: T, index: usize) -> Result<(), T> {
        match (self.0).0.try_insert(index, UninitContainer::new(item)) {
            None => Ok(()),
            Some(reject) => Err(reject),
        }
    }

    /// Insert an item into this list, and panic if the insert operation fails.
    #[inline]
    pub fn insert(&mut self, item: T, index: usize) {
        if let Err(_) = self.try_insert(item, index) {
            panic!("<StorageVec> Failed to insert item into list due to capacity overflow");
        }
    }

    /// Remove an item from this list.
    #[inline]
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len() - 1 {
            None
        } else {
            Some(self.remove_impl(index))
        }
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn remove_impl(&mut self, index: usize) -> T {
        (self.0).0.remove(index)
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn remove_impl(&mut self, index: usize) -> T {
        (self.0).0.remove(index)
    }

    /// Create a drain iterator for this vector.
    #[inline]
    pub fn drain<'a, R: RangeBounds<usize> + 'a>(
        &'a mut self,
        range: R,
    ) -> impl Iterator<Item = T> + 'a
    where
        T: 'a,
    {
        (self.0).0.drain(range)
    }
}

/// An owning iterator for the `StorageVec`. Returned by `StorageVec::into_iter`.
#[repr(transparent)]
pub struct StorageVecIterator<T: Default, const N: usize>(SVIterImpl<T, N>);

#[cfg(not(feature = "alloc"))]
#[repr(transparent)]
struct SVIterImpl<T: Default, const N: usize>(ArrayVecIterator<[T; N]>);

#[cfg(all(feature = "alloc", not(feature = "stack")))]
#[repr(transparent)]
struct SVIterImpl<T: Default, const N: usize>(vec::IntoIter<T>, PhantomData<[(); N]>);

#[cfg(all(feature = "alloc", feature = "stack"))]
#[repr(transparent)]
struct SVIterImpl<T: Default, const N: usize>(TinyVecIterator<[T; N]>);

impl<T: Default, const N: usize> StorageVecIterator<T, N> {
    #[cfg(any(not(feature = "alloc"), feature = "stack"))]
    #[inline]
    fn new(list: StorageVec<T, N>) -> Self {
        Self(SVIterImpl((list.0).0.into_iter()))
    }

    #[cfg(all(feature = "alloc", not(feature = "stack")))]
    #[inline]
    fn new(list: StorageVec<T, N>) -> Self {
        Self(SVIterImpl((list.0).0.into_iter(), PhantomData))
    }
}

impl<T: Default, const N: usize> Iterator for StorageVecIterator<T, N> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        (self.0).0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0).0.size_hint()
    }
}

impl<T: Default, const N: usize> ExactSizeIterator for StorageVecIterator<T, N> {}

impl<T: Default, const N: usize> DoubleEndedIterator for StorageVecIterator<T, N> {
    #[inline]
    fn next_back(&mut self) -> Option<T> {
        (self.0).0.next_back()
    }
}

impl<T: Default, const N: usize> ops::Deref for StorageVec<T, N> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &[T] {
        self.deref_impl()
    }
}

impl<T: Clone + Default, const N: usize> Clone for StorageVec<T, N> {
    #[cfg(any(not(feature = "alloc"), feature = "stack"))]
    #[inline]
    fn clone(&self) -> Self {
        Self(SVImpl((self.0).0.clone()))
    }

    #[cfg(all(feature = "alloc", not(feature = "stack")))]
    #[inline]
    fn clone(&self) -> Self {
        Self(SVImpl((self.0).0.clone(), PhantomData))
    }
}

impl<T: Default, const N: usize> ops::DerefMut for StorageVec<T, N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [T] {
        self.deref_mut_impl()
    }
}

impl<T: Default, const N: usize> iter::IntoIterator for StorageVec<T, N> {
    type Item = T;
    type IntoIter = StorageVecIterator<T, N>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        StorageVecIterator::new(self)
    }
}

impl<T: Default, const N: usize> iter::Extend<T> for StorageVec<T, N> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        (self.0).0.extend(iter)
    }
}

impl<T: Default, const N: usize> iter::FromIterator<T> for StorageVec<T, N> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut collection = Self::new();
        collection.extend(iter);
        collection
    }
}

impl<T: Default, const N: usize> Default for StorageVec<T, N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Default + fmt::Debug, const N: usize> fmt::Debug for StorageVec<T, N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&(self.0).0, f)
    }
}
