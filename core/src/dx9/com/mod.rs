//! Direct3D 9 COM interface proxy implementations.
//!
//! This module contains proxy wrappers for all Direct3D 9 COM interfaces,
//! providing instrumentation, logging, and potential interception capabilities
//! for DirectX 9 graphics API calls.

use windows::Win32::Foundation::S_OK;
use windows_core::HRESULT;

/// Creates a Direct3D-specific HRESULT from a given error code.
#[allow(non_snake_case)]
const fn MAKE_D3DHRESULT(code: u32) -> HRESULT {
    // MAKE_HRESULT(1, _FACD3D, code) where _FACD3D is 0x876
    // -> (1 << 31) | (0x876 << 16) | code
    HRESULT((0x88760800 | code) as i32)
}

/// Standard success result for Direct3D operations.
pub const D3D_OK: HRESULT = S_OK;

/// Device lost error - occurs when the Direct3D device becomes unavailable.
pub const D3DERR_DEVICELOST: HRESULT = MAKE_D3DHRESULT(2152);

/// Invalid call error - indicates improper API usage or invalid parameters.
pub const D3DERR_INVALIDCALL: HRESULT = MAKE_D3DHRESULT(2156);

/// Implements Debug trait for proxy COM interfaces.
///
/// Provides formatted debug output showing the type name and both proxy and target interface pointers.
macro_rules! impl_debug {
    ($name:ident) => {
        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{} {:p} (<=> {:p})",
                    std::any::type_name::<Self>(),
                    self.as_interface::<IUnknown>().as_raw(),
                    self.target.as_raw()
                )
            }
        }
    };
}

/// Validates that a pointer is not null and returns an error if it is null.
///
/// This macro helps return an error early without creating unnecessary objects
/// when the destination pointer is null.
macro_rules! check_nullptr {
    ($ptr:expr) => {
        if $ptr.is_null() {
            #[cfg(feature = "tracing")]
            tracing::error!("Null pointer passed to {}", stringify!($ptr));
            return Err(D3DERR_INVALIDCALL.into());
        }
    };
    ($ptr:expr, $err:expr) => {
        if $ptr.is_null() {
            #[cfg(feature = "tracing")]
            tracing::error!("Null pointer passed to {}", stringify!($ptr));
            return Err($err);
        }
    };
}

use super::config::*;
use crate::try_out_param;

mod device_context;
mod idirect3d9;
mod idirect3d9ex;
mod idirect3dcubetexture9;
mod idirect3ddevice9;
mod idirect3ddevice9ex;
mod idirect3dindexbuffer9;
mod idirect3dpixelshader9;
mod idirect3dquery9;
mod idirect3dstateblock9;
mod idirect3dsurface9;
mod idirect3dswapchain9;
mod idirect3dswapchain9ex;
mod idirect3dtexture9;
mod idirect3dvertexbuffer9;
mod idirect3dvertexdeclaration9;
mod idirect3dvertexshader9;
mod idirect3dvolume9;
mod idirect3dvolumetexture9;

pub use device_context::*;
pub use idirect3d9::*;
pub use idirect3d9ex::*;
pub use idirect3dcubetexture9::*;
pub use idirect3ddevice9::*;
pub use idirect3ddevice9ex::*;
pub use idirect3dindexbuffer9::*;
pub use idirect3dpixelshader9::*;
pub use idirect3dquery9::*;
pub use idirect3dstateblock9::*;
pub use idirect3dsurface9::*;
pub use idirect3dswapchain9::*;
pub use idirect3dswapchain9ex::*;
pub use idirect3dtexture9::*;
pub use idirect3dvertexbuffer9::*;
pub use idirect3dvertexdeclaration9::*;
pub use idirect3dvertexshader9::*;
pub use idirect3dvolume9::*;
pub use idirect3dvolumetexture9::*;
