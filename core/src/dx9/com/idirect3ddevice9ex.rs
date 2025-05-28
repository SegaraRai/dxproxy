//! [`IDirect3DDevice9Ex`] proxy implementation.
//!
//! This module provides a proxy wrapper for the IDirect3DDevice9Ex interface,
//! which extends IDirect3DDevice9 with additional functionality for Windows Vista
//! and later, including improved resource management and presentation features.

use super::*;
use std::{
    ffi::c_void,
    mem::{transmute, transmute_copy},
    slice::from_raw_parts,
};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::{Direct3D9::*, Gdi::*},
    },
    core::*,
};
use windows_numerics::Matrix4x4;

/// Proxy wrapper for [`IDirect3DDevice9Ex`] interface.
///
/// Extends [`IDirect3DDevice9`] functionality with Windows Vista+ features while maintaining
/// a device context for state tracking. Intercepts Extended Direct3D device operations
/// including resource residency checks, presentation controls, and GPU priority management.
///
/// Methods of [`IDirect3DDevice9`] are delegated to the inner [`IDirect3DDevice9`] proxy, which is implemented by [`ProxyDirect3DDevice9`].
#[implement(IDirect3DDevice9Ex)]
#[derive(Debug)]
pub struct ProxyDirect3DDevice9Ex {
    proxy: ComObject<ProxyDirect3DDevice9>,
    target: IDirect3DDevice9Ex,
    context: DX9ProxyDeviceContext,
}

impl ProxyDirect3DDevice9Ex {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret))]
    pub fn new(target: IDirect3DDevice9Ex, config: DX9ProxyConfig, container: IDirect3D9Ex) -> Self {
        let proxy = ProxyDirect3DDevice9::new(target.clone().into(), config, container.into());
        let context = proxy.get_context().clone();

        Self { proxy: proxy.into(), target, context }
    }
}

impl Drop for ProxyDirect3DDevice9Ex {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret))]
    fn drop(&mut self) {}
}

impl_debug!(ProxyDirect3DDevice9Ex_Impl);

