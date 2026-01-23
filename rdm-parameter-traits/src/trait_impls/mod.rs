#[cfg(feature = "alloc")]
pub mod alloc_impl;
pub mod core_impl;
pub mod heapless_impl; // We use heapless for convenient no_std collections with fixed capacity
#[cfg(feature = "std")]
pub mod std_impl;
