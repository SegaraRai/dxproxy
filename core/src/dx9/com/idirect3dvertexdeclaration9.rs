//! [`IDirect3DVertexDeclaration9`] proxy implementation.

use super::*;
use windows::{Win32::Graphics::Direct3D9::*, core::*};

#[implement(IDirect3DVertexDeclaration9)]
#[derive(Debug)]
pub struct ProxyDirect3DVertexDeclaration9 {
    target: IDirect3DVertexDeclaration9,
    context: DX9ProxyDeviceContext,
    proxy_device: IDirect3DDevice9,
}

impl ProxyDirect3DVertexDeclaration9 {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "debug"))]
    pub fn new(target: IDirect3DVertexDeclaration9, context: DX9ProxyDeviceContext, proxy_device: IDirect3DDevice9) -> Self {
        Self { target, context, proxy_device }
    }
}

impl Drop for ProxyDirect3DVertexDeclaration9 {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "debug"))]
    fn drop(&mut self) {
        self.context.on_proxy_destroy(&self.target);
    }
}

impl_debug!(ProxyDirect3DVertexDeclaration9_Impl);

#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]
impl IDirect3DVertexDeclaration9_Impl for ProxyDirect3DVertexDeclaration9_Impl {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn GetDevice(&self) -> Result<IDirect3DDevice9> {
        Ok(self.proxy_device.clone())
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn GetDeclaration(&self, pelement: *mut D3DVERTEXELEMENT9, pnumofelements: *mut u32) -> Result<()> {
        unsafe { self.target.GetDeclaration(pelement, pnumofelements) }
    }
}
