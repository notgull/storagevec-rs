// MIT/Apache2 License

//! Provides two types: `StorageVec` and `StorageMap`. These will either use stack-based storage
//! methods or heap-based storage methods, based on if the `alloc` feature is enabled.
//!
//! The idea behind this crate is to allow crates that require vector or map types to be able
//! to be `no_std` by allowing heap storage to be toggled on or off via features.

#![cfg_attr(all(feature = "alloc", not(feature = "stack")), forbid(unsafe_code))]
#![feature(min_const_generics)]
#![feature(maybe_uninit_uninit_array)]
#![no_std]
#![warn(clippy::pedantic)]
#![allow(clippy::redundant_pattern_matching)] // i try to avoid generating a lot of LLVM IR in order
                                              // to reduce compile times

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod smap;
pub mod svec;

pub use smap::*;
pub use svec::*;
