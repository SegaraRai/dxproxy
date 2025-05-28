//! DirectX 9 DLL entry points and initialization.
//!
//! This module implements the main DirectX 9 DLL export functions that applications
//! call to create DirectX objects. It handles:
//! - Loading the original system d3d9.dll
//! - Initializing logging and tracing
//! - Intercepting Direct3DCreate9 and Direct3DCreate9Ex calls
//! - Creating proxy wrappers around the original DirectX objects
//!
//! Note that the actual DLL exports are in the crates under the entrypoints directory,
//! which are built as dynamic libraries. This module provides the implementation
//! for the proxy DLL that intercepts these calls and provides enhanced functionality.

use super::com::*;
use std::{
    env::var,
    fs::File,
    mem::transmute,
    sync::{Mutex, Once},
};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Direct3D9::*,
        System::{Console::*, LibraryLoader::*},
    },
    core::*,
};

/// One-time initialization guard for DLL setup.
static INIT: Once = Once::new();

/// Handle to the original system d3d9.dll.
static mut ORIGINAL_D3D9: HMODULE = HMODULE(std::ptr::null_mut());

/// Function pointer to the original Direct3DCreate9 function.
static mut ORIGINAL_DIRECT3DCREATE9: Option<extern "system" fn(u32) -> Option<IDirect3D9>> = None;

/// Function pointer to the original Direct3DCreate9Ex function.
static mut ORIGINAL_DIRECT3DCREATE9EX: Option<extern "system" fn(u32, *mut Option<IDirect3D9Ex>) -> HRESULT> = None;

#[cfg(any(feature = "tracing", feature = "tracing-instrument"))]
fn init_tracing() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let do_alloc_console = var("DXPROXY_ALLOC_CONSOLE").map_or(true, |v| v == "1");
    if do_alloc_console {
        let _ = unsafe { AllocConsole() }.inspect_err(|err| {
            eprintln!("Failed to allocate console: {err}");
        });
    }

    let log_filename = var("DXPROXY_LOG_FILE").unwrap_or_else(|_| "dxproxy.log".to_string());

    // Initialize tracing with console and optional file logging
    let registry = tracing_subscriber::registry().with(tracing_subscriber::EnvFilter::from_default_env());

    // Console layer with formatting
    let console_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_names(true)
        .with_ansi(true);

    // Try to create file layer, fall back to console-only if it fails
    match File::create(&log_filename) {
        Ok(log_file) => {
            let file_layer = tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .with_thread_names(true)
                .with_writer(Mutex::new(log_file))
                .with_ansi(false);

            registry.with(console_layer).with(file_layer).init();

            tracing::info!("Logging initialized with console and file output: {log_filename}");
        }
        Err(err) => {
            registry.with(console_layer).init();

            tracing::warn!("Failed to create log file {log_filename}: {err}, using console-only logging");
        }
    }
}

/// Initializes the proxy DLL by setting up logging and loading the original d3d9.dll.
///
/// This function:
/// - Allocates a console for debug output
/// - Sets up tracing with both console and file logging
/// - Loads the original system d3d9.dll from System32
/// - Resolves Direct3DCreate9 and Direct3DCreate9Ex function pointers
fn init() {
    #[cfg(any(feature = "tracing", feature = "tracing-instrument"))]
    init_tracing();

    // Load the original d3d9.dll
    #[allow(clippy::missing_transmute_annotations)]
    unsafe {
        let windows_dir = var("SystemRoot").map_or_else(|_| "C:\\Windows".to_string(), |value| value.trim_end_matches('\\').to_string());
        let original_dll = LoadLibraryW(&HSTRING::from(format!("{windows_dir}\\System32\\d3d9.dll")));
        match original_dll {
            Ok(dll_handle) => {
                #[cfg(feature = "tracing")]
                tracing::info!("Successfully loaded d3d9.dll: {dll_handle:?}");

                ORIGINAL_D3D9 = dll_handle;
                ORIGINAL_DIRECT3DCREATE9 = transmute(GetProcAddress(dll_handle, s!("Direct3DCreate9")));
                ORIGINAL_DIRECT3DCREATE9EX = transmute(GetProcAddress(dll_handle, s!("Direct3DCreate9Ex")));
            }
            Err(_err) => {
                #[cfg(feature = "tracing")]
                tracing::error!("Failed to load d3d9.dll: {_err}");
            }
        }
    }
}