#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]
impl IDirect3DDevice9Ex_Impl for ProxyDirect3DDevice9Ex_Impl {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn CheckDeviceState(&self, hdestinationwindow: HWND) -> Result<()> {
        unsafe { self.target.CheckDeviceState(hdestinationwindow) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace", skip(presourcearray)))]
    fn CheckResourceResidency(&self, presourcearray: OutRef<IDirect3DResource9>, numresources: u32) -> Result<()> {
        let proxies: &[Option<&IDirect3DResource9>] = unsafe { from_raw_parts(transmute_copy(&presourcearray), numresources as usize) };
        let targets = proxies
            .iter()
            .map(|proxy| self.context.get_target_nullable(*proxy).ok_or(D3DERR_INVALIDCALL.into()))
            .collect::<Result<Vec<_>>>()?;
        unsafe {
            #[allow(clippy::missing_transmute_annotations)]
            self.target.CheckResourceResidency(transmute(targets.as_ptr()), numresources)
        }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace", skip(psrc, pdst, psrcrectdescs, pdstrectdescs)))]
    fn ComposeRects(
        &self,
        psrc: Ref<IDirect3DSurface9>,
        pdst: Ref<IDirect3DSurface9>,
        psrcrectdescs: Ref<IDirect3DVertexBuffer9>,
        numrects: u32,
        pdstrectdescs: Ref<IDirect3DVertexBuffer9>,
        operation: D3DCOMPOSERECTSOP,
        xoffset: i32,
        yoffset: i32,
    ) -> Result<()> {
        let target_src = self.context.get_target_nullable(psrc).ok_or(D3DERR_INVALIDCALL)?;
        let target_dest = self.context.get_target_nullable(pdst).ok_or(D3DERR_INVALIDCALL)?;
        let target_src_descs = self.context.get_target_nullable(psrcrectdescs).ok_or(D3DERR_INVALIDCALL)?;
        let target_dst_descs = self.context.get_target_nullable(pdstrectdescs).ok_or(D3DERR_INVALIDCALL)?;

        unsafe {
            self.target
                .ComposeRects(target_src, target_dest, target_src_descs, numrects, target_dst_descs, operation, xoffset, yoffset)
        }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace", skip(ppsurface)))]
    fn CreateDepthStencilSurfaceEx(
        &self,
        width: u32,
        height: u32,
        format: D3DFORMAT,
        multisample: D3DMULTISAMPLE_TYPE,
        multisamplequality: u32,
        discard: BOOL,
        ppsurface: OutRef<IDirect3DSurface9>,
        psharedhandle: *mut HANDLE,
        usage: u32,
    ) -> Result<()> {
        check_nullptr!(ppsurface);

        let target = try_out_param(|out| unsafe {
            self.target
                .CreateDepthStencilSurfaceEx(width, height, format, multisample, multisamplequality, discard.into(), out, psharedhandle, usage)
        })?;
        let proxy = self.context.ensure_proxy(target, |target| {
            ProxyDirect3DSurface9::new(target, self.context.clone(), self.to_interface::<IDirect3DDevice9Ex>().into(), DX9SurfaceContainer::Standalone).into()
        });
        ppsurface.write(Some(proxy))
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace", skip(ppsurface)))]
    fn CreateOffscreenPlainSurfaceEx(&self, width: u32, height: u32, format: D3DFORMAT, pool: D3DPOOL, ppsurface: OutRef<IDirect3DSurface9>, psharedhandle: *mut HANDLE, usage: u32) -> Result<()> {
        check_nullptr!(ppsurface);

        let target = try_out_param(|out| unsafe { self.target.CreateOffscreenPlainSurfaceEx(width, height, format, pool, out, psharedhandle, usage) })?;
        let proxy = self.context.ensure_proxy(target, |target| {
            ProxyDirect3DSurface9::new(target, self.context.clone(), self.to_interface::<IDirect3DDevice9Ex>().into(), DX9SurfaceContainer::Standalone).into()
        });
        ppsurface.write(Some(proxy))
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace", skip(ppsurface)))]
    fn CreateRenderTargetEx(
        &self,
        width: u32,
        height: u32,
        format: D3DFORMAT,
        multisample: D3DMULTISAMPLE_TYPE,
        multisamplequality: u32,
        lockable: BOOL,
        ppsurface: OutRef<IDirect3DSurface9>,
        psharedhandle: *mut HANDLE,
        usage: u32,
    ) -> Result<()> {
        check_nullptr!(ppsurface);

        let target = try_out_param(|out| unsafe {
            self.target
                .CreateRenderTargetEx(width, height, format, multisample, multisamplequality, lockable.into(), out, psharedhandle, usage)
        })?;
        let proxy = self.context.ensure_proxy(target, |target| {
            ProxyDirect3DSurface9::new(target, self.context.clone(), self.to_interface::<IDirect3DDevice9Ex>().into(), DX9SurfaceContainer::Standalone).into()
        });
        ppsurface.write(Some(proxy))
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn PresentEx(&self, psourcerect: *const RECT, pdestrect: *const RECT, hdestwindowoverride: HWND, pdirtyregion: *const RGNDATA, dwflags: u32) -> Result<()> {
        unsafe { self.target.PresentEx(psourcerect, pdestrect, hdestwindowoverride, pdirtyregion, dwflags) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn ResetEx(&self, ppresentationparameters: *mut D3DPRESENT_PARAMETERS, pfullscreendisplaymode: *mut D3DDISPLAYMODEEX) -> Result<()> {
        unsafe { self.target.ResetEx(ppresentationparameters, pfullscreendisplaymode) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn GetDisplayModeEx(&self, iswapchain: u32, pmode: *mut D3DDISPLAYMODEEX, protation: *mut D3DDISPLAYROTATION) -> Result<()> {
        unsafe { self.target.GetDisplayModeEx(iswapchain, pmode, protation) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn SetConvolutionMonoKernel(&self, width: u32, height: u32, rows: *mut f32, columns: *mut f32) -> Result<()> {
        unsafe { self.target.SetConvolutionMonoKernel(width, height, rows, columns) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn SetGPUThreadPriority(&self, priority: i32) -> Result<()> {
        unsafe { self.target.SetGPUThreadPriority(priority) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn GetGPUThreadPriority(&self, ppriority: *mut i32) -> Result<()> {
        unsafe { self.target.GetGPUThreadPriority(ppriority) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn SetMaximumFrameLatency(&self, maxlatency: u32) -> Result<()> {
        unsafe { self.target.SetMaximumFrameLatency(maxlatency) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn GetMaximumFrameLatency(&self, pmaxlatency: *mut u32) -> Result<()> {
        unsafe { self.target.GetMaximumFrameLatency(pmaxlatency) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn WaitForVBlank(&self, iswapchain: u32) -> Result<()> {
        unsafe { self.target.WaitForVBlank(iswapchain) }
    }
}

macro_rules! proxy_as_interface {
    ($this:ident) => {
        $this.proxy.as_interface::<IDirect3DDevice9>()
    };
}

macro_rules! get_base_interface_fn {
    ($this:ident) => {
        || $this.to_interface::<IDirect3DDevice9Ex>().into()
    };
}

/// Implementation of [`IDirect3DDevice9`] for [`ProxyDirect3DDevice9Ex`].
///
/// Most methods delegate to the inner [`IDirect3DDevice9`] proxy. However, for methods that need to pass
/// a COM interface pointer of `self`, use the corresponding `*_Impl` methods from [`ProxyDirect3DDevice9_Impl`]
/// when available. Check the base implementation to determine if a `*_Impl` variant exists before
/// delegating directly to avoid interface inconsistencies in inheritance scenarios.
///
/// We should not customize methods here, since [`ProxyDirect3DDevice9Ex::proxy`] points to an [`IDirect3DDevice9`] object of [`ProxyDirect3DDevice9`].
#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]
impl IDirect3DDevice9_Impl for ProxyDirect3DDevice9Ex_Impl {
    fn TestCooperativeLevel(&self) -> Result<()> {
        unsafe { proxy_as_interface!(self).TestCooperativeLevel() }
    }

    fn GetAvailableTextureMem(&self) -> u32 {
        unsafe { proxy_as_interface!(self).GetAvailableTextureMem() }
    }

    fn EvictManagedResources(&self) -> Result<()> {
        unsafe { proxy_as_interface!(self).EvictManagedResources() }
    }

    fn GetDirect3D(&self) -> Result<IDirect3D9> {
        unsafe { proxy_as_interface!(self).GetDirect3D() }
    }

    fn GetDeviceCaps(&self, pcaps: *mut D3DCAPS9) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetDeviceCaps(pcaps) }
    }

    fn GetDisplayMode(&self, iswapchain: u32, pmode: *mut D3DDISPLAYMODE) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetDisplayMode(iswapchain, pmode) }
    }

    fn GetCreationParameters(&self, pparameters: *mut D3DDEVICE_CREATION_PARAMETERS) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetCreationParameters(pparameters) }
    }

    fn SetCursorProperties(&self, xhotspot: u32, yhotspot: u32, pcursorbitmap: Ref<IDirect3DSurface9>) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetCursorProperties(xhotspot, yhotspot, pcursorbitmap.as_ref()) }
    }

    fn SetCursorPosition(&self, x: i32, y: i32, flags: u32) {
        unsafe { proxy_as_interface!(self).SetCursorPosition(x, y, flags) }
    }

    fn ShowCursor(&self, bshow: BOOL) -> BOOL {
        unsafe { proxy_as_interface!(self).ShowCursor(bshow.into()) }
    }

    fn CreateAdditionalSwapChain(&self, ppresentationparameters: *mut D3DPRESENT_PARAMETERS, pswapchain: OutRef<IDirect3DSwapChain9>) -> Result<()> {
        unsafe { self.proxy.CreateAdditionalSwapChain_Impl(get_base_interface_fn!(self), ppresentationparameters, pswapchain) }
    }

    fn GetSwapChain(&self, iswapchain: u32) -> Result<IDirect3DSwapChain9> {
        unsafe { self.proxy.GetSwapChain_Impl(get_base_interface_fn!(self), iswapchain) }
    }

    fn GetNumberOfSwapChains(&self) -> u32 {
        unsafe { proxy_as_interface!(self).GetNumberOfSwapChains() }
    }

    fn Reset(&self, ppresentationparameters: *mut D3DPRESENT_PARAMETERS) -> Result<()> {
        unsafe { proxy_as_interface!(self).Reset(ppresentationparameters) }
    }

    fn Present(&self, psourcerect: *const RECT, pdestrect: *const RECT, hdestwindowoverride: HWND, pdirtyregion: *const RGNDATA) -> Result<()> {
        unsafe { proxy_as_interface!(self).Present(psourcerect, pdestrect, hdestwindowoverride, pdirtyregion) }
    }

    fn GetBackBuffer(&self, iswapchain: u32, ibackbuffer: u32, r#type: D3DBACKBUFFER_TYPE) -> Result<IDirect3DSurface9> {
        unsafe { self.proxy.GetBackBuffer_Impl(get_base_interface_fn!(self), iswapchain, ibackbuffer, r#type) }
    }

    fn GetRasterStatus(&self, iswapchain: u32, prasterstatus: *mut D3DRASTER_STATUS) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetRasterStatus(iswapchain, prasterstatus) }
    }

    fn SetDialogBoxMode(&self, benabledialogs: BOOL) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetDialogBoxMode(benabledialogs.into()) }
    }

    fn SetGammaRamp(&self, iswapchain: u32, flags: u32, pramp: *const D3DGAMMARAMP) {
        unsafe { proxy_as_interface!(self).SetGammaRamp(iswapchain, flags, pramp) }
    }

    fn GetGammaRamp(&self, iswapchain: u32, pramp: *mut D3DGAMMARAMP) {
        unsafe { proxy_as_interface!(self).GetGammaRamp(iswapchain, pramp) }
    }

    fn CreateTexture(&self, width: u32, height: u32, levels: u32, usage: u32, format: D3DFORMAT, pool: D3DPOOL, pptexture: OutRef<IDirect3DTexture9>, psharedhandle: *mut HANDLE) -> Result<()> {
        unsafe {
            self.proxy
                .CreateTexture_Impl(get_base_interface_fn!(self), width, height, levels, usage, format, pool, pptexture, psharedhandle)
        }
    }

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
        unsafe {
            self.proxy
                .CreateVolumeTexture_Impl(get_base_interface_fn!(self), width, height, depth, levels, usage, format, pool, ppvolumetexture, psharedhandle)
        }
    }

    fn CreateCubeTexture(&self, edgelength: u32, levels: u32, usage: u32, format: D3DFORMAT, pool: D3DPOOL, ppcubetexture: OutRef<IDirect3DCubeTexture9>, psharedhandle: *mut HANDLE) -> Result<()> {
        unsafe {
            self.proxy
                .CreateCubeTexture_Impl(get_base_interface_fn!(self), edgelength, levels, usage, format, pool, ppcubetexture, psharedhandle)
        }
    }

    fn CreateVertexBuffer(&self, length: u32, usage: u32, fvf: u32, pool: D3DPOOL, ppvertexbuffer: OutRef<IDirect3DVertexBuffer9>, psharedhandle: *mut HANDLE) -> Result<()> {
        unsafe {
            self.proxy
                .CreateVertexBuffer_Impl(get_base_interface_fn!(self), length, usage, fvf, pool, ppvertexbuffer, psharedhandle)
        }
    }

    fn CreateIndexBuffer(&self, length: u32, usage: u32, format: D3DFORMAT, pool: D3DPOOL, ppindexbuffer: OutRef<IDirect3DIndexBuffer9>, psharedhandle: *mut HANDLE) -> Result<()> {
        unsafe {
            self.proxy
                .CreateIndexBuffer_Impl(get_base_interface_fn!(self), length, usage, format, pool, ppindexbuffer, psharedhandle)
        }
    }

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
        unsafe {
            self.proxy
                .CreateDepthStencilSurface_Impl(get_base_interface_fn!(self), width, height, format, multisample, multisamplequality, discard, ppsurface, psharedhandle)
        }
    }

    fn CreateOffscreenPlainSurface(&self, width: u32, height: u32, format: D3DFORMAT, pool: D3DPOOL, ppsurface: OutRef<IDirect3DSurface9>, psharedhandle: *mut HANDLE) -> Result<()> {
        unsafe {
            self.proxy
                .CreateOffscreenPlainSurface_Impl(get_base_interface_fn!(self), width, height, format, pool, ppsurface, psharedhandle)
        }
    }

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
        unsafe {
            self.proxy
                .CreateRenderTarget_Impl(get_base_interface_fn!(self), width, height, format, multisample, multisamplequality, lockable, ppsurface, psharedhandle)
        }
    }

    fn UpdateSurface(&self, psourcesurface: Ref<IDirect3DSurface9>, psourcerect: *const RECT, pdestinationsurface: Ref<IDirect3DSurface9>, pdestpoint: *const POINT) -> Result<()> {
        unsafe { proxy_as_interface!(self).UpdateSurface(psourcesurface.as_ref(), psourcerect, pdestinationsurface.as_ref(), pdestpoint) }
    }

    fn UpdateTexture(&self, psourcetexture: Ref<IDirect3DBaseTexture9>, pdestinationtexture: Ref<IDirect3DBaseTexture9>) -> Result<()> {
        unsafe { proxy_as_interface!(self).UpdateTexture(psourcetexture.as_ref(), pdestinationtexture.as_ref()) }
    }

    fn GetRenderTargetData(&self, prendertarget: Ref<IDirect3DSurface9>, pdestsurface: Ref<IDirect3DSurface9>) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetRenderTargetData(prendertarget.as_ref(), pdestsurface.as_ref()) }
    }

    fn GetFrontBufferData(&self, iswapchain: u32, pdestsurface: Ref<IDirect3DSurface9>) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetFrontBufferData(iswapchain, pdestsurface.as_ref()) }
    }

    fn StretchRect(&self, psourcesurface: Ref<IDirect3DSurface9>, psourcerect: *const RECT, pdestsurface: Ref<IDirect3DSurface9>, pdestrect: *const RECT, filter: D3DTEXTUREFILTERTYPE) -> Result<()> {
        unsafe { proxy_as_interface!(self).StretchRect(psourcesurface.as_ref(), psourcerect, pdestsurface.as_ref(), pdestrect, filter) }
    }

    fn ColorFill(&self, psurface: Ref<IDirect3DSurface9>, prect: *const RECT, color: u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).ColorFill(psurface.as_ref(), prect, color) }
    }

    fn SetRenderTarget(&self, rendertargetindex: u32, prendertarget: Ref<IDirect3DSurface9>) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetRenderTarget(rendertargetindex, prendertarget.as_ref()) }
    }

    fn GetRenderTarget(&self, rendertargetindex: u32) -> Result<IDirect3DSurface9> {
        unsafe { self.proxy.GetRenderTarget_Impl(get_base_interface_fn!(self), rendertargetindex) }
    }

    fn SetDepthStencilSurface(&self, pnewzstencil: Ref<IDirect3DSurface9>) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetDepthStencilSurface(pnewzstencil.as_ref()) }
    }

