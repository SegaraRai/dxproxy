//! DirectX 9 proxy implementation and utilities.
//!
//! This module contains the complete DirectX 9 proxying implementation, including:
//! - COM object proxies and wrappers
//! - Configuration management
//! - DLL export functions for Direct3D creation

pub mod com;
pub mod config;
pub mod dll;

#[cfg(test)]
mod tests;

pub use config::*;
pub use dll::*;
