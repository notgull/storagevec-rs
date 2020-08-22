// MIT/Apache2 License

//! Contains the `StorageVec`; a feature-gated vector structure that alternates between stack and heap
//! storage depending on the `alloc` feature.

#[cfg(any(not(feature = "alloc"), feature = "stack"))]
use core::mem::MaybeUninit;
#[cfg(not(feature = "alloc"))]
use tinyvec::{ArrayVec, ArrayVecIterator};

#[cfg(all(feature = "alloc", not(feature = "stack")))]
use alloc::vec::{self, Vec};
#[cfg(all(feature = "alloc", not(feature = "stack")))]
use core::marker::PhantomData;

#[cfg(all(feature = "alloc", feature = "stack"))]
use tinyvec::{TinyVec, TinyVecIterator};

use core::{iter, ops};

// helper struct to wrap MaybeUninit in a Default-compatible layer
// this is your fault: https://github.com/rust-lang/rust/issues/49147
#[cfg(any(not(feature = "alloc"), feature = "stack"))]
#[repr(transparent)]
struct UninitContainer<T>(MaybeUninit<T>);

#[cfg(any(not(feature = "alloc"), feature = "stack"))]
impl<T> UninitContainer<T> {
    #[inline]
    const fn new(item: T) -> Self {
        Self(MaybeUninit::new(item))
    }

    #[inline]
    const fn uninit() -> Self {
        Self(MaybeUninit::uninit())
    }

    #[allow(clippy::declare_interior_mutable_const)]
    const UNINIT: Self = Self::uninit();

    #[inline]
    fn uninit_array<const N: usize>() -> [Self; N] {
        [Self::UNINIT; N]
    }

    #[inline]
    unsafe fn assume_init(this: Self) -> T {
        MaybeUninit::assume_init(this.0)
    }

    #[inline]
    unsafe fn slice_get_ref(this: &[Self]) -> &[T] {
        &*(this as *const [Self] as *const [T])
    }

    #[inline]
    unsafe fn slice_get_mut(this: &mut [Self]) -> &mut [T] {
        &mut *(this as *mut [Self] as *mut [T])
    }
}

#[cfg(any(not(feature = "alloc"), feature = "stack"))]
impl<T> Default for UninitContainer<T> {
    fn default() -> Self {
        Self::uninit()
    }
}

/// A list-like object that will either use the tinyvec `ArrayVec`, the standard library `Vec`,
/// or the tinyvec `TinyVec` as a backing implementation. It will use the `alloc` and `stack`
/// features to control this.
#[repr(transparent)]
pub struct StorageVec<T, const N: usize>(SVImpl<T, N>);

#[cfg(not(feature = "alloc"))]
#[repr(transparent)]
struct SVImpl<T, const N: usize>(ArrayVec<[UninitContainer<T>; N]>);

#[cfg(all(feature = "alloc", not(feature = "stack")))]
#[repr(transparent)]
struct SVImpl<T, const N: usize>(Vec<T>, PhantomData<[T; N]>);

#[cfg(all(feature = "alloc", feature = "stack"))]
#[repr(transparent)]
struct SVImpl<T, const N: usize>(TinyVec<[UninitContainer<T>; N]>);