/// Creates a Direct3D9 object with proxy wrapping.
///
/// This function intercepts calls to Direct3DCreate9 and creates a proxy wrapper
/// around the original DirectX object.
///
/// # Arguments
/// * `sdkversion` - The DirectX SDK version requested by the application
///
/// # Returns
/// * `Some(IDirect3D9)` - A proxy-wrapped Direct3D9 object on success
/// * `None` - If creation fails or the original DLL cannot be loaded
///
/// # Safety
/// This function interfaces with system DLLs and COM objects. It is safe to call
/// from applications as it maintains the same contract as the original Direct3DCreate9.
#[allow(non_snake_case)]
pub unsafe extern "system" fn Direct3DCreate9(sdkversion: u32) -> Option<IDirect3D9> {
    INIT.call_once(init);

    #[cfg(feature = "tracing")]
    tracing::info!("Direct3DCreate9 called with SDK version: {sdkversion}");

    if let Some(create_fn) = unsafe { ORIGINAL_DIRECT3DCREATE9 } {
        #[cfg(feature = "tracing")]
        tracing::debug!("Calling original Direct3DCreate9 function");

        let d3d9 = create_fn(sdkversion);
        if let Some(d3d9) = d3d9 {
            #[cfg(feature = "tracing")]
            tracing::info!("Successfully created IDirect3D9, creating proxy wrapper");

            let proxy = ProxyDirect3D9::new_or_upgrade(d3d9);

            #[cfg(feature = "tracing")]
            tracing::debug!("ProxyDirect3D9 created: {proxy:?}");

            return Some(proxy);
        } else {
            #[cfg(feature = "tracing")]
            tracing::error!("Original Direct3DCreate9 returned null for SDK version {sdkversion}");
        }
    } else {
        #[cfg(feature = "tracing")]
        tracing::error!("Original Direct3DCreate9 function not loaded from system d3d9.dll");
    }

    #[cfg(feature = "tracing")]
    tracing::error!("Direct3DCreate9 failed, returning null");

    None
}

/// Creates a Direct3D9Ex object with proxy wrapping.
///
/// This function intercepts calls to Direct3DCreate9Ex and creates a proxy wrapper
/// around the original DirectX Extended object.
///
/// # Arguments
/// * `sdkversion` - The DirectX SDK version requested by the application
/// * `ppd3d` - Output parameter for the created Direct3D9Ex object
///
/// # Returns
/// * `S_OK` - If the object was created successfully
/// * `E_POINTER` - If the output parameter is null
/// * `E_NOTIMPL` - If creation fails or the original DLL cannot be loaded
///
/// # Safety
/// This function interfaces with system DLLs and COM objects. The caller must ensure
/// that `ppd3d` points to valid memory that can hold an `Option<IDirect3D9Ex>`.
#[allow(non_snake_case)]
pub unsafe extern "system" fn Direct3DCreate9Ex(sdkversion: u32, ppd3d: *mut Option<IDirect3D9Ex>) -> HRESULT {
    INIT.call_once(init);

    #[cfg(feature = "tracing")]
    tracing::info!("Direct3DCreate9Ex called with SDK version: {sdkversion}");

    if ppd3d.is_null() {
        #[cfg(feature = "tracing")]
        tracing::error!("Direct3DCreate9Ex called with null output parameter");

        return E_POINTER;
    }

    if let Some(create_fn) = unsafe { ORIGINAL_DIRECT3DCREATE9EX } {
        #[cfg(feature = "tracing")]
        tracing::debug!("Calling original Direct3DCreate9Ex function");

        let mut d3d9_ex: Option<IDirect3D9Ex> = None;
        let result = create_fn(sdkversion, &mut d3d9_ex).ok();

        match result {
            Ok(_) => {
                if let Some(d3d9_ex) = d3d9_ex {
                    #[cfg(feature = "tracing")]
                    tracing::info!("Successfully created IDirect3D9Ex, creating proxy wrapper");

                    let wrapped_ex = ProxyDirect3D9Ex::new(d3d9_ex);

                    #[cfg(feature = "tracing")]
                    tracing::debug!("ProxyDirect3D9Ex created: {wrapped_ex:?}");

                    unsafe { ppd3d.write(Some(wrapped_ex.into())) };

                    #[cfg(feature = "tracing")]
                    tracing::info!("Direct3DCreate9Ex completed successfully");

                    return S_OK;
                } else {
                    #[cfg(feature = "tracing")]
                    tracing::error!("Original Direct3DCreate9Ex succeeded but returned null IDirect3D9Ex");
                }
            }
            Err(_err) => {
                #[cfg(feature = "tracing")]
                tracing::error!("Original Direct3DCreate9Ex failed with {_err} for SDK version {sdkversion}");
            }
        }
    } else {
        #[cfg(feature = "tracing")]
        tracing::error!("Original Direct3DCreate9Ex function not loaded from system d3d9.dll");
    }

    #[cfg(feature = "tracing")]
    tracing::error!("Direct3DCreate9Ex failed, returning E_NOTIMPL");

    E_NOTIMPL
}
