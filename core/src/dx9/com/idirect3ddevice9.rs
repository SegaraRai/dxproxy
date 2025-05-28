//! [`IDirect3DDevice9`] proxy implementation.
//!
//! This module provides a proxy wrapper for the IDirect3DDevice9 interface,
//! which represents a Direct3D device and provides methods for rendering,
//! state management, resource creation, and drawing operations.

use super::*;
use std::ffi::c_void;
use tracing::instrument;
use windows::{
    Win32::{
        Foundation::*,
        Graphics::{Direct3D9::*, Gdi::*},
    },
    core::*,
};
use windows_numerics::Matrix4x4;

/// Proxy wrapper for [`IDirect3DDevice9`] interface.
///
/// Intercepts and instruments all Direct3D device operations including rendering,
/// state management, resource creation, and drawing calls. Maintains a device context
/// for tracking state and configuration while forwarding operations to the target device.
#[implement(IDirect3DDevice9)]
#[derive(Debug)]
pub struct ProxyDirect3DDevice9 {
    target: IDirect3DDevice9,
    context: DX9ProxyDeviceContext,
    container: IDirect3D9,
}

impl ProxyDirect3DDevice9 {
    #[instrument(ret)]
    pub fn new(target: IDirect3DDevice9, config: DX9ProxyConfig, container: IDirect3D9) -> Self {
        Self {
            target,
            context: DX9ProxyDeviceContext::new(config),
            container,
        }
    }

    /// Creates a new proxy device or upgrades to an Ex version if available.
    ///
    /// This function checks if the provided `target` and `container` can be cast to
    /// [`IDirect3DDevice9Ex`] and [`IDirect3D9Ex`], respectively. If they can, it creates a
    /// [`ProxyDirect3DDevice9Ex`] instance; otherwise, it falls back to creating a
    /// regular [`ProxyDirect3DDevice9`].
    ///
    /// It is recommended to use this method rather than [`new`] directly, as it handles both
    /// cases seamlessly, ensuring that the correct interface is returned based on the target's type.
    ///
    /// # Arguments
    /// * `target` - The target device to wrap.
    /// * `context` - The device context for the proxy.
    /// * `container` - The Direct3D container associated with the device.
    ///
    /// # Returns
    /// An [`IDirect3DDevice9`] instance, which may be a proxy for either
    /// [`IDirect3DDevice9Ex`] or [`IDirect3DDevice9`], depending on the target's type.
    ///
    /// [`new`]: Self::new
    #[instrument(ret, level = "debug")]
    pub fn new_or_upgrade(target: IDirect3DDevice9, config: DX9ProxyConfig, container: IDirect3D9) -> IDirect3DDevice9 {
        if let Ok(ex_target) = target.cast::<IDirect3DDevice9Ex>() {
            if let Ok(ex_container) = container.cast::<IDirect3D9Ex>() {
                let ex_interface: IDirect3DDevice9Ex = ProxyDirect3DDevice9Ex::new(ex_target, config, ex_container).into();
                return ex_interface.into();
            }
        }

        // If the target and/or container are not an Ex version, we downgrade to the regular device.
        Self::new(target, config, container).into()
    }

    #[instrument(ret, level = "trace")]
    pub(super) fn get_context(&self) -> &DX9ProxyDeviceContext {
        &self.context
    }
}

impl Drop for ProxyDirect3DDevice9 {
    #[instrument(ret)]
    fn drop(&mut self) {}
}

impl_debug!(ProxyDirect3DDevice9_Impl);

/// Implementation block providing `*_Impl` methods that accept a COM interface getter function.
///
/// Since [`IDirect3DDevice9`] may be inherited by [`IDirect3DDevice9Ex`], directly exposing the
/// device instance could cause inconsistencies such as upgrade issues or pointer changes.
/// These methods take an additional `get_self_interface` parameter that accepts a COM pointer
/// to expose only the necessary interface instances, ensuring proper type consistency.
#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref, clippy::too_many_arguments)]
impl ProxyDirect3DDevice9_Impl {
    #[instrument(err, ret, level = "trace", skip(get_self_interface, pswapchain))]
    pub(super) unsafe fn CreateAdditionalSwapChain_Impl<F: FnOnce() -> IDirect3DDevice9>(
        &self,
        get_self_interface: F,
        ppresentationparameters: *mut D3DPRESENT_PARAMETERS,
        pswapchain: OutRef<IDirect3DSwapChain9>,
    ) -> Result<()> {
        check_nullptr!(pswapchain);

        let target = try_out_param(|out| unsafe { self.target.CreateAdditionalSwapChain(ppresentationparameters, out) })?;
        let proxy = self
            .context
            .ensure_proxy(target, |target| ProxyDirect3DSwapChain9::new_or_upgrade(target, self.context.clone(), get_self_interface()));
        pswapchain.write(Some(proxy))
    }

    #[instrument(err, ret, level = "trace", skip(get_self_interface))]
    pub(super) unsafe fn GetSwapChain_Impl<F: FnOnce() -> IDirect3DDevice9>(&self, get_self_interface: F, iswapchain: u32) -> Result<IDirect3DSwapChain9> {
        let target = unsafe { self.target.GetSwapChain(iswapchain) }?;
        let proxy = self
            .context
            .ensure_proxy(target, |target| ProxyDirect3DSwapChain9::new_or_upgrade(target, self.context.clone(), get_self_interface()));
        Ok(proxy)
    }

