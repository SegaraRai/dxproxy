//! [`IDirect3DVertexBuffer9`] proxy implementation.

use super::*;
use std::ffi::c_void;
use tracing::instrument;
use windows::{Win32::Graphics::Direct3D9::*, core::*};

#[implement(IDirect3DVertexBuffer9)]
#[derive(Debug)]
pub struct ProxyDirect3DVertexBuffer9 {
    target: IDirect3DVertexBuffer9,
    context: DX9ProxyDeviceContext,
    proxy_device: IDirect3DDevice9,
}

impl ProxyDirect3DVertexBuffer9 {
    #[instrument(ret, level = "debug")]
    pub fn new(target: IDirect3DVertexBuffer9, context: DX9ProxyDeviceContext, proxy_device: IDirect3DDevice9) -> Self {
        Self { target, context, proxy_device }
    }
}

impl Drop for ProxyDirect3DVertexBuffer9 {
    #[instrument(ret, level = "debug")]
    fn drop(&mut self) {
        self.context.on_proxy_destroy(&self.target);
    }
}

impl_debug!(ProxyDirect3DVertexBuffer9_Impl);

#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]
impl IDirect3DVertexBuffer9_Impl for ProxyDirect3DVertexBuffer9_Impl {
    #[instrument(err, ret, level = "trace")]
    fn Lock(&self, offsettolock: u32, sizetolock: u32, ppbdata: *mut *mut c_void, flags: u32) -> Result<()> {
        unsafe { self.target.Lock(offsettolock, sizetolock, ppbdata, flags) }
    }

    #[instrument(err, ret, level = "trace")]
    fn Unlock(&self) -> Result<()> {
        unsafe { self.target.Unlock() }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetDesc(&self, pdesc: *mut D3DVERTEXBUFFER_DESC) -> Result<()> {
        unsafe { self.target.GetDesc(pdesc) }
    }
}

#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]
impl IDirect3DResource9_Impl for ProxyDirect3DVertexBuffer9_Impl {
    #[instrument(err, ret, level = "trace")]
    fn GetDevice(&self) -> Result<IDirect3DDevice9> {
        Ok(self.proxy_device.clone())
    }

    #[instrument(err, ret, level = "trace")]
    fn SetPrivateData(&self, refguid: *const GUID, pdata: *const c_void, sizeofdata: u32, flags: u32) -> Result<()> {
        unsafe { self.target.SetPrivateData(refguid, pdata, sizeofdata, flags) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetPrivateData(&self, refguid: *const GUID, pdata: *mut c_void, psizeofdata: *mut u32) -> Result<()> {
        unsafe { self.target.GetPrivateData(refguid, pdata, psizeofdata) }
    }

    #[instrument(err, ret, level = "trace")]
    fn FreePrivateData(&self, refguid: *const GUID) -> Result<()> {
        unsafe { self.target.FreePrivateData(refguid) }
    }

    #[instrument(ret, level = "trace")]
    fn SetPriority(&self, prioritynew: u32) -> u32 {
        unsafe { self.target.SetPriority(prioritynew) }
    }

    #[instrument(ret, level = "trace")]
    fn GetPriority(&self) -> u32 {
        unsafe { self.target.GetPriority() }
    }

    #[instrument(ret, level = "trace")]
    fn PreLoad(&self) {
        unsafe { self.target.PreLoad() }
    }

    #[instrument(ret, level = "trace")]
    fn GetType(&self) -> D3DRESOURCETYPE {
        unsafe { self.target.GetType() }
    }
}
