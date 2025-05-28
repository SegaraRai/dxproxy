//! Utility for handling COM-style output parameters with proper error handling.
//!
//! This module provides a helper function for working with output parameters
//! in COM-style APIs, ensuring proper error handling when parameters are not set.

use windows::{Win32::Foundation::*, core::*};

/// Executes a function that takes an output parameter and returns the result or an error.
///
/// This helper function simplifies working with COM-style APIs that use output parameters.
/// It ensures that if the function succeeds but doesn't set the output parameter,
/// an appropriate error is returned.
///
/// # Arguments
/// * `func` - A closure that takes a mutable reference to an Option<T> and may set it
///
/// # Returns
/// * `Ok(T)` if the function succeeds and sets the output parameter
/// * `Err(E_POINTER)` if the function succeeds but doesn't set the parameter
/// * `Err(...)` if the function itself returns an error
pub fn try_out_param<T, F>(func: F) -> Result<T>
where
    F: FnOnce(&mut Option<T>) -> Result<()>,
{
    let mut out: Option<T> = None;
    func(&mut out)?;
    match out {
        Some(value) => Ok(value),
        None => Err(E_POINTER.into()), // Should never happen if the function is implemented correctly
    }
}
