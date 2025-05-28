//! [`IDirect3DStateBlock9`] proxy implementation.

use super::*;
use windows::{Win32::Graphics::Direct3D9::*, core::*};

#[implement(IDirect3DStateBlock9)]
#[derive(Debug)]
pub struct ProxyDirect3DStateBlock9 {
    target: IDirect3DStateBlock9,
    context: DX9ProxyDeviceContext,
    proxy_device: IDirect3DDevice9,
}

impl ProxyDirect3DStateBlock9 {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "debug"))]
    pub fn new(target: IDirect3DStateBlock9, context: DX9ProxyDeviceContext, proxy_device: IDirect3DDevice9) -> Self {
        Self { target, context, proxy_device }
    }
}

impl Drop for ProxyDirect3DStateBlock9 {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "debug"))]
    fn drop(&mut self) {
        self.context.on_proxy_destroy(&self.target);
    }
}

impl_debug!(ProxyDirect3DStateBlock9_Impl);

#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]
impl IDirect3DStateBlock9_Impl for ProxyDirect3DStateBlock9_Impl {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn GetDevice(&self) -> Result<IDirect3DDevice9> {
        Ok(self.proxy_device.clone())
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn Capture(&self) -> Result<()> {
        unsafe { self.target.Capture() }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn Apply(&self) -> Result<()> {
        unsafe { self.target.Apply() }
    }
}
