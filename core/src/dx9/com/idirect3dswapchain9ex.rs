//! [`IDirect3DSwapChain9Ex`] proxy implementation.

use super::*;
use tracing::instrument;
use windows::{
    Win32::Foundation::*,
    Win32::Graphics::{Direct3D9::*, Gdi::*},
    core::*,
};

#[implement(IDirect3DSwapChain9Ex)]
#[derive(Debug)]
pub struct ProxyDirect3DSwapChain9Ex {
    proxy: ComObject<ProxyDirect3DSwapChain9>,
    target: IDirect3DSwapChain9Ex,
    context: DX9ProxyDeviceContext,
}

impl ProxyDirect3DSwapChain9Ex {
    #[instrument(ret, level = "debug")]
    pub fn new(target: IDirect3DSwapChain9Ex, context: DX9ProxyDeviceContext, proxy_device: IDirect3DDevice9) -> Self {
        Self {
            proxy: ProxyDirect3DSwapChain9::new(target.clone().into(), context.clone(), proxy_device).into_object(),
            target,
            context,
        }
    }
}

impl Drop for ProxyDirect3DSwapChain9Ex {
    #[instrument(ret, level = "debug")]
    fn drop(&mut self) {
        self.context.on_proxy_destroy(&self.target);
    }
}

impl_debug!(ProxyDirect3DSwapChain9Ex_Impl);

#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]
impl IDirect3DSwapChain9Ex_Impl for ProxyDirect3DSwapChain9Ex_Impl {
    #[instrument(err, ret, level = "trace")]
    fn GetLastPresentCount(&self, plastpresentcount: *mut u32) -> Result<()> {
        unsafe { self.target.GetLastPresentCount(plastpresentcount) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetPresentStats(&self, ppresentationstatistics: *mut D3DPRESENTSTATS) -> Result<()> {
        unsafe { self.target.GetPresentStats(ppresentationstatistics) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetDisplayModeEx(&self, pmode: *mut D3DDISPLAYMODEEX, protation: *mut D3DDISPLAYROTATION) -> Result<()> {
        unsafe { self.target.GetDisplayModeEx(pmode, protation) }
    }
}

macro_rules! proxy_as_interface {
    ($this:ident) => {
        $this.proxy.as_interface::<IDirect3DSwapChain9>()
    };
}

macro_rules! get_base_interface_fn {
    ($this:ident) => {
        || $this.to_interface::<IDirect3DSwapChain9Ex>().into()
    };
}

/// Implementation of [`IDirect3DSwapChain9`] for [`ProxyDirect3DSwapChain9Ex`].
///
/// Most methods delegate to the inner [`IDirect3DSwapChain9`] proxy. However, for methods that need to pass
/// a COM interface pointer of `self`, use the corresponding `*_Impl` methods from [`ProxyDirect3DSwapChain9_Impl`]
/// when available. Check the base implementation to determine if a `*_Impl` variant exists before
/// delegating directly to avoid interface inconsistencies in inheritance scenarios.
///
/// We don't have to customize methods here, since [`ProxyDirect3DSwapChain9Ex::proxy`] points to an [`IDirect3DSwapChain9`] object of [`ProxyDirect3DSwapChain9`].
#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]
impl IDirect3DSwapChain9_Impl for ProxyDirect3DSwapChain9Ex_Impl {
    fn Present(&self, psourcerect: *const RECT, pdestrect: *const RECT, hdestwindowoverride: HWND, pdirtyregion: *const RGNDATA, dwflags: u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).Present(psourcerect, pdestrect, hdestwindowoverride, pdirtyregion, dwflags) }
    }

    fn GetFrontBufferData(&self, pdestsurface: Ref<IDirect3DSurface9>) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetFrontBufferData(pdestsurface.as_ref()) }
    }

    fn GetBackBuffer(&self, ibackbuffer: u32, r#type: D3DBACKBUFFER_TYPE) -> Result<IDirect3DSurface9> {
        // Here, we call `GetBackBufferImpl` instead of `GetBackBuffer` to avoid exposing internal proxy of `ProxyDirect3DSwapChain9`.
        unsafe { self.proxy.GetBackBuffer_Impl(get_base_interface_fn!(self), ibackbuffer, r#type) }
    }

    fn GetRasterStatus(&self, prasterstatus: *mut D3DRASTER_STATUS) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetRasterStatus(prasterstatus) }
    }

    fn GetDisplayMode(&self, pmode: *mut D3DDISPLAYMODE) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetDisplayMode(pmode) }
    }

    fn GetDevice(&self) -> Result<IDirect3DDevice9> {
        unsafe { proxy_as_interface!(self).GetDevice() }
    }

    fn GetPresentParameters(&self, ppresentationparameters: *mut D3DPRESENT_PARAMETERS) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetPresentParameters(ppresentationparameters) }
    }
}
