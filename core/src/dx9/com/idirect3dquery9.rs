//! [`IDirect3DQuery9`] proxy implementation.

use super::*;
use std::ffi::c_void;
use windows::{Win32::Graphics::Direct3D9::*, core::*};

#[implement(IDirect3DQuery9)]
#[derive(Debug)]
pub struct ProxyDirect3DQuery9 {
    target: IDirect3DQuery9,
    context: DX9ProxyDeviceContext,
    proxy_device: IDirect3DDevice9,
}

impl ProxyDirect3DQuery9 {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "debug"))]
    pub fn new(target: IDirect3DQuery9, context: DX9ProxyDeviceContext, proxy_device: IDirect3DDevice9) -> Self {
        Self { target, context, proxy_device }
    }
}

impl Drop for ProxyDirect3DQuery9 {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "debug"))]
    fn drop(&mut self) {
        self.context.on_proxy_destroy(&self.target);
    }
}

impl_debug!(ProxyDirect3DQuery9_Impl);

#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]
impl IDirect3DQuery9_Impl for ProxyDirect3DQuery9_Impl {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn GetDevice(&self) -> Result<IDirect3DDevice9> {
        Ok(self.proxy_device.clone())
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "trace"))]
    fn GetType(&self) -> D3DQUERYTYPE {
        unsafe { self.target.GetType() }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "trace"))]
    fn GetDataSize(&self) -> u32 {
        unsafe { self.target.GetDataSize() }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn Issue(&self, dwissueflags: u32) -> Result<()> {
        unsafe { self.target.Issue(dwissueflags) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn GetData(&self, pdata: *mut c_void, dwsize: u32, dwgetdataflags: u32) -> Result<()> {
        unsafe { self.target.GetData(pdata, dwsize, dwgetdataflags) }
    }
}