impl<T, const N: usize> StorageVec<T, N> {
    /// Create a new `StorageVec`.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::new_impl()
    }

    #[cfg(all(feature = "alloc", not(feature = "stack")))]
    #[inline]
    const fn new_impl() -> Self {
        Self(SVImpl(Vec::new(), PhantomData))
    }

    #[cfg(all(feature = "alloc", feature = "stack"))]
    #[inline]
    fn new_impl() -> Self {
        Self(SVImpl(TinyVec::from_array_len(
            UninitContainer::uninit_array::<N>(),
            0,
        )))
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn new_impl() -> Self {
        Self(SVImpl(ArrayVec::from_array_len(
            UninitContainer::uninit_array::<N>(),
            0,
        )))
    }

    #[cfg(all(feature = "alloc", not(feature = "stack")))]
    #[inline]
    fn deref_impl(&self) -> &[T] {
        &(self.0).0
    }

    #[cfg(any(not(feature = "alloc"), feature = "stack"))]
    #[inline]
    fn deref_impl(&self) -> &[T] {
        // SAFETY: MaybeUninit<T> is a zero-size struct that has the same layout as a
        //         T. The slice points to the same location, so it should be guaranteed to
        //         be valid.
        unsafe { UninitContainer::slice_get_ref(&(self.0).0) }
    }

    #[cfg(all(feature = "alloc", not(feature = "stack")))]
    #[inline]
    fn deref_mut_impl(&mut self) -> &mut [T] {
        &mut (self.0).0
    }

    #[cfg(any(not(feature = "alloc"), feature = "stack"))]
    #[inline]
    fn deref_mut_impl(&mut self) -> &mut [T] {
        // SAFETY: Same as above.
        unsafe { UninitContainer::slice_get_mut(&mut (self.0).0) }
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

    #[cfg(all(feature = "alloc", not(feature = "stack")))]
    #[inline]
    fn try_push_impl(&mut self, item: T) -> Result<(), T> {
        (self.0).0.push(item);
        Ok(())
    }

    #[cfg(all(feature = "alloc", feature = "stack"))]
    #[inline]
    fn try_push_impl(&mut self, item: T) -> Result<(), T> {
        (self.0).0.push(UninitContainer::new(item));
        Ok(())
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn try_push_impl(&mut self, item: T) -> Result<(), T> {
        match (self.0).0.try_push(UninitContainer::new(item)) {
            None => Ok(()),
            // SAFETY: we have just confirmed the that container is initialized
            Some(reject) => Err(unsafe { UninitContainer::assume_init(reject) }),
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

    #[cfg(all(feature = "alloc", not(feature = "stack")))]
    #[inline]
    fn pop_impl(&mut self) -> Option<T> {
        (self.0).0.pop()
    }

    #[cfg(any(not(feature = "alloc"), feature = "stack"))]
    #[inline]
    fn pop_impl(&mut self) -> Option<T> {
        // SAFETY: the ArrayVec's length variable keeps track of which items of that
        //         array can be considered initialized. The MaybeUninit is really mostly
        //         for the Default requirement.
        match (self.0).0.pop() {
            Some(p) => Some(unsafe { UninitContainer::assume_init(p) }),
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

    #[cfg(all(feature = "alloc", not(feature = "stack")))]
    #[inline]
    fn try_insert_impl(&mut self, item: T, index: usize) -> Result<(), T> {
        (self.0).0.insert(index, item);
        Ok(())
    }

    #[cfg(all(feature = "alloc", feature = "stack"))]
    #[inline]
    fn try_insert_impl(&mut self, item: T, index: usize) -> Result<(), T> {
        (self.0).0.insert(index, UninitContainer::new(item));
        Ok(())
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn try_insert_impl(&mut self, item: T, index: usize) -> Result<(), T> {
        match (self.0).0.try_insert(index, UninitContainer::new(item)) {
            None => Ok(()),
            // SAFETY: Same as above.
            Some(reject) => Err(unsafe { UninitContainer::assume_init(reject) }),
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

    #[cfg(all(feature = "alloc", not(feature = "stack")))]
    #[inline]
    fn remove_impl(&mut self, index: usize) -> T {
        (self.0).0.remove(index)
    }

    #[cfg(any(not(feature = "alloc"), feature = "stack"))]
    #[inline]
    fn remove_impl(&mut self, index: usize) -> T {
        // SAFETY: See "pop" above
        unsafe { UninitContainer::assume_init((self.0).0.remove(index)) }
    }
}

/// An owning iterator for the `StorageVec`. Returned by `StorageVec::into_iter`.
#[repr(transparent)]
pub struct StorageVecIterator<T, const N: usize>(SVIterImpl<T, N>);

#[cfg(not(feature = "alloc"))]
#[repr(transparent)]
struct SVIterImpl<T, const N: usize>(ArrayVecIterator<[UninitContainer<T>; N]>);

#[cfg(all(feature = "alloc", not(feature = "stack")))]
#[repr(transparent)]
struct SVIterImpl<T, const N: usize>(vec::IntoIter<T>, PhantomData<[(); N]>);

#[cfg(all(feature = "alloc", feature = "stack"))]
#[repr(transparent)]
struct SVIterImpl<T, const N: usize>(TinyVecIterator<[UninitContainer<T>; N]>);

impl<T, const N: usize> StorageVecIterator<T, N> {
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

impl<T, const N: usize> Iterator for StorageVecIterator<T, N> {
    type Item = T;

    #[cfg(any(not(feature = "alloc"), feature = "stack"))]
    #[inline]
    fn next(&mut self) -> Option<T> {
        match (self.0).0.next() {
            None => None,
            // safety: same as "pop" above.
            Some(next) => Some(unsafe { UninitContainer::assume_init(next) }),
        }
    }

    #[cfg(all(feature = "alloc", not(feature = "stack")))]
    #[inline]
    fn next(&mut self) -> Option<T> {
        (self.0).0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0).0.size_hint()
    }
}

#[cfg(any(not(feature = "alloc"), not(feature = "stack")))]
impl<T, const N: usize> ExactSizeIterator for StorageVecIterator<T, N> {}

#[cfg(any(not(feature = "alloc"), not(feature = "stack")))]
impl<T, const N: usize> DoubleEndedIterator for StorageVecIterator<T, N> {
    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn next_back(&mut self) -> Option<T> {
        match (self.0).0.next_back() {
            None => None,
            // safety: same as "pop" above.
            Some(next) => Some(unsafe { UninitContainer::assume_init(next) }),
        }
    }

    #[cfg(all(feature = "alloc", not(feature = "stack")))]
    #[inline]
    fn next_back(&mut self) -> Option<T> {
        (self.0).0.next_back()
    }
}

impl<T, const N: usize> ops::Deref for StorageVec<T, N> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &[T] {
        self.deref_impl()
    }
}

#[cfg(any(not(feature = "alloc"), feature = "stack"))]
#[inline]
fn cloner<T: Clone, const N: usize>(sv: &StorageVec<T, N>) -> [UninitContainer<T>; N] {
    let mut arr: [UninitContainer<T>; N] = UninitContainer::uninit_array::<N>();
    sv.iter().enumerate().for_each(|(i, t)| {
        arr[i] = UninitContainer::new(t.clone());
    });
    arr
}

impl<T: Clone, const N: usize> Clone for StorageVec<T, N> {
    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn clone(&self) -> Self {
        Self(SVImpl(ArrayVec::from_array_len(cloner(self), self.len())))
    }

    #[cfg(all(feature = "alloc", feature = "stack"))]
    fn clone(&self) -> Self {
        Self(SVImpl(TinyVec::from_array_len(cloner(self), self.len())))
    }

    #[cfg(all(feature = "alloc", not(feature = "stack")))]
    #[inline]
    fn clone(&self) -> Self {
        Self(SVImpl((self.0).0.clone(), PhantomData))
    }
}

impl<T, const N: usize> ops::DerefMut for StorageVec<T, N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [T] {
        self.deref_mut_impl()
    }
}

impl<T, const N: usize> iter::IntoIterator for StorageVec<T, N> {
    type Item = T;
    type IntoIter = StorageVecIterator<T, N>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        StorageVecIterator::new(self)
    }
}

impl<T, const N: usize> iter::Extend<T> for StorageVec<T, N> {
    #[cfg(all(feature = "alloc", not(feature = "stack")))]
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        (self.0).0.extend(iter)
    }

    #[cfg(any(not(feature = "alloc"), feature = "stack"))]
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        (self.0)
            .0
            .extend(iter.into_iter().map(UninitContainer::new));
    }
}

impl<T, const N: usize> iter::FromIterator<T> for StorageVec<T, N> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut collection = Self::new();
        collection.extend(iter);
        collection
    }
}

impl<T, const N: usize> Default for StorageVec<T, N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
