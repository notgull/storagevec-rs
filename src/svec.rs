// MIT/Apache2 License

//! Contains the `StorageVec`; a feature-gated vector structure that alternates between stack and heap
//! storage depending on the `alloc` feature.

#[cfg(not(feature = "alloc"))]
use core::{mem::MaybeUninit, slice};
#[cfg(not(feature = "alloc"))]
use tinyvec::ArrayVec;

#[cfg(all(feature = "alloc", not(feature = "stack")))]
use core::marker::PhantomData;

#[cfg(all(feature = "alloc", feature = "stack"))]
use smallvec::SmallVec;

use core::ops;

// Container for MaybeUninit stuff that implements Default
#[doc(hidden)]
#[repr(transparent)]
pub struct UninitContainer<T>(MaybeUninit<T>);

impl<T> UninitContainer<T> {
    #[inline]
    fn new(item: T) -> Self { Self(MaybeUninit::new(item)) }

    #[inline]
    fn uninit() -> Self { Self(MaybeUninit::uninit()) }
  
    const UNINIT: Self = Self(MaybeUninit::uninit());

    #[inline]
    unsafe fn assume_init(this: Self) -> T { MaybeUninit::assume_init(this) }
}

impl<T> Default for UninitContainer<T> {
    #[inline]
    fn default() -> Self { Self::uninit() }
}

/// A list-like object that will either use the tinyvec `ArrayVec`, the standard library `Vec`,
/// or the smallvec `SmallVec` as a backing implementation. It will use the `alloc` and `stack`
/// features to control this.
pub struct StorageVec<T, const N: usize>(SVImpl<T, N>);

#[cfg(not(feature = "alloc"))]
struct SVImpl<T, const N: usize>(ArrayVec<[UninitContainer<T>; N]>);

#[cfg(all(feature = "alloc", not(feature = "stack")))]
struct SVImpl<T, const N: usize>(Vec<T>, PhantomData<[T; N]>);

#[cfg(all(feature = "alloc", feature = "stack"))]
struct SVImpl<T, const N: usize>(SmallVec<[T; N]>);

impl<T, const N: usize> StorageVec<T, N> {
    /// Create a new StorageVec.
    #[inline]
    pub fn new() where [UninitContainer<T>; N]: Default -> Self { Self::new_impl() }

    #[cfg(all(feature = "alloc", not(feature = "stack")))]
    #[inline]
    const fn new_impl() -> Self { Self(SVImpl(Vec::new(), PhantomData)) }

    #[cfg(all(feature = "alloc", feature = "stack"))]
    #[inline]
    fn new_impl() -> Self { Self(SVImpl(SmallVec::new())) }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn new_impl() where [MaybeUninit<T>; N]: Default -> Self { Self(SVImpl([UninitContainer::UNINIT; N].into())) }    

    #[cfg(feature = "alloc")]
    #[inline]
    fn deref_impl(&self) -> &[T] {
        &self.0.0
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn deref_impl(&self) -> &[T] {
        // SAFETY: MaybeUninit<T> is a zero-size struct that has the same layout as a
        //         T. The slice points to the same location, so it should be guaranteed to
        //         be valid.
        unsafe { slice::from_raw_parts(self.0.0.as_ptr() as *const T, self.0.0.len()) }
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn deref_mut_impl(&self) -> &mut [T] {
        &mut self.0.0
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn deref_mut_impl(&self) -> &mut [T] {
        // SAFETY: Same as above.
        unsafe { slice::from_raw_parts_mut(self.0.0.as_mut_ptr() as *mut T, self.0.0.len()) }
    }

    /// Try to push an item onto this list.
    #[inline]
    pub fn try_push(&mut self, item: T) -> Result<(), T> {
        self.try_push_impl(item)
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn try_push_impl(&mut self, item: T) -> Result<(), T> {
        self.0.0.push(item);
        Ok(())
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn try_push_impl(&mut self, item: T) -> Result<(), T> {
        match self.0.0.try_push(UninitContainer::new(item)) {
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

    #[cfg(feature = "alloc")]
    #[inline]
    fn pop(&mut self) -> Option<T> {
        self.0.0.pop()
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn pop(&mut self) -> Option<T> {
        // SAFETY: the ArrayVec's length variable keeps track of which items of that
        //         array can be considered initialized. The MaybeUninit is really mostly
        //         for the Default requirement.
        unsafe { UninitContainer::assume_init(self.0.0.pop()) }
    }

    /// Try to insert an item into this list.
    #[inline]
    pub fn try_insert(&mut self, item: T, index: usize) -> Result<(), T> {
        self.try_insert_impl(item, index)
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn try_insert_impl(&mut self, item: T, index: usize) -> Result<(), T> {
        self.0.0.insert(item, index);
        Ok(())
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn try_insert_impl(&mut self, item: T, index: usize) -> Result<(), T> {
        match self.0.0.try_insert(UninitContainer::new(item)) {
            None => Ok(()), 
            // SAFETY: Same as above.
            Some(reject) => Err(unsafe { UninitContainer::assume_init(reject) }),
        }
    }

    /// Insert an item into this list, and panic if the insert operation fails.
    #[inline]
    pub fn insert(&mut self, item: T, index: usize) {
        if let Err(_) = self.try_insert(item) {
            panic!("<StorageVec> Failed to insert item into list due to capacity overflow");
        }
    }

    /// Remove an item from this list.
    #[inline]
    pub fn remove(&mut self, index: usize) -> Option<T> {
        self.remove_impl(index)
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn remove_impl(&mut self, index: usize) -> Option<T> {
        self.0.0.remove(index)
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn remove_impl(&mut self, index: usize) -> Option<T> {
        // SAFETY: See "pop" above
        unsafe { UninitContainer::assume_init(self.0.0.remove(index)) }
    }
}

impl<T, const N: usize> ops::Deref for StorageVec<T, N> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &[T] {
        self.deref_impl()
    }
}

impl<T, const N: usize> ops::DerefMut for StorageVec<T, N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [T] {
        self.deref_mut_impl()
    }
}