    fn GetDepthStencilSurface(&self) -> Result<IDirect3DSurface9> {
        unsafe { self.proxy.GetDepthStencilSurface_Impl(get_base_interface_fn!(self)) }
    }

    fn BeginScene(&self) -> Result<()> {
        unsafe { proxy_as_interface!(self).BeginScene() }
    }

    fn EndScene(&self) -> Result<()> {
        unsafe { proxy_as_interface!(self).EndScene() }
    }

    fn Clear(&self, count: u32, prects: *const D3DRECT, flags: u32, color: u32, z: f32, stencil: u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).Clear(count, prects, flags, color, z, stencil) }
    }

    fn SetTransform(&self, state: D3DTRANSFORMSTATETYPE, pmatrix: *const Matrix4x4) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetTransform(state, pmatrix) }
    }

    fn GetTransform(&self, state: D3DTRANSFORMSTATETYPE, pmatrix: *mut Matrix4x4) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetTransform(state, pmatrix) }
    }

    fn MultiplyTransform(&self, param0: D3DTRANSFORMSTATETYPE, param1: *const Matrix4x4) -> Result<()> {
        unsafe { proxy_as_interface!(self).MultiplyTransform(param0, param1) }
    }

    fn SetViewport(&self, pviewport: *const D3DVIEWPORT9) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetViewport(pviewport) }
    }

    fn GetViewport(&self, pviewport: *mut D3DVIEWPORT9) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetViewport(pviewport) }
    }

    fn SetMaterial(&self, pmaterial: *const D3DMATERIAL9) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetMaterial(pmaterial) }
    }

    fn GetMaterial(&self, pmaterial: *mut D3DMATERIAL9) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetMaterial(pmaterial) }
    }

    fn SetLight(&self, index: u32, param1: *const D3DLIGHT9) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetLight(index, param1) }
    }

    fn GetLight(&self, index: u32, param1: *mut D3DLIGHT9) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetLight(index, param1) }
    }

    fn LightEnable(&self, index: u32, enable: BOOL) -> Result<()> {
        unsafe { proxy_as_interface!(self).LightEnable(index, enable.into()) }
    }

    fn GetLightEnable(&self, index: u32, penable: *mut BOOL) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetLightEnable(index, penable) }
    }

    fn SetClipPlane(&self, index: u32, pplane: *const f32) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetClipPlane(index, pplane) }
    }

    fn GetClipPlane(&self, index: u32, pplane: *mut f32) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetClipPlane(index, pplane) }
    }

    fn SetRenderState(&self, state: D3DRENDERSTATETYPE, value: u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetRenderState(state, value) }
    }

    fn GetRenderState(&self, state: D3DRENDERSTATETYPE, pvalue: *mut u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetRenderState(state, pvalue) }
    }

    fn CreateStateBlock(&self, r#type: D3DSTATEBLOCKTYPE) -> Result<IDirect3DStateBlock9> {
        unsafe { self.proxy.CreateStateBlock_Impl(get_base_interface_fn!(self), r#type) }
    }

    fn BeginStateBlock(&self) -> Result<()> {
        unsafe { proxy_as_interface!(self).BeginStateBlock() }
    }

    fn EndStateBlock(&self) -> Result<IDirect3DStateBlock9> {
        unsafe { self.proxy.EndStateBlock_Impl(get_base_interface_fn!(self)) }
    }

    fn SetClipStatus(&self, pclipstatus: *const D3DCLIPSTATUS9) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetClipStatus(pclipstatus) }
    }

    fn GetClipStatus(&self, pclipstatus: *mut D3DCLIPSTATUS9) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetClipStatus(pclipstatus) }
    }

    fn GetTexture(&self, stage: u32) -> Result<IDirect3DBaseTexture9> {
        unsafe { proxy_as_interface!(self).GetTexture(stage) }
    }

    fn SetTexture(&self, stage: u32, ptexture: Ref<IDirect3DBaseTexture9>) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetTexture(stage, ptexture.as_ref()) }
    }

    fn GetTextureStageState(&self, stage: u32, r#type: D3DTEXTURESTAGESTATETYPE, pvalue: *mut u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetTextureStageState(stage, r#type, pvalue) }
    }

    fn SetTextureStageState(&self, stage: u32, r#type: D3DTEXTURESTAGESTATETYPE, value: u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetTextureStageState(stage, r#type, value) }
    }

    fn GetSamplerState(&self, sampler: u32, r#type: D3DSAMPLERSTATETYPE, pvalue: *mut u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetSamplerState(sampler, r#type, pvalue) }
    }

    fn SetSamplerState(&self, sampler: u32, r#type: D3DSAMPLERSTATETYPE, value: u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetSamplerState(sampler, r#type, value) }
    }

    fn ValidateDevice(&self, pnumpasses: *mut u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).ValidateDevice(pnumpasses) }
    }

    fn SetPaletteEntries(&self, palettenumber: u32, pentries: *const PALETTEENTRY) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetPaletteEntries(palettenumber, pentries) }
    }

    fn GetPaletteEntries(&self, palettenumber: u32, pentries: *mut PALETTEENTRY) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetPaletteEntries(palettenumber, pentries) }
    }

    fn SetCurrentTexturePalette(&self, palettenumber: u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetCurrentTexturePalette(palettenumber) }
    }

    fn GetCurrentTexturePalette(&self, ppalettenumber: *mut u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetCurrentTexturePalette(ppalettenumber) }
    }

    fn SetScissorRect(&self, prect: *const RECT) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetScissorRect(prect) }
    }

    fn GetScissorRect(&self, prect: *mut RECT) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetScissorRect(prect) }
    }

    fn SetSoftwareVertexProcessing(&self, bsoftware: BOOL) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetSoftwareVertexProcessing(bsoftware.into()) }
    }

    fn GetSoftwareVertexProcessing(&self) -> BOOL {
        unsafe { proxy_as_interface!(self).GetSoftwareVertexProcessing() }
    }

    fn SetNPatchMode(&self, nsegments: f32) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetNPatchMode(nsegments) }
    }

    fn GetNPatchMode(&self) -> f32 {
        unsafe { proxy_as_interface!(self).GetNPatchMode() }
    }

    fn DrawPrimitive(&self, primitivetype: D3DPRIMITIVETYPE, startvertex: u32, primitivecount: u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).DrawPrimitive(primitivetype, startvertex, primitivecount) }
    }

    fn DrawIndexedPrimitive(&self, param0: D3DPRIMITIVETYPE, basevertexindex: i32, minvertexindex: u32, numvertices: u32, startindex: u32, primcount: u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).DrawIndexedPrimitive(param0, basevertexindex, minvertexindex, numvertices, startindex, primcount) }
    }

    fn DrawPrimitiveUP(&self, primitivetype: D3DPRIMITIVETYPE, primitivecount: u32, pvertexstreamzerodata: *const c_void, vertexstreamzerostride: u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).DrawPrimitiveUP(primitivetype, primitivecount, pvertexstreamzerodata, vertexstreamzerostride) }
    }

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
            proxy_as_interface!(self).DrawIndexedPrimitiveUP(
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

    fn ProcessVertices(&self, srcstartindex: u32, destindex: u32, vertexcount: u32, pdestbuffer: Ref<IDirect3DVertexBuffer9>, pvertexdecl: Ref<IDirect3DVertexDeclaration9>, flags: u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).ProcessVertices(srcstartindex, destindex, vertexcount, pdestbuffer.as_ref(), pvertexdecl.as_ref(), flags) }
    }

    fn CreateVertexDeclaration(&self, pvertexelements: *const D3DVERTEXELEMENT9) -> Result<IDirect3DVertexDeclaration9> {
        unsafe { self.proxy.CreateVertexDeclaration_Impl(get_base_interface_fn!(self), pvertexelements) }
    }

    fn SetVertexDeclaration(&self, pdecl: Ref<IDirect3DVertexDeclaration9>) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetVertexDeclaration(pdecl.as_ref()) }
    }

    fn GetVertexDeclaration(&self) -> Result<IDirect3DVertexDeclaration9> {
        unsafe { self.proxy.GetVertexDeclaration_Impl(get_base_interface_fn!(self)) }
    }

    fn SetFVF(&self, fvf: u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetFVF(fvf) }
    }

    fn GetFVF(&self, pfvf: *mut u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetFVF(pfvf) }
    }

    fn CreateVertexShader(&self, pfunction: *const u32) -> Result<IDirect3DVertexShader9> {
        unsafe { self.proxy.CreateVertexShader_Impl(get_base_interface_fn!(self), pfunction) }
    }

    fn SetVertexShader(&self, pshader: Ref<IDirect3DVertexShader9>) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetVertexShader(pshader.as_ref()) }
    }

    fn GetVertexShader(&self) -> Result<IDirect3DVertexShader9> {
        unsafe { self.proxy.GetVertexShader_Impl(get_base_interface_fn!(self)) }
    }

    fn SetVertexShaderConstantF(&self, startregister: u32, pconstantdata: *const f32, vector4fcount: u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetVertexShaderConstantF(startregister, pconstantdata, vector4fcount) }
    }

    fn GetVertexShaderConstantF(&self, startregister: u32, pconstantdata: *mut f32, vector4fcount: u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetVertexShaderConstantF(startregister, pconstantdata, vector4fcount) }
    }

    fn SetVertexShaderConstantI(&self, startregister: u32, pconstantdata: *const i32, vector4icount: u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetVertexShaderConstantI(startregister, pconstantdata, vector4icount) }
    }

    fn GetVertexShaderConstantI(&self, startregister: u32, pconstantdata: *mut i32, vector4icount: u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetVertexShaderConstantI(startregister, pconstantdata, vector4icount) }
    }

    fn SetVertexShaderConstantB(&self, startregister: u32, pconstantdata: *const BOOL, boolcount: u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetVertexShaderConstantB(startregister, pconstantdata, boolcount) }
    }

    fn GetVertexShaderConstantB(&self, startregister: u32, pconstantdata: *mut BOOL, boolcount: u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetVertexShaderConstantB(startregister, pconstantdata, boolcount) }
    }

    fn SetStreamSource(&self, streamnumber: u32, pstreamdata: Ref<IDirect3DVertexBuffer9>, offsetinbytes: u32, stride: u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetStreamSource(streamnumber, pstreamdata.as_ref(), offsetinbytes, stride) }
    }

    fn GetStreamSource(&self, streamnumber: u32, ppstreamdata: OutRef<IDirect3DVertexBuffer9>, poffsetinbytes: *mut u32, pstride: *mut u32) -> Result<()> {
        unsafe { self.proxy.GetStreamSource_Impl(get_base_interface_fn!(self), streamnumber, ppstreamdata, poffsetinbytes, pstride) }
    }

    fn SetStreamSourceFreq(&self, streamnumber: u32, setting: u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetStreamSourceFreq(streamnumber, setting) }
    }

    fn GetStreamSourceFreq(&self, streamnumber: u32, psetting: *mut u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetStreamSourceFreq(streamnumber, psetting) }
    }

    fn SetIndices(&self, pindexdata: Ref<IDirect3DIndexBuffer9>) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetIndices(pindexdata.as_ref()) }
    }

    fn GetIndices(&self) -> Result<IDirect3DIndexBuffer9> {
        unsafe { self.proxy.GetIndices_Impl(get_base_interface_fn!(self)) }
    }

    fn CreatePixelShader(&self, pfunction: *const u32) -> Result<IDirect3DPixelShader9> {
        unsafe { self.proxy.CreatePixelShader_Impl(get_base_interface_fn!(self), pfunction) }
    }

    fn SetPixelShader(&self, pshader: Ref<IDirect3DPixelShader9>) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetPixelShader(pshader.as_ref()) }
    }

    fn GetPixelShader(&self) -> Result<IDirect3DPixelShader9> {
        unsafe { self.proxy.GetPixelShader_Impl(get_base_interface_fn!(self)) }
    }

    fn SetPixelShaderConstantF(&self, startregister: u32, pconstantdata: *const f32, vector4fcount: u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetPixelShaderConstantF(startregister, pconstantdata, vector4fcount) }
    }

    fn GetPixelShaderConstantF(&self, startregister: u32, pconstantdata: *mut f32, vector4fcount: u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetPixelShaderConstantF(startregister, pconstantdata, vector4fcount) }
    }

    fn SetPixelShaderConstantI(&self, startregister: u32, pconstantdata: *const i32, vector4icount: u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetPixelShaderConstantI(startregister, pconstantdata, vector4icount) }
    }

    fn GetPixelShaderConstantI(&self, startregister: u32, pconstantdata: *mut i32, vector4icount: u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetPixelShaderConstantI(startregister, pconstantdata, vector4icount) }
    }

    fn SetPixelShaderConstantB(&self, startregister: u32, pconstantdata: *const BOOL, boolcount: u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).SetPixelShaderConstantB(startregister, pconstantdata, boolcount) }
    }

    fn GetPixelShaderConstantB(&self, startregister: u32, pconstantdata: *mut BOOL, boolcount: u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetPixelShaderConstantB(startregister, pconstantdata, boolcount) }
    }

    fn DrawRectPatch(&self, handle: u32, pnumsegs: *const f32, prectpatchinfo: *const D3DRECTPATCH_INFO) -> Result<()> {
        unsafe { proxy_as_interface!(self).DrawRectPatch(handle, pnumsegs, prectpatchinfo) }
    }

    fn DrawTriPatch(&self, handle: u32, pnumsegs: *const f32, ptripatchinfo: *const D3DTRIPATCH_INFO) -> Result<()> {
        unsafe { proxy_as_interface!(self).DrawTriPatch(handle, pnumsegs, ptripatchinfo) }
    }

    fn DeletePatch(&self, handle: u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).DeletePatch(handle) }
    }

    fn CreateQuery(&self, r#type: D3DQUERYTYPE) -> Result<IDirect3DQuery9> {
        unsafe { self.proxy.CreateQuery_Impl(get_base_interface_fn!(self), r#type) }
    }
}
