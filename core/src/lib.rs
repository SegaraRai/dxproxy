//! Core library for dxproxy.
//!
//! This library provides the foundation for intercepting and proxying graphics API calls.
//! It currently supports:
//!
//! - DirectX 9 proxying with COM object management
//! - Common utilities for proxy lifecycle management
//! - Configuration and context management
//!
//! The library is designed to be used by DLL entry points that proxies system
//! graphics libraries while maintaining full API compatibility.

mod common;
use common::*;

pub mod dx9;

#[cfg(test)]
mod tests;

pub use windows;
pub use windows_core;
pub use windows_numerics;
