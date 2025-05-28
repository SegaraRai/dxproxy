//! Common utilities and types shared across the library.
//!
//! This module provides reusable components for COM interface management,
//! parameter handling, and mapping between proxy and target objects.

mod com_mapping_tracker;
mod try_out_param;

pub use com_mapping_tracker::*;
pub use try_out_param::*;
