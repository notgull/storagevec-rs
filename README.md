# Deprecated

I originally created this crate because I wanted to create a graphics framework that didn't allocate at all on the hot path. Looking back, I kind of cringe when I think about it (as well as the very loose and fast examples here). As far as I know no one is using this crate, and I don't plan on maintaining it. If you want to use this crate, feel free to fork it and make it your own, but know that it's probably a bad idea.

 - John "notgull" Nunley

# storagevec

[![Build Status](https://dev.azure.com/jtnunley01/gui-tools/_apis/build/status/not-a-seagull.storagevec-rs?branchName=master)](https://dev.azure.com/jtnunley01/gui-tools/_build/latest?definitionId=10&branchName=master)
[![crates.io](https://img.shields.io/crates/v/storagevec)](https://crates.io/crates/storagevec)
[![docs.rs](https://docs.rs/storagevec/badge.svg)](https://docs.rs/storagevec)

The `storagevec` crate provides the `StorageVec` and `StorageMap` types. If the `alloc` feature is enabled, these types will use the standard library `Vec` and `HashMap`, respectively. If it is not enabled, it will use `ArrayVec` and `TinyMap`, which both use stack-based storage. This is useful for crates that need to support `no_std` targets without allocators, but also need the convenience of list/map-like types.

If the `alloc` feature is enabled, there is no unsafe code introduced in this crate. If the `alloc` feature is disabled, or the `alloc` feature is enabled with the `stack` feature, unsafe code is introduced in the form of the `MaybeUninit` struct. However, I doubt this code will cause undefined behavior.

If the `stack` feature is enabled with the `alloc` feature, `StorageVec` will use `TinyVec` as backing storage.

This crate requires a nightly compiler due to the use of const generics.

## License

This crate is dual-licensed under the MIT License or the Apache 2.0 License, at your option. See LICENSE-MIT and LICENSE-Apache for more information.
