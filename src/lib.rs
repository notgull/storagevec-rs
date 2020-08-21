// MIT/Apache2 License

//! Provides two types: `StorageVec` and `StorageMap`. These will either use stack-based storage methods or heap-based storage methods, based on if the `alloc` feature is enabled.

#[allow(incomplete_features)]
#[feature(const_generics)]

pub mod svec;
pub mod smap;

pub use svec::*;
pub use smap::*;
