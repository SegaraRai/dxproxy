//! [`IDirect3DPixelShader9`] proxy implementation.

use super::*;
use std::ffi::c_void;
use tracing::instrument;
use windows::{Win32::Graphics::Direct3D9::*, core::*};

#[implement(IDirect3DPixelShader9)]
#[derive(Debug)]
pub struct ProxyDirect3DPixelShader9 {
    target: IDirect3DPixelShader9,
    context: DX9ProxyDeviceContext,
    proxy_device: IDirect3DDevice9,
}

impl ProxyDirect3DPixelShader9 {
    #[instrument(ret, level = "debug")]
    pub fn new(target: IDirect3DPixelShader9, context: DX9ProxyDeviceContext, proxy_device: IDirect3DDevice9) -> Self {
        Self { target, context, proxy_device }
    }
}

impl Drop for ProxyDirect3DPixelShader9 {
    #[instrument(ret, level = "debug")]
    fn drop(&mut self) {
        self.context.on_proxy_destroy(&self.target);
    }
}

impl_debug!(ProxyDirect3DPixelShader9_Impl);

#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]
impl IDirect3DPixelShader9_Impl for ProxyDirect3DPixelShader9_Impl {
    #[instrument(err, ret, level = "trace")]
    fn GetDevice(&self) -> Result<IDirect3DDevice9> {
        Ok(self.proxy_device.clone())
    }

    #[instrument(err, ret, level = "trace")]
    fn GetFunction(&self, pdata: *mut c_void, psizeofdata: *mut u32) -> Result<()> {
        unsafe { self.target.GetFunction(pdata, psizeofdata) }
    }
}
