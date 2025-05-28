//! [`IDirect3DIndexBuffer9`] proxy implementation.

use super::*;
use std::ffi::c_void;
use windows::{Win32::Graphics::Direct3D9::*, core::*};

#[implement(IDirect3DIndexBuffer9)]
#[derive(Debug)]
pub struct ProxyDirect3DIndexBuffer9 {
    target: IDirect3DIndexBuffer9,
    context: DX9ProxyDeviceContext,
    proxy_device: IDirect3DDevice9,
}

impl ProxyDirect3DIndexBuffer9 {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "debug"))]
    pub fn new(target: IDirect3DIndexBuffer9, context: DX9ProxyDeviceContext, proxy_device: IDirect3DDevice9) -> Self {
        Self { target, context, proxy_device }
    }
}

impl Drop for ProxyDirect3DIndexBuffer9 {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "debug"))]
    fn drop(&mut self) {
        self.context.on_proxy_destroy(&self.target);
    }
}

impl_debug!(ProxyDirect3DIndexBuffer9_Impl);

#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]
impl IDirect3DIndexBuffer9_Impl for ProxyDirect3DIndexBuffer9_Impl {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn Lock(&self, offsettolock: u32, sizetolock: u32, ppbdata: *mut *mut c_void, flags: u32) -> Result<()> {
        unsafe { self.target.Lock(offsettolock, sizetolock, ppbdata, flags) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn Unlock(&self) -> Result<()> {
        unsafe { self.target.Unlock() }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn GetDesc(&self, pdesc: *mut D3DINDEXBUFFER_DESC) -> Result<()> {
        unsafe { self.target.GetDesc(pdesc) }
    }
}

#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]
impl IDirect3DResource9_Impl for ProxyDirect3DIndexBuffer9_Impl {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn GetDevice(&self) -> Result<IDirect3DDevice9> {
        Ok(self.proxy_device.clone())
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn SetPrivateData(&self, refguid: *const GUID, pdata: *const c_void, sizeofdata: u32, flags: u32) -> Result<()> {
        unsafe { self.target.SetPrivateData(refguid, pdata, sizeofdata, flags) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn GetPrivateData(&self, refguid: *const GUID, pdata: *mut c_void, psizeofdata: *mut u32) -> Result<()> {
        unsafe { self.target.GetPrivateData(refguid, pdata, psizeofdata) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn FreePrivateData(&self, refguid: *const GUID) -> Result<()> {
        unsafe { self.target.FreePrivateData(refguid) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "trace"))]
    fn SetPriority(&self, prioritynew: u32) -> u32 {
        unsafe { self.target.SetPriority(prioritynew) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "trace"))]
    fn GetPriority(&self) -> u32 {
        unsafe { self.target.GetPriority() }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "trace"))]
    fn PreLoad(&self) {
        unsafe { self.target.PreLoad() }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "trace"))]
    fn GetType(&self) -> D3DRESOURCETYPE {
        unsafe { self.target.GetType() }
    }
}