    #[instrument(err, ret, level = "trace", skip(get_self_interface))]
    pub(super) unsafe fn GetBackBuffer_Impl<F: FnOnce() -> IDirect3DDevice9>(&self, get_self_interface: F, iswapchain: u32, ibackbuffer: u32, r#type: D3DBACKBUFFER_TYPE) -> Result<IDirect3DSurface9> {
        unsafe { self.GetSwapChain_Impl(get_self_interface, iswapchain)?.GetBackBuffer(ibackbuffer, r#type) }
    }

    #[instrument(err, ret, level = "trace", skip(get_self_interface, pptexture))]
    pub(super) unsafe fn CreateTexture_Impl<F: FnOnce() -> IDirect3DDevice9>(
        &self,
        get_self_interface: F,
        width: u32,
        height: u32,
        levels: u32,
        usage: u32,
        format: D3DFORMAT,
        pool: D3DPOOL,
        pptexture: OutRef<IDirect3DTexture9>,
        psharedhandle: *mut HANDLE,
    ) -> Result<()> {
        check_nullptr!(pptexture);

        let target = try_out_param(|out| unsafe { self.target.CreateTexture(width, height, levels, usage, format, pool, out, psharedhandle) })?;
        let proxy = self
            .context
            .ensure_proxy(target, |target| ProxyDirect3DTexture9::new(target, self.context.clone(), get_self_interface()).into());
        pptexture.write(Some(proxy))
    }

    #[instrument(err, ret, level = "trace", skip(get_self_interface, ppvolumetexture))]
    pub(super) unsafe fn CreateVolumeTexture_Impl<F: FnOnce() -> IDirect3DDevice9>(
        &self,
        get_self_interface: F,
        width: u32,
        height: u32,
        depth: u32,
        levels: u32,
        usage: u32,
        format: D3DFORMAT,
        pool: D3DPOOL,
        ppvolumetexture: OutRef<IDirect3DVolumeTexture9>,
        psharedhandle: *mut HANDLE,
    ) -> Result<()> {
        check_nullptr!(ppvolumetexture);

        let target = try_out_param(|out| unsafe { self.target.CreateVolumeTexture(width, height, depth, levels, usage, format, pool, out, psharedhandle) })?;
        let proxy = self
            .context
            .ensure_proxy(target, |target| ProxyDirect3DVolumeTexture9::new(target, self.context.clone(), get_self_interface()).into());
        ppvolumetexture.write(Some(proxy))
    }

    #[instrument(err, ret, level = "trace", skip(get_self_interface, ppcubetexture))]
    pub(super) unsafe fn CreateCubeTexture_Impl<F: FnOnce() -> IDirect3DDevice9>(
        &self,
        get_self_interface: F,
        edgelength: u32,
        levels: u32,
        usage: u32,
        format: D3DFORMAT,
        pool: D3DPOOL,
        ppcubetexture: OutRef<IDirect3DCubeTexture9>,
        psharedhandle: *mut HANDLE,
    ) -> Result<()> {
        check_nullptr!(ppcubetexture);

        let target = try_out_param(|out| unsafe { self.target.CreateCubeTexture(edgelength, levels, usage, format, pool, out, psharedhandle) })?;
        let proxy = self
            .context
            .ensure_proxy(target, |target| ProxyDirect3DCubeTexture9::new(target, self.context.clone(), get_self_interface()).into());
        ppcubetexture.write(Some(proxy))
    }

    #[instrument(err, ret, level = "trace", skip(get_self_interface, ppvertexbuffer))]
    pub(super) unsafe fn CreateVertexBuffer_Impl<F: FnOnce() -> IDirect3DDevice9>(
        &self,
        get_self_interface: F,
        length: u32,
        usage: u32,
        fvf: u32,
        pool: D3DPOOL,
        ppvertexbuffer: OutRef<IDirect3DVertexBuffer9>,
        psharedhandle: *mut HANDLE,
    ) -> Result<()> {
        check_nullptr!(ppvertexbuffer);

        let target = try_out_param(|out| unsafe { self.target.CreateVertexBuffer(length, usage, fvf, pool, out, psharedhandle) })?;
        let proxy = self
            .context
            .ensure_proxy(target, |target| ProxyDirect3DVertexBuffer9::new(target, self.context.clone(), get_self_interface()).into());
        ppvertexbuffer.write(Some(proxy))
    }

    #[instrument(err, ret, level = "trace", skip(get_self_interface, ppindexbuffer))]
    pub(super) unsafe fn CreateIndexBuffer_Impl<F: FnOnce() -> IDirect3DDevice9>(
        &self,
        get_self_interface: F,
        length: u32,
        usage: u32,
        format: D3DFORMAT,
        pool: D3DPOOL,
        ppindexbuffer: OutRef<IDirect3DIndexBuffer9>,
        psharedhandle: *mut HANDLE,
    ) -> Result<()> {
        check_nullptr!(ppindexbuffer);

        let target = try_out_param(|out| unsafe { self.target.CreateIndexBuffer(length, usage, format, pool, out, psharedhandle) })?;
        let proxy = self
            .context
            .ensure_proxy(target, |target| ProxyDirect3DIndexBuffer9::new(target, self.context.clone(), get_self_interface()).into());
        ppindexbuffer.write(Some(proxy))
    }

    #[instrument(err, ret, level = "trace", skip(get_self_interface, ppsurface))]
    pub(super) unsafe fn CreateDepthStencilSurface_Impl<F: FnOnce() -> IDirect3DDevice9>(
        &self,
        get_self_interface: F,
        width: u32,
        height: u32,
        format: D3DFORMAT,
        multisample: D3DMULTISAMPLE_TYPE,
        multisamplequality: u32,
        discard: BOOL,
        ppsurface: OutRef<IDirect3DSurface9>,
        psharedhandle: *mut HANDLE,
    ) -> Result<()> {
        check_nullptr!(ppsurface);

        let target = try_out_param(|out| unsafe {
            self.target
                .CreateDepthStencilSurface(width, height, format, multisample, multisamplequality, discard.into(), out, psharedhandle)
        })?;
        let proxy = self.context.ensure_proxy(target, |target| {
            ProxyDirect3DSurface9::new(target, self.context.clone(), get_self_interface(), DX9SurfaceContainer::Standalone).into()
        });
        ppsurface.write(Some(proxy))
    }

    #[instrument(err, ret, level = "trace", skip(get_self_interface, ppsurface))]
    pub(super) unsafe fn CreateOffscreenPlainSurface_Impl<F: FnOnce() -> IDirect3DDevice9>(
        &self,
        get_self_interface: F,
        width: u32,
        height: u32,
        format: D3DFORMAT,
        pool: D3DPOOL,
        ppsurface: OutRef<IDirect3DSurface9>,
        psharedhandle: *mut HANDLE,
    ) -> Result<()> {
        check_nullptr!(ppsurface);

        let target = try_out_param(|out| unsafe { self.target.CreateOffscreenPlainSurface(width, height, format, pool, out, psharedhandle) })?;
        let proxy = self.context.ensure_proxy(target, |target| {
            ProxyDirect3DSurface9::new(target, self.context.clone(), get_self_interface(), DX9SurfaceContainer::Standalone).into()
        });
        ppsurface.write(Some(proxy))
    }

    #[instrument(err, ret, level = "trace", skip(get_self_interface, ppsurface))]
    pub(super) unsafe fn CreateRenderTarget_Impl<F: FnOnce() -> IDirect3DDevice9>(
        &self,
        get_self_interface: F,
        width: u32,
        height: u32,
        format: D3DFORMAT,
        multisample: D3DMULTISAMPLE_TYPE,
        multisamplequality: u32,
        lockable: BOOL,
        ppsurface: OutRef<IDirect3DSurface9>,
        psharedhandle: *mut HANDLE,
    ) -> Result<()> {
        check_nullptr!(ppsurface);

        let target = try_out_param(|out| unsafe {
            self.target
                .CreateRenderTarget(width, height, format, multisample, multisamplequality, lockable.into(), out, psharedhandle)
        })?;
        let proxy = self.context.ensure_proxy(target, |target| {
            ProxyDirect3DSurface9::new(target, self.context.clone(), get_self_interface(), DX9SurfaceContainer::Standalone).into()
        });
        ppsurface.write(Some(proxy))
    }

    #[instrument(err, ret, level = "trace", skip(get_self_interface))]
    pub(super) unsafe fn GetRenderTarget_Impl<F: FnOnce() -> IDirect3DDevice9>(&self, get_self_interface: F, rendertargetindex: u32) -> Result<IDirect3DSurface9> {
        let target = unsafe { self.target.GetRenderTarget(rendertargetindex) }?;
        let proxy = self.context.ensure_proxy(target, |target| {
            ProxyDirect3DSurface9::new(target, self.context.clone(), get_self_interface(), DX9SurfaceContainer::Standalone).into()
        });
        Ok(proxy)
    }

    #[instrument(err, ret, level = "trace", skip(get_self_interface))]
    pub(super) unsafe fn GetDepthStencilSurface_Impl<F: FnOnce() -> IDirect3DDevice9>(&self, get_self_interface: F) -> Result<IDirect3DSurface9> {
        let target = unsafe { self.target.GetDepthStencilSurface() }?;
        let proxy = self.context.ensure_proxy(target, |target| {
            ProxyDirect3DSurface9::new(target, self.context.clone(), get_self_interface(), DX9SurfaceContainer::Standalone).into()
        });
        Ok(proxy)
    }

    #[instrument(err, ret, level = "trace", skip(get_self_interface))]
    pub(super) unsafe fn CreateStateBlock_Impl<F: FnOnce() -> IDirect3DDevice9>(&self, get_self_interface: F, r#type: D3DSTATEBLOCKTYPE) -> Result<IDirect3DStateBlock9> {
        let target = unsafe { self.target.CreateStateBlock(r#type) }?;
        let proxy = self
            .context
            .ensure_proxy(target, |target| ProxyDirect3DStateBlock9::new(target, self.context.clone(), get_self_interface()).into());
        Ok(proxy)
    }

    #[instrument(err, ret, level = "trace", skip(get_self_interface))]
    pub(super) unsafe fn EndStateBlock_Impl<F: FnOnce() -> IDirect3DDevice9>(&self, get_self_interface: F) -> Result<IDirect3DStateBlock9> {
        let target = unsafe { self.target.EndStateBlock() }?;
        let proxy = self
            .context
            .ensure_proxy(target, |target| ProxyDirect3DStateBlock9::new(target, self.context.clone(), get_self_interface()).into());
        Ok(proxy)
    }

    #[instrument(err, ret, level = "trace", skip(get_self_interface))]
    pub(super) unsafe fn CreateVertexDeclaration_Impl<F: FnOnce() -> IDirect3DDevice9>(&self, get_self_interface: F, pvertexelements: *const D3DVERTEXELEMENT9) -> Result<IDirect3DVertexDeclaration9> {
        let target = unsafe { self.target.CreateVertexDeclaration(pvertexelements) }?;
        let proxy = self
            .context
            .ensure_proxy(target, |target| ProxyDirect3DVertexDeclaration9::new(target, self.context.clone(), get_self_interface()).into());
        Ok(proxy)
    }

    #[instrument(err, ret, level = "trace", skip(get_self_interface))]
    pub(super) unsafe fn GetVertexDeclaration_Impl<F: FnOnce() -> IDirect3DDevice9>(&self, get_self_interface: F) -> Result<IDirect3DVertexDeclaration9> {
        let target = unsafe { self.target.GetVertexDeclaration() }?;
        let proxy = self
            .context
            .ensure_proxy(target, |target| ProxyDirect3DVertexDeclaration9::new(target, self.context.clone(), get_self_interface()).into());
        Ok(proxy)
    }

    #[instrument(err, ret, level = "trace", skip(get_self_interface))]
    pub(super) unsafe fn CreateVertexShader_Impl<F: FnOnce() -> IDirect3DDevice9>(&self, get_self_interface: F, pfunction: *const u32) -> Result<IDirect3DVertexShader9> {
        let target = unsafe { self.target.CreateVertexShader(pfunction) }?;
        let proxy = self
            .context
            .ensure_proxy(target, |target| ProxyDirect3DVertexShader9::new(target, self.context.clone(), get_self_interface()).into());
        Ok(proxy)
    }

    #[instrument(err, ret, level = "trace", skip(get_self_interface))]
    pub(super) unsafe fn GetVertexShader_Impl<F: FnOnce() -> IDirect3DDevice9>(&self, get_self_interface: F) -> Result<IDirect3DVertexShader9> {
        let target = unsafe { self.target.GetVertexShader() }?;
        let proxy = self
            .context
            .ensure_proxy(target, |target| ProxyDirect3DVertexShader9::new(target, self.context.clone(), get_self_interface()).into());
        Ok(proxy)
    }

    #[instrument(err, ret, level = "trace", skip(get_self_interface, ppstreamdata))]
    pub(super) unsafe fn GetStreamSource_Impl<F: FnOnce() -> IDirect3DDevice9>(
        &self,
        get_self_interface: F,
        streamnumber: u32,
        ppstreamdata: OutRef<IDirect3DVertexBuffer9>,
        poffsetinbytes: *mut u32,
        pstride: *mut u32,
    ) -> Result<()> {
        check_nullptr!(ppstreamdata);

        let target = try_out_param(|out| unsafe { self.target.GetStreamSource(streamnumber, out, poffsetinbytes, pstride) })?;
        let proxy = self
            .context
            .ensure_proxy(target, |target| ProxyDirect3DVertexBuffer9::new(target, self.context.clone(), get_self_interface()).into());
        ppstreamdata.write(Some(proxy))
    }

    #[instrument(err, ret, level = "trace", skip(get_self_interface))]
    pub(super) unsafe fn GetIndices_Impl<F: FnOnce() -> IDirect3DDevice9>(&self, get_self_interface: F) -> Result<IDirect3DIndexBuffer9> {
        let target = unsafe { self.target.GetIndices() }?;
        let proxy = self
            .context
            .ensure_proxy(target, |target| ProxyDirect3DIndexBuffer9::new(target, self.context.clone(), get_self_interface()).into());
        Ok(proxy)
    }

    #[instrument(err, ret, level = "trace", skip(get_self_interface))]
    pub(super) unsafe fn CreatePixelShader_Impl<F: FnOnce() -> IDirect3DDevice9>(&self, get_self_interface: F, pfunction: *const u32) -> Result<IDirect3DPixelShader9> {
        let target = unsafe { self.target.CreatePixelShader(pfunction) }?;
        let proxy = self
            .context
            .ensure_proxy(target, |target| ProxyDirect3DPixelShader9::new(target, self.context.clone(), get_self_interface()).into());
        Ok(proxy)
    }

    #[instrument(err, ret, level = "trace", skip(get_self_interface))]
    pub(super) unsafe fn GetPixelShader_Impl<F: FnOnce() -> IDirect3DDevice9>(&self, get_self_interface: F) -> Result<IDirect3DPixelShader9> {
        let target = unsafe { self.target.GetPixelShader() }?;
        let proxy = self
            .context
            .ensure_proxy(target, |target| ProxyDirect3DPixelShader9::new(target, self.context.clone(), get_self_interface()).into());
        Ok(proxy)
    }

    #[instrument(err, ret, level = "trace", skip(get_self_interface))]
    pub(super) unsafe fn CreateQuery_Impl<F: FnOnce() -> IDirect3DDevice9>(&self, get_self_interface: F, r#type: D3DQUERYTYPE) -> Result<IDirect3DQuery9> {
        let target = unsafe { self.target.CreateQuery(r#type) }?;
        let proxy = self
            .context
            .ensure_proxy(target, |target| ProxyDirect3DQuery9::new(target, self.context.clone(), get_self_interface()).into());
        Ok(proxy)
    }
}

/// Implementation of [`IDirect3DDevice9`] for [`ProxyDirect3DDevice9`].
///
/// Methods that need to pass a COM interface pointer of `self` should use the corresponding
/// `*_Impl` methods from the implementation block above. This ensures proper type consistency
/// when dealing with interface inheritance (e.g., [`IDirect3DDevice9Ex`] extending [`IDirect3DDevice9`]).
#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]
impl IDirect3DDevice9_Impl for ProxyDirect3DDevice9_Impl {
    #[instrument(err, ret, level = "trace")]
    fn TestCooperativeLevel(&self) -> Result<()> {
        unsafe { self.target.TestCooperativeLevel() }
    }

    #[instrument(ret, level = "trace")]
    fn GetAvailableTextureMem(&self) -> u32 {
        unsafe { self.target.GetAvailableTextureMem() }
    }

    #[instrument(err, ret, level = "trace")]
    fn EvictManagedResources(&self) -> Result<()> {
        unsafe { self.target.EvictManagedResources() }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetDirect3D(&self) -> Result<IDirect3D9> {
        Ok(self.container.clone())
    }

    #[instrument(err, ret, level = "trace")]
    fn GetDeviceCaps(&self, pcaps: *mut D3DCAPS9) -> Result<()> {
        unsafe { self.target.GetDeviceCaps(pcaps) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetDisplayMode(&self, iswapchain: u32, pmode: *mut D3DDISPLAYMODE) -> Result<()> {
        unsafe { self.target.GetDisplayMode(iswapchain, pmode) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetCreationParameters(&self, pparameters: *mut D3DDEVICE_CREATION_PARAMETERS) -> Result<()> {
        unsafe { self.target.GetCreationParameters(pparameters) }
    }

    #[instrument(err, ret, level = "trace", skip(pcursorbitmap))]
    fn SetCursorProperties(&self, xhotspot: u32, yhotspot: u32, pcursorbitmap: Ref<IDirect3DSurface9>) -> Result<()> {
        let target = self.context.get_target_nullable(pcursorbitmap).ok_or(D3DERR_INVALIDCALL)?;
        unsafe { self.target.SetCursorProperties(xhotspot, yhotspot, target) }
    }

    #[instrument(ret, level = "trace")]
    fn SetCursorPosition(&self, x: i32, y: i32, flags: u32) {
        unsafe { self.target.SetCursorPosition(x, y, flags) }
    }

    #[instrument(ret, level = "trace")]
    fn ShowCursor(&self, bshow: BOOL) -> BOOL {
        unsafe { self.target.ShowCursor(bshow.into()) }
    }

    #[instrument(err, ret, level = "trace", skip(pswapchain))]
    fn CreateAdditionalSwapChain(&self, ppresentationparameters: *mut D3DPRESENT_PARAMETERS, pswapchain: OutRef<IDirect3DSwapChain9>) -> Result<()> {
        unsafe { self.CreateAdditionalSwapChain_Impl(|| self.to_interface(), ppresentationparameters, pswapchain) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetSwapChain(&self, iswapchain: u32) -> Result<IDirect3DSwapChain9> {
        unsafe { self.GetSwapChain_Impl(|| self.to_interface(), iswapchain) }
    }

    #[instrument(ret, level = "trace")]
    fn GetNumberOfSwapChains(&self) -> u32 {
        unsafe { self.target.GetNumberOfSwapChains() }
    }

    #[instrument(err, ret, level = "trace")]
    fn Reset(&self, ppresentationparameters: *mut D3DPRESENT_PARAMETERS) -> Result<()> {
        unsafe { self.target.Reset(ppresentationparameters) }
    }

    #[instrument(err, ret, level = "trace")]
    fn Present(&self, psourcerect: *const RECT, pdestrect: *const RECT, hdestwindowoverride: HWND, pdirtyregion: *const RGNDATA) -> Result<()> {
        unsafe { self.target.Present(psourcerect, pdestrect, hdestwindowoverride, pdirtyregion) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetBackBuffer(&self, iswapchain: u32, ibackbuffer: u32, r#type: D3DBACKBUFFER_TYPE) -> Result<IDirect3DSurface9> {
        unsafe { self.GetBackBuffer_Impl(|| self.to_interface(), iswapchain, ibackbuffer, r#type) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetRasterStatus(&self, iswapchain: u32, prasterstatus: *mut D3DRASTER_STATUS) -> Result<()> {
        unsafe { self.target.GetRasterStatus(iswapchain, prasterstatus) }
    }

    #[instrument(err, ret, level = "trace")]
    fn SetDialogBoxMode(&self, benabledialogs: BOOL) -> Result<()> {
        unsafe { self.target.SetDialogBoxMode(benabledialogs.into()) }
    }

    #[instrument(ret, level = "trace")]
    fn SetGammaRamp(&self, iswapchain: u32, flags: u32, pramp: *const D3DGAMMARAMP) {
        unsafe { self.target.SetGammaRamp(iswapchain, flags, pramp) }
    }

    #[instrument(ret, level = "trace")]
    fn GetGammaRamp(&self, iswapchain: u32, pramp: *mut D3DGAMMARAMP) {
        unsafe { self.target.GetGammaRamp(iswapchain, pramp) }
    }

    #[instrument(err, ret, level = "trace", skip(pptexture))]
    fn CreateTexture(&self, width: u32, height: u32, levels: u32, usage: u32, format: D3DFORMAT, pool: D3DPOOL, pptexture: OutRef<IDirect3DTexture9>, psharedhandle: *mut HANDLE) -> Result<()> {
        unsafe { self.CreateTexture_Impl(|| self.to_interface(), width, height, levels, usage, format, pool, pptexture, psharedhandle) }
    }

    #[instrument(err, ret, level = "trace", skip(ppvolumetexture))]
    fn CreateVolumeTexture(
        &self,
        width: u32,
        height: u32,
        depth: u32,
        levels: u32,
        usage: u32,
        format: D3DFORMAT,
        pool: D3DPOOL,
        ppvolumetexture: OutRef<IDirect3DVolumeTexture9>,
        psharedhandle: *mut HANDLE,
    ) -> Result<()> {
        unsafe { self.CreateVolumeTexture_Impl(|| self.to_interface(), width, height, depth, levels, usage, format, pool, ppvolumetexture, psharedhandle) }
    }

    #[instrument(err, ret, level = "trace", skip(ppcubetexture))]
    fn CreateCubeTexture(&self, edgelength: u32, levels: u32, usage: u32, format: D3DFORMAT, pool: D3DPOOL, ppcubetexture: OutRef<IDirect3DCubeTexture9>, psharedhandle: *mut HANDLE) -> Result<()> {
        unsafe { self.CreateCubeTexture_Impl(|| self.to_interface(), edgelength, levels, usage, format, pool, ppcubetexture, psharedhandle) }
    }

    #[instrument(err, ret, level = "trace", skip(ppvertexbuffer))]
    fn CreateVertexBuffer(&self, length: u32, usage: u32, fvf: u32, pool: D3DPOOL, ppvertexbuffer: OutRef<IDirect3DVertexBuffer9>, psharedhandle: *mut HANDLE) -> Result<()> {
        unsafe { self.CreateVertexBuffer_Impl(|| self.to_interface(), length, usage, fvf, pool, ppvertexbuffer, psharedhandle) }
    }

    #[instrument(err, ret, level = "trace", skip(ppindexbuffer))]
    fn CreateIndexBuffer(&self, length: u32, usage: u32, format: D3DFORMAT, pool: D3DPOOL, ppindexbuffer: OutRef<IDirect3DIndexBuffer9>, psharedhandle: *mut HANDLE) -> Result<()> {
        unsafe { self.CreateIndexBuffer_Impl(|| self.to_interface(), length, usage, format, pool, ppindexbuffer, psharedhandle) }
    }

    #[instrument(err, ret, level = "trace", skip(ppsurface))]
    fn CreateDepthStencilSurface(
        &self,
        width: u32,
        height: u32,
        format: D3DFORMAT,
        multisample: D3DMULTISAMPLE_TYPE,
        multisamplequality: u32,
        discard: BOOL,
        ppsurface: OutRef<IDirect3DSurface9>,
        psharedhandle: *mut HANDLE,
    ) -> Result<()> {
        unsafe { self.CreateDepthStencilSurface_Impl(|| self.to_interface(), width, height, format, multisample, multisamplequality, discard, ppsurface, psharedhandle) }
    }

    #[instrument(err, ret, level = "trace", skip(ppsurface))]
    fn CreateOffscreenPlainSurface(&self, width: u32, height: u32, format: D3DFORMAT, pool: D3DPOOL, ppsurface: OutRef<IDirect3DSurface9>, psharedhandle: *mut HANDLE) -> Result<()> {
        unsafe { self.CreateOffscreenPlainSurface_Impl(|| self.to_interface(), width, height, format, pool, ppsurface, psharedhandle) }
    }

    #[instrument(err, ret, level = "trace", skip(ppsurface))]
    fn CreateRenderTarget(
        &self,
        width: u32,
        height: u32,
        format: D3DFORMAT,
        multisample: D3DMULTISAMPLE_TYPE,
        multisamplequality: u32,
        lockable: BOOL,
        ppsurface: OutRef<IDirect3DSurface9>,
        psharedhandle: *mut HANDLE,
    ) -> Result<()> {
        unsafe { self.CreateRenderTarget_Impl(|| self.to_interface(), width, height, format, multisample, multisamplequality, lockable, ppsurface, psharedhandle) }
    }

    #[instrument(err, ret, level = "trace", skip(psourcesurface, pdestinationsurface))]
    fn UpdateSurface(&self, psourcesurface: Ref<IDirect3DSurface9>, psourcerect: *const RECT, pdestinationsurface: Ref<IDirect3DSurface9>, pdestpoint: *const POINT) -> Result<()> {
        let target_source = self.context.get_target_nullable(psourcesurface).ok_or(D3DERR_INVALIDCALL)?;
        let target_dest = self.context.get_target_nullable(pdestinationsurface).ok_or(D3DERR_INVALIDCALL)?;
        unsafe { self.target.UpdateSurface(target_source, psourcerect, target_dest, pdestpoint) }
    }

    #[instrument(err, ret, level = "trace", skip(psourcetexture, pdestinationtexture))]
    fn UpdateTexture(&self, psourcetexture: Ref<IDirect3DBaseTexture9>, pdestinationtexture: Ref<IDirect3DBaseTexture9>) -> Result<()> {
        let target_source = self.context.get_target_nullable(psourcetexture).ok_or(D3DERR_INVALIDCALL)?;
        let target_dest = self.context.get_target_nullable(pdestinationtexture).ok_or(D3DERR_INVALIDCALL)?;
        unsafe { self.target.UpdateTexture(target_source, target_dest) }
    }

    #[instrument(err, ret, level = "trace", skip(prendertarget, pdestsurface))]
    fn GetRenderTargetData(&self, prendertarget: Ref<IDirect3DSurface9>, pdestsurface: Ref<IDirect3DSurface9>) -> Result<()> {
        let target_render_target = self.context.get_target_nullable(prendertarget).ok_or(D3DERR_INVALIDCALL)?;
        let target_dest = self.context.get_target_nullable(pdestsurface).ok_or(D3DERR_INVALIDCALL)?;
        unsafe { self.target.GetRenderTargetData(target_render_target, target_dest) }
    }

    #[instrument(err, ret, level = "trace", skip(pdestsurface))]
    fn GetFrontBufferData(&self, iswapchain: u32, pdestsurface: Ref<IDirect3DSurface9>) -> Result<()> {
        let target = self.context.get_target_nullable(pdestsurface).ok_or(D3DERR_INVALIDCALL)?;
        unsafe { self.target.GetFrontBufferData(iswapchain, target) }
    }

    #[instrument(err, ret, level = "trace", skip(psourcesurface, pdestsurface))]
    fn StretchRect(&self, psourcesurface: Ref<IDirect3DSurface9>, psourcerect: *const RECT, pdestsurface: Ref<IDirect3DSurface9>, pdestrect: *const RECT, filter: D3DTEXTUREFILTERTYPE) -> Result<()> {
        let target_source = self.context.get_target_nullable(psourcesurface).ok_or(D3DERR_INVALIDCALL)?;
        let target_dest = self.context.get_target_nullable(pdestsurface).ok_or(D3DERR_INVALIDCALL)?;
        unsafe { self.target.StretchRect(target_source, psourcerect, target_dest, pdestrect, filter) }
    }

    #[instrument(err, ret, level = "trace", skip(psurface))]
    fn ColorFill(&self, psurface: Ref<IDirect3DSurface9>, prect: *const RECT, color: u32) -> Result<()> {
        let target = self.context.get_target_nullable(psurface).ok_or(D3DERR_INVALIDCALL)?;
        unsafe { self.target.ColorFill(target, prect, color) }
    }

    #[instrument(err, ret, level = "trace", skip(prendertarget))]
    fn SetRenderTarget(&self, rendertargetindex: u32, prendertarget: Ref<IDirect3DSurface9>) -> Result<()> {
        let target = self.context.get_target_nullable(prendertarget).ok_or(D3DERR_INVALIDCALL)?;
        unsafe { self.target.SetRenderTarget(rendertargetindex, target) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetRenderTarget(&self, rendertargetindex: u32) -> Result<IDirect3DSurface9> {
        unsafe { self.GetRenderTarget_Impl(|| self.to_interface(), rendertargetindex) }
    }

    #[instrument(err, ret, level = "trace", skip(pnewzstencil))]
    fn SetDepthStencilSurface(&self, pnewzstencil: Ref<IDirect3DSurface9>) -> Result<()> {
        let target = self.context.get_target_nullable(pnewzstencil).ok_or(D3DERR_INVALIDCALL)?;
        unsafe { self.target.SetDepthStencilSurface(target) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetDepthStencilSurface(&self) -> Result<IDirect3DSurface9> {
        unsafe { self.GetDepthStencilSurface_Impl(|| self.to_interface()) }
    }

    #[instrument(err, ret, level = "trace")]
    fn BeginScene(&self) -> Result<()> {
        unsafe { self.target.BeginScene() }
    }

    #[instrument(err, ret, level = "trace")]
    fn EndScene(&self) -> Result<()> {
        unsafe { self.target.EndScene() }
    }

    #[instrument(err, ret, level = "trace")]
    fn Clear(&self, count: u32, prects: *const D3DRECT, flags: u32, color: u32, z: f32, stencil: u32) -> Result<()> {
        unsafe { self.target.Clear(count, prects, flags, color, z, stencil) }
    }

    #[instrument(err, ret, level = "trace")]
    fn SetTransform(&self, state: D3DTRANSFORMSTATETYPE, pmatrix: *const Matrix4x4) -> Result<()> {
        unsafe { self.target.SetTransform(state, pmatrix) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetTransform(&self, state: D3DTRANSFORMSTATETYPE, pmatrix: *mut Matrix4x4) -> Result<()> {
        unsafe { self.target.GetTransform(state, pmatrix) }
    }

    #[instrument(err, ret, level = "trace")]
    fn MultiplyTransform(&self, param0: D3DTRANSFORMSTATETYPE, param1: *const Matrix4x4) -> Result<()> {
        unsafe { self.target.MultiplyTransform(param0, param1) }
    }

    #[instrument(err, ret, level = "trace")]
    fn SetViewport(&self, pviewport: *const D3DVIEWPORT9) -> Result<()> {
        unsafe { self.target.SetViewport(pviewport) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetViewport(&self, pviewport: *mut D3DVIEWPORT9) -> Result<()> {
        unsafe { self.target.GetViewport(pviewport) }
    }

    #[instrument(err, ret, level = "trace")]
    fn SetMaterial(&self, pmaterial: *const D3DMATERIAL9) -> Result<()> {
        unsafe { self.target.SetMaterial(pmaterial) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetMaterial(&self, pmaterial: *mut D3DMATERIAL9) -> Result<()> {
        unsafe { self.target.GetMaterial(pmaterial) }
    }

    #[instrument(err, ret, level = "trace")]
    fn SetLight(&self, index: u32, param1: *const D3DLIGHT9) -> Result<()> {
        unsafe { self.target.SetLight(index, param1) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetLight(&self, index: u32, param1: *mut D3DLIGHT9) -> Result<()> {
        unsafe { self.target.GetLight(index, param1) }
    }

    #[instrument(err, ret, level = "trace")]
    fn LightEnable(&self, index: u32, enable: BOOL) -> Result<()> {
        unsafe { self.target.LightEnable(index, enable.into()) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetLightEnable(&self, index: u32, penable: *mut BOOL) -> Result<()> {
        unsafe { self.target.GetLightEnable(index, penable) }
    }

    #[instrument(err, ret, level = "trace")]
    fn SetClipPlane(&self, index: u32, pplane: *const f32) -> Result<()> {
        unsafe { self.target.SetClipPlane(index, pplane) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetClipPlane(&self, index: u32, pplane: *mut f32) -> Result<()> {
        unsafe { self.target.GetClipPlane(index, pplane) }
    }

    #[instrument(err, ret, level = "trace")]
    fn SetRenderState(&self, state: D3DRENDERSTATETYPE, value: u32) -> Result<()> {
        unsafe { self.target.SetRenderState(state, value) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetRenderState(&self, state: D3DRENDERSTATETYPE, pvalue: *mut u32) -> Result<()> {
        unsafe { self.target.GetRenderState(state, pvalue) }
    }

    #[instrument(err, ret, level = "trace")]
    fn CreateStateBlock(&self, r#type: D3DSTATEBLOCKTYPE) -> Result<IDirect3DStateBlock9> {
        unsafe { self.CreateStateBlock_Impl(|| self.to_interface(), r#type) }
    }

    #[instrument(err, ret, level = "trace")]
    fn BeginStateBlock(&self) -> Result<()> {
        unsafe { self.target.BeginStateBlock() }
    }

    #[instrument(err, ret, level = "trace")]
    fn EndStateBlock(&self) -> Result<IDirect3DStateBlock9> {
        unsafe { self.EndStateBlock_Impl(|| self.to_interface()) }
    }

    #[instrument(err, ret, level = "trace")]
    fn SetClipStatus(&self, pclipstatus: *const D3DCLIPSTATUS9) -> Result<()> {
        unsafe { self.target.SetClipStatus(pclipstatus) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetClipStatus(&self, pclipstatus: *mut D3DCLIPSTATUS9) -> Result<()> {
        unsafe { self.target.GetClipStatus(pclipstatus) }
    }

    #[instrument(err, ret, level = "trace", skip(ptexture))]
    fn SetTexture(&self, stage: u32, ptexture: Ref<IDirect3DBaseTexture9>) -> Result<()> {
        let target = self.context.get_target_nullable(ptexture).ok_or(D3DERR_INVALIDCALL)?;
        unsafe { self.target.SetTexture(stage, target) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetTexture(&self, stage: u32) -> Result<IDirect3DBaseTexture9> {
        let target = unsafe { self.target.GetTexture(stage) }?;
        let proxy = self.context.get_proxy(target).ok_or(D3DERR_INVALIDCALL).inspect_err(|err| {
            tracing::error!("Failed to get texture proxy: {err}");
        })?;
        Ok(proxy)
    }

    #[instrument(err, ret, level = "trace")]
    fn GetTextureStageState(&self, stage: u32, r#type: D3DTEXTURESTAGESTATETYPE, pvalue: *mut u32) -> Result<()> {
        unsafe { self.target.GetTextureStageState(stage, r#type, pvalue) }
    }

    #[instrument(err, ret, level = "trace")]
    fn SetTextureStageState(&self, stage: u32, r#type: D3DTEXTURESTAGESTATETYPE, value: u32) -> Result<()> {
        unsafe { self.target.SetTextureStageState(stage, r#type, value) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetSamplerState(&self, sampler: u32, r#type: D3DSAMPLERSTATETYPE, pvalue: *mut u32) -> Result<()> {
        unsafe { self.target.GetSamplerState(sampler, r#type, pvalue) }
    }

    #[instrument(err, ret, level = "trace")]
    fn SetSamplerState(&self, sampler: u32, r#type: D3DSAMPLERSTATETYPE, value: u32) -> Result<()> {
        unsafe { self.target.SetSamplerState(sampler, r#type, value) }
    }

    #[instrument(err, ret, level = "trace")]
    fn ValidateDevice(&self, pnumpasses: *mut u32) -> Result<()> {
        unsafe { self.target.ValidateDevice(pnumpasses) }
    }

    #[instrument(err, ret, level = "trace")]
    fn SetPaletteEntries(&self, palettenumber: u32, pentries: *const PALETTEENTRY) -> Result<()> {
        unsafe { self.target.SetPaletteEntries(palettenumber, pentries) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetPaletteEntries(&self, palettenumber: u32, pentries: *mut PALETTEENTRY) -> Result<()> {
        unsafe { self.target.GetPaletteEntries(palettenumber, pentries) }
    }

    #[instrument(err, ret, level = "trace")]
    fn SetCurrentTexturePalette(&self, palettenumber: u32) -> Result<()> {
        unsafe { self.target.SetCurrentTexturePalette(palettenumber) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetCurrentTexturePalette(&self, ppalettenumber: *mut u32) -> Result<()> {
        unsafe { self.target.GetCurrentTexturePalette(ppalettenumber) }
    }

    #[instrument(err, ret, level = "trace")]
    fn SetScissorRect(&self, prect: *const RECT) -> Result<()> {
        unsafe { self.target.SetScissorRect(prect) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetScissorRect(&self, prect: *mut RECT) -> Result<()> {
        unsafe { self.target.GetScissorRect(prect) }
    }

    #[instrument(err, ret, level = "trace")]
    fn SetSoftwareVertexProcessing(&self, bsoftware: BOOL) -> Result<()> {
        unsafe { self.target.SetSoftwareVertexProcessing(bsoftware.into()) }
    }

    #[instrument(ret, level = "trace")]
    fn GetSoftwareVertexProcessing(&self) -> BOOL {
        unsafe { self.target.GetSoftwareVertexProcessing() }
    }

    #[instrument(err, ret, level = "trace")]
    fn SetNPatchMode(&self, nsegments: f32) -> Result<()> {
        unsafe { self.target.SetNPatchMode(nsegments) }
    }

    #[instrument(ret, level = "trace")]
    fn GetNPatchMode(&self) -> f32 {
        unsafe { self.target.GetNPatchMode() }
    }

    #[instrument(err, ret, level = "trace")]
    fn DrawPrimitive(&self, primitivetype: D3DPRIMITIVETYPE, startvertex: u32, primitivecount: u32) -> Result<()> {
        unsafe { self.target.DrawPrimitive(primitivetype, startvertex, primitivecount) }
    }

    #[instrument(err, ret, level = "trace")]
    fn DrawIndexedPrimitive(&self, param0: D3DPRIMITIVETYPE, basevertexindex: i32, minvertexindex: u32, numvertices: u32, startindex: u32, primcount: u32) -> Result<()> {
        unsafe { self.target.DrawIndexedPrimitive(param0, basevertexindex, minvertexindex, numvertices, startindex, primcount) }
    }

    #[instrument(err, ret, level = "trace")]
    fn DrawPrimitiveUP(&self, primitivetype: D3DPRIMITIVETYPE, primitivecount: u32, pvertexstreamzerodata: *const c_void, vertexstreamzerostride: u32) -> Result<()> {
        unsafe { self.target.DrawPrimitiveUP(primitivetype, primitivecount, pvertexstreamzerodata, vertexstreamzerostride) }
    }

    #[instrument(err, ret, level = "trace")]
    fn DrawIndexedPrimitiveUP(
        &self,
        primitivetype: D3DPRIMITIVETYPE,
        minvertexindex: u32,
        numvertices: u32,
        primitivecount: u32,
        pindexdata: *const c_void,
        indexdataformat: D3DFORMAT,
        pvertexstreamzerodata: *const c_void,
        vertexstreamzerostride: u32,
    ) -> Result<()> {
        unsafe {
            self.target.DrawIndexedPrimitiveUP(
                primitivetype,
                minvertexindex,
                numvertices,
                primitivecount,
                pindexdata,
                indexdataformat,
                pvertexstreamzerodata,
                vertexstreamzerostride,
            )
        }
    }

    #[instrument(err, ret, level = "trace", skip(pdestbuffer, pvertexdecl))]
    fn ProcessVertices(&self, srcstartindex: u32, destindex: u32, vertexcount: u32, pdestbuffer: Ref<IDirect3DVertexBuffer9>, pvertexdecl: Ref<IDirect3DVertexDeclaration9>, flags: u32) -> Result<()> {
        let target_dest = self.context.get_target_nullable(pdestbuffer).ok_or(D3DERR_INVALIDCALL)?;
        let target_decl = self.context.get_target_nullable(pvertexdecl).ok_or(D3DERR_INVALIDCALL)?;
        unsafe { self.target.ProcessVertices(srcstartindex, destindex, vertexcount, target_dest, target_decl, flags) }
    }

    #[instrument(err, ret, level = "trace")]
    fn CreateVertexDeclaration(&self, pvertexelements: *const D3DVERTEXELEMENT9) -> Result<IDirect3DVertexDeclaration9> {
        unsafe { self.CreateVertexDeclaration_Impl(|| self.to_interface(), pvertexelements) }
    }

    #[instrument(err, ret, level = "trace", skip(pdecl))]
    fn SetVertexDeclaration(&self, pdecl: Ref<IDirect3DVertexDeclaration9>) -> Result<()> {
        let target = self.context.get_target_nullable(pdecl).ok_or(D3DERR_INVALIDCALL)?;
        unsafe { self.target.SetVertexDeclaration(target) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetVertexDeclaration(&self) -> Result<IDirect3DVertexDeclaration9> {
        unsafe { self.GetVertexDeclaration_Impl(|| self.to_interface()) }
    }

    #[instrument(err, ret, level = "trace")]
    fn SetFVF(&self, fvf: u32) -> Result<()> {
        unsafe { self.target.SetFVF(fvf) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetFVF(&self, pfvf: *mut u32) -> Result<()> {
        unsafe { self.target.GetFVF(pfvf) }
    }

    #[instrument(err, ret, level = "trace")]
    fn CreateVertexShader(&self, pfunction: *const u32) -> Result<IDirect3DVertexShader9> {
        unsafe { self.CreateVertexShader_Impl(|| self.to_interface(), pfunction) }
    }

    #[instrument(err, ret, level = "trace", skip(pshader))]
    fn SetVertexShader(&self, pshader: Ref<IDirect3DVertexShader9>) -> Result<()> {
        let target = self.context.get_target_nullable(pshader).ok_or(D3DERR_INVALIDCALL)?;
        unsafe { self.target.SetVertexShader(target) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetVertexShader(&self) -> Result<IDirect3DVertexShader9> {
        unsafe { self.GetVertexShader_Impl(|| self.to_interface()) }
    }

    #[instrument(err, ret, level = "trace")]
    fn SetVertexShaderConstantF(&self, startregister: u32, pconstantdata: *const f32, vector4fcount: u32) -> Result<()> {
        unsafe { self.target.SetVertexShaderConstantF(startregister, pconstantdata, vector4fcount) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetVertexShaderConstantF(&self, startregister: u32, pconstantdata: *mut f32, vector4fcount: u32) -> Result<()> {
        unsafe { self.target.GetVertexShaderConstantF(startregister, pconstantdata, vector4fcount) }
    }

    #[instrument(err, ret, level = "trace")]
    fn SetVertexShaderConstantI(&self, startregister: u32, pconstantdata: *const i32, vector4icount: u32) -> Result<()> {
        unsafe { self.target.SetVertexShaderConstantI(startregister, pconstantdata, vector4icount) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetVertexShaderConstantI(&self, startregister: u32, pconstantdata: *mut i32, vector4icount: u32) -> Result<()> {
        unsafe { self.target.GetVertexShaderConstantI(startregister, pconstantdata, vector4icount) }
    }

    #[instrument(err, ret, level = "trace")]
    fn SetVertexShaderConstantB(&self, startregister: u32, pconstantdata: *const BOOL, boolcount: u32) -> Result<()> {
        unsafe { self.target.SetVertexShaderConstantB(startregister, pconstantdata, boolcount) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetVertexShaderConstantB(&self, startregister: u32, pconstantdata: *mut BOOL, boolcount: u32) -> Result<()> {
        unsafe { self.target.GetVertexShaderConstantB(startregister, pconstantdata, boolcount) }
    }

    #[instrument(err, ret, level = "trace", skip(pstreamdata))]
    fn SetStreamSource(&self, streamnumber: u32, pstreamdata: Ref<IDirect3DVertexBuffer9>, offsetinbytes: u32, stride: u32) -> Result<()> {
        let target = self.context.get_target_nullable(pstreamdata).ok_or(D3DERR_INVALIDCALL)?;
        unsafe { self.target.SetStreamSource(streamnumber, target, offsetinbytes, stride) }
    }

    #[instrument(err, ret, level = "trace", skip(ppstreamdata))]
    fn GetStreamSource(&self, streamnumber: u32, ppstreamdata: OutRef<IDirect3DVertexBuffer9>, poffsetinbytes: *mut u32, pstride: *mut u32) -> Result<()> {
        unsafe { self.GetStreamSource_Impl(|| self.to_interface(), streamnumber, ppstreamdata, poffsetinbytes, pstride) }
    }

    #[instrument(err, ret, level = "trace")]
    fn SetStreamSourceFreq(&self, streamnumber: u32, setting: u32) -> Result<()> {
        unsafe { self.target.SetStreamSourceFreq(streamnumber, setting) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetStreamSourceFreq(&self, streamnumber: u32, psetting: *mut u32) -> Result<()> {
        unsafe { self.target.GetStreamSourceFreq(streamnumber, psetting) }
    }

    #[instrument(err, ret, level = "trace", skip(pindexdata))]
    fn SetIndices(&self, pindexdata: Ref<IDirect3DIndexBuffer9>) -> Result<()> {
        let target = self.context.get_target_nullable(pindexdata).ok_or(D3DERR_INVALIDCALL)?;
        unsafe { self.target.SetIndices(target) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetIndices(&self) -> Result<IDirect3DIndexBuffer9> {
        unsafe { self.GetIndices_Impl(|| self.to_interface()) }
    }

    #[instrument(err, ret, level = "trace")]
    fn CreatePixelShader(&self, pfunction: *const u32) -> Result<IDirect3DPixelShader9> {
        unsafe { self.CreatePixelShader_Impl(|| self.to_interface(), pfunction) }
    }

    #[instrument(err, ret, level = "trace", skip(pshader))]
    fn SetPixelShader(&self, pshader: Ref<IDirect3DPixelShader9>) -> Result<()> {
        let target = self.context.get_target_nullable(pshader).ok_or(D3DERR_INVALIDCALL)?;
        unsafe { self.target.SetPixelShader(target) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetPixelShader(&self) -> Result<IDirect3DPixelShader9> {
        unsafe { self.GetPixelShader_Impl(|| self.to_interface()) }
    }

    #[instrument(err, ret, level = "trace")]
    fn SetPixelShaderConstantF(&self, startregister: u32, pconstantdata: *const f32, vector4fcount: u32) -> Result<()> {
        unsafe { self.target.SetPixelShaderConstantF(startregister, pconstantdata, vector4fcount) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetPixelShaderConstantF(&self, startregister: u32, pconstantdata: *mut f32, vector4fcount: u32) -> Result<()> {
        unsafe { self.target.GetPixelShaderConstantF(startregister, pconstantdata, vector4fcount) }
    }

    #[instrument(err, ret, level = "trace")]
    fn SetPixelShaderConstantI(&self, startregister: u32, pconstantdata: *const i32, vector4icount: u32) -> Result<()> {
        unsafe { self.target.SetPixelShaderConstantI(startregister, pconstantdata, vector4icount) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetPixelShaderConstantI(&self, startregister: u32, pconstantdata: *mut i32, vector4icount: u32) -> Result<()> {
        unsafe { self.target.GetPixelShaderConstantI(startregister, pconstantdata, vector4icount) }
    }

    #[instrument(err, ret, level = "trace")]
    fn SetPixelShaderConstantB(&self, startregister: u32, pconstantdata: *const BOOL, boolcount: u32) -> Result<()> {
        unsafe { self.target.SetPixelShaderConstantB(startregister, pconstantdata, boolcount) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetPixelShaderConstantB(&self, startregister: u32, pconstantdata: *mut BOOL, boolcount: u32) -> Result<()> {
        unsafe { self.target.GetPixelShaderConstantB(startregister, pconstantdata, boolcount) }
    }

    #[instrument(err, ret, level = "trace")]
    fn DrawRectPatch(&self, handle: u32, pnumsegs: *const f32, prectpatchinfo: *const D3DRECTPATCH_INFO) -> Result<()> {
        unsafe { self.target.DrawRectPatch(handle, pnumsegs, prectpatchinfo) }
    }

    #[instrument(err, ret, level = "trace")]
    fn DrawTriPatch(&self, handle: u32, pnumsegs: *const f32, ptripatchinfo: *const D3DTRIPATCH_INFO) -> Result<()> {
        unsafe { self.target.DrawTriPatch(handle, pnumsegs, ptripatchinfo) }
    }

    #[instrument(err, ret, level = "trace")]
    fn DeletePatch(&self, handle: u32) -> Result<()> {
        unsafe { self.target.DeletePatch(handle) }
    }

    #[instrument(err, ret, level = "trace")]
    fn CreateQuery(&self, r#type: D3DQUERYTYPE) -> Result<IDirect3DQuery9> {
        unsafe { self.CreateQuery_Impl(|| self.to_interface(), r#type) }
    }
}
