//! [`IDirect3DSwapChain9`] proxy implementation.

use super::*;
use windows::{
    Win32::Foundation::*,
    Win32::Graphics::{Direct3D9::*, Gdi::*},
    core::*,
};

#[implement(IDirect3DSwapChain9)]
#[derive(Debug)]
pub struct ProxyDirect3DSwapChain9 {
    target: IDirect3DSwapChain9,
    context: DX9ProxyDeviceContext,
    proxy_device: IDirect3DDevice9,
}

impl ProxyDirect3DSwapChain9 {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "debug"))]
    pub fn new(target: IDirect3DSwapChain9, context: DX9ProxyDeviceContext, proxy_device: IDirect3DDevice9) -> Self {
        Self { target, context, proxy_device }
    }

    /// Creates a new proxy swap chain or upgrades to an Ex version if available.
    ///
    /// This function checks if the provided `target` can be cast to [`IDirect3DSwapChain9Ex`].
    /// If it can, it creates a [`ProxyDirect3DSwapChain9Ex`] instance; otherwise, it falls back to
    /// creating a regular [`ProxyDirect3DSwapChain9`].
    ///
    /// It is recommended to use this method rather than [`new`] directly, as it handles both
    /// cases seamlessly, ensuring that the correct interface is returned based on the target's type.
    ///
    /// # Arguments
    /// * `target` - The target swap chain to wrap.
    /// * `context` - The device context for the proxy.
    /// * `proxy_device` - The proxy device associated with the swap chain.
    ///
    /// # Returns
    /// An [`IDirect3DSwapChain9`] instance, which may be a proxy for either
    /// [`IDirect3DSwapChain9Ex`] or [`IDirect3DSwapChain9`], depending on the target's type.
    ///
    /// [`new`]: Self::new
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "debug"))]
    pub fn new_or_upgrade(target: IDirect3DSwapChain9, context: DX9ProxyDeviceContext, proxy_device: IDirect3DDevice9) -> IDirect3DSwapChain9 {
        if let Ok(ex_target) = target.cast::<IDirect3DSwapChain9Ex>() {
            let ex_interface: IDirect3DSwapChain9Ex = ProxyDirect3DSwapChain9Ex::new(ex_target, context, proxy_device).into();
            ex_interface.into()
        } else {
            // If the target is not an Ex version, we downgrade to the regular swap chain.
            Self::new(target, context, proxy_device).into()
        }
    }
}

impl Drop for ProxyDirect3DSwapChain9 {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "debug"))]
    fn drop(&mut self) {
        self.context.on_proxy_destroy(&self.target);
    }
}

impl_debug!(ProxyDirect3DSwapChain9_Impl);

/// Implementation block providing `*_Impl` methods that accept a COM interface getter function.
///
/// Since [`IDirect3DSwapChain9`] may be inherited by [`IDirect3DSwapChain9Ex`], directly exposing the
/// swap chain instance could cause inconsistencies such as upgrade issues or pointer changes.
/// These methods take an additional `get_self_interface` parameter that accepts a COM pointer
/// to expose only the necessary interface instances, ensuring proper type consistency.
#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref, clippy::too_many_arguments)]
impl ProxyDirect3DSwapChain9_Impl {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace", skip(get_self_interface)))]
    pub(super) unsafe fn GetBackBuffer_Impl<F: FnOnce() -> IDirect3DSwapChain9>(&self, get_self_interface: F, ibackbuffer: u32, r#type: D3DBACKBUFFER_TYPE) -> Result<IDirect3DSurface9> {
        let target = unsafe { self.target.GetBackBuffer(ibackbuffer, r#type) }?;
        let proxy = self.context.ensure_proxy(target, |target| {
            ProxyDirect3DSurface9::new(target, self.context.clone(), self.proxy_device.clone(), DX9SurfaceContainer::SwapChain(get_self_interface())).into()
        });
        Ok(proxy)
    }
}

/// Implementation of [`IDirect3DSwapChain9`] for [`ProxyDirect3DSwapChain9`].
///
/// Methods that need to pass a COM interface pointer of `self` should use the corresponding
/// `*_Impl` methods from the implementation block above. This ensures proper type consistency
/// when dealing with interface inheritance (e.g., [`IDirect3DSwapChain9Ex`] extending [`IDirect3DSwapChain9`]).
#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]
impl IDirect3DSwapChain9_Impl for ProxyDirect3DSwapChain9_Impl {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn Present(&self, psourcerect: *const RECT, pdestrect: *const RECT, hdestwindowoverride: HWND, pdirtyregion: *const RGNDATA, dwflags: u32) -> Result<()> {
        unsafe { self.target.Present(psourcerect, pdestrect, hdestwindowoverride, pdirtyregion, dwflags) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace", skip(pdestsurface)))]
    fn GetFrontBufferData(&self, pdestsurface: Ref<IDirect3DSurface9>) -> Result<()> {
        let target = self.context.get_target_nullable(pdestsurface).ok_or(D3DERR_INVALIDCALL)?;
        unsafe { self.target.GetFrontBufferData(target) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn GetBackBuffer(&self, ibackbuffer: u32, r#type: D3DBACKBUFFER_TYPE) -> Result<IDirect3DSurface9> {
        unsafe { self.GetBackBuffer_Impl(|| self.to_interface(), ibackbuffer, r#type) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn GetRasterStatus(&self, prasterstatus: *mut D3DRASTER_STATUS) -> Result<()> {
        unsafe { self.target.GetRasterStatus(prasterstatus) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn GetDisplayMode(&self, pmode: *mut D3DDISPLAYMODE) -> Result<()> {
        unsafe { self.target.GetDisplayMode(pmode) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn GetDevice(&self) -> Result<IDirect3DDevice9> {
        Ok(self.proxy_device.clone())
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn GetPresentParameters(&self, ppresentationparameters: *mut D3DPRESENT_PARAMETERS) -> Result<()> {
        unsafe { self.target.GetPresentParameters(ppresentationparameters) }
    }
}
