//! [`IDirect3DSurface9`] proxy implementation.

use super::*;
use std::ffi::c_void;
use tracing::instrument;
use windows::{
    Win32::Foundation::*,
    Win32::Graphics::{Direct3D9::*, Gdi::*},
    core::*,
};

#[derive(Debug, Clone)]
pub enum DX9SurfaceContainer {
    Texture(IDirect3DTexture9),
    VolumeTexture(IDirect3DVolumeTexture9),
    CubeTexture(IDirect3DCubeTexture9),
    SwapChain(IDirect3DSwapChain9),
    /// For CreateRenderTarget, CreateOffscreenPlainSurface, and CreateDepthStencilSurface
    Standalone,
}

#[implement(IDirect3DSurface9)]
#[derive(Debug)]
pub struct ProxyDirect3DSurface9 {
    target: IDirect3DSurface9,
    context: DX9ProxyDeviceContext,
    proxy_device: IDirect3DDevice9,
    proxy_container: DX9SurfaceContainer,
}

impl ProxyDirect3DSurface9 {
    #[instrument(ret, level = "debug")]
    pub fn new(target: IDirect3DSurface9, context: DX9ProxyDeviceContext, proxy_device: IDirect3DDevice9, proxy_container: DX9SurfaceContainer) -> Self {
        Self {
            target,
            context,
            proxy_device,
            proxy_container,
        }
    }
}

impl Drop for ProxyDirect3DSurface9 {
    #[instrument(ret, level = "debug")]
    fn drop(&mut self) {
        self.context.on_proxy_destroy(&self.target);
    }
}

impl_debug!(ProxyDirect3DSurface9_Impl);

#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]
impl IDirect3DSurface9_Impl for ProxyDirect3DSurface9_Impl {
    #[instrument(err, ret, level = "trace")]
    fn GetContainer(&self, riid: *const GUID, ppcontainer: *mut *mut c_void) -> Result<()> {
        check_nullptr!(riid);
        check_nullptr!(ppcontainer);

        match &self.proxy_container {
            DX9SurfaceContainer::Texture(proxy) => {
                if unsafe { *riid } == IDirect3DTexture9::IID {
                    unsafe { ppcontainer.write(proxy.clone().into_raw()) };
                    return Ok(());
                }
            }
            DX9SurfaceContainer::VolumeTexture(proxy) => {
                if unsafe { *riid } == IDirect3DVolumeTexture9::IID {
                    unsafe { ppcontainer.write(proxy.clone().into_raw()) };
                    return Ok(());
                }
            }
            DX9SurfaceContainer::CubeTexture(proxy) => {
                if unsafe { *riid } == IDirect3DCubeTexture9::IID {
                    unsafe { ppcontainer.write(proxy.clone().into_raw()) };
                    return Ok(());
                }
            }
            DX9SurfaceContainer::SwapChain(proxy) => {
                if unsafe { *riid } == IDirect3DSwapChain9::IID {
                    unsafe { ppcontainer.write(proxy.clone().into_raw()) };
                    return Ok(());
                }
            }
            DX9SurfaceContainer::Standalone => {
                // TODO: Should we allow IDirect3DDevice9 anywhere?
                if unsafe { *riid } == IDirect3DDevice9::IID {
                    unsafe { ppcontainer.write(self.proxy_device.clone().into_raw()) };
                    return Ok(());
                }
            }
        }

        Err(D3DERR_INVALIDCALL.into())
    }

    #[instrument(err, ret, level = "trace")]
    fn GetDesc(&self, pdesc: *mut D3DSURFACE_DESC) -> Result<()> {
        unsafe { self.target.GetDesc(pdesc) }
    }

    #[instrument(err, ret, level = "trace")]
    fn LockRect(&self, plockedrect: *mut D3DLOCKED_RECT, prect: *const RECT, flags: u32) -> Result<()> {
        unsafe { self.target.LockRect(plockedrect, prect, flags) }
    }

    #[instrument(err, ret, level = "trace")]
    fn UnlockRect(&self) -> Result<()> {
        unsafe { self.target.UnlockRect() }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetDC(&self, phdc: *mut HDC) -> Result<()> {
        unsafe { self.target.GetDC(phdc) }
    }

    #[instrument(err, ret, level = "trace")]
    fn ReleaseDC(&self, hdc: HDC) -> Result<()> {
        unsafe { self.target.ReleaseDC(hdc) }
    }
}

#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]
impl IDirect3DResource9_Impl for ProxyDirect3DSurface9_Impl {
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
