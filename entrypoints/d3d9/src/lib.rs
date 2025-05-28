//! DirectX 9 DLL entry point
//!
//! This library serves as a drop-in replacement for d3d9.dll, intercepting
//! Direct3D creation calls and wrapping them with proxy objects.
//!
//! ## Usage
//!
//! Place d3d9.dll alongside an application executable. The library will
//! intercept calls to `Direct3DCreate9` and `Direct3DCreate9Ex`, creating
//! proxy-wrapped Direct3D objects.

#![windows_subsystem = "windows"]

use dxproxy::{windows::Win32::Graphics::Direct3D9::*, windows_core::*, *};

/// Creates a proxied Direct3D9 object.
///
/// This function replaces the system Direct3DCreate9 export, creating
/// a proxy wrapper around the original DirectX object.
///
/// # Arguments
/// * `sdkversion` - The DirectX SDK version requested by the application
///
/// # Returns
/// A proxy-wrapped IDirect3D9 object
///
/// # Safety
/// This function maintains the same safety contract as the original
/// Direct3DCreate9 function from the Windows SDK.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn Direct3DCreate9(sdkversion: u32) -> Option<IDirect3D9> {
    unsafe { dx9::Direct3DCreate9(sdkversion) }
}

/// Creates a proxied Direct3D9Ex object.
///
/// This function replaces the system Direct3DCreate9Ex export, creating
/// a proxy wrapper around the original DirectX Extended object.
///
/// # Arguments
/// * `sdkversion` - The DirectX SDK version requested by the application
/// * `ppd3d` - Output parameter for the created Direct3D9Ex object
///
/// # Returns
/// * `S_OK` on success
/// * Error HRESULT on failure
///
/// # Safety
/// This function maintains the same safety contract as the original
/// Direct3DCreate9Ex function. The caller must ensure `ppd3d` points
/// to valid memory.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn Direct3DCreate9Ex(sdkversion: u32, ppd3d: *mut Option<IDirect3D9Ex>) -> HRESULT {
    unsafe { dx9::Direct3DCreate9Ex(sdkversion, ppd3d) }
}
