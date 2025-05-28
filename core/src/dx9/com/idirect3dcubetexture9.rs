//! [`IDirect3DCubeTexture9`] proxy implementation.

use super::*;
use std::ffi::c_void;
use tracing::instrument;
use windows::{Win32::Foundation::*, Win32::Graphics::Direct3D9::*, core::*};

#[implement(IDirect3DCubeTexture9)]
#[derive(Debug)]
pub struct ProxyDirect3DCubeTexture9 {
    target: IDirect3DCubeTexture9,
    context: DX9ProxyDeviceContext,
    proxy_device: IDirect3DDevice9,
}

impl ProxyDirect3DCubeTexture9 {
    #[instrument(ret, level = "debug")]
    pub fn new(target: IDirect3DCubeTexture9, context: DX9ProxyDeviceContext, proxy_device: IDirect3DDevice9) -> Self {
        Self { target, context, proxy_device }
    }
}

impl Drop for ProxyDirect3DCubeTexture9 {
    #[instrument(ret, level = "debug")]
    fn drop(&mut self) {
        self.context.on_proxy_destroy(&self.target);
    }
}

impl_debug!(ProxyDirect3DCubeTexture9_Impl);

#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]
impl IDirect3DCubeTexture9_Impl for ProxyDirect3DCubeTexture9_Impl {
    #[instrument(err, ret, level = "trace")]
    fn GetLevelDesc(&self, level: u32, pdesc: *mut D3DSURFACE_DESC) -> Result<()> {
        unsafe { self.target.GetLevelDesc(level, pdesc) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetCubeMapSurface(&self, facetype: D3DCUBEMAP_FACES, level: u32) -> Result<IDirect3DSurface9> {
        let target = unsafe { self.target.GetCubeMapSurface(facetype, level) }?;
        Ok(self.context.ensure_proxy(target, |target| {
            ProxyDirect3DSurface9::new(target, self.context.clone(), self.proxy_device.clone(), DX9SurfaceContainer::CubeTexture(self.to_interface())).into()
        }))
    }

    #[instrument(err, ret, level = "trace")]
    fn LockRect(&self, facetype: D3DCUBEMAP_FACES, level: u32, plockedrect: *mut D3DLOCKED_RECT, prect: *const RECT, flags: u32) -> Result<()> {
        unsafe { self.target.LockRect(facetype, level, plockedrect, prect, flags) }
    }

    #[instrument(err, ret, level = "trace")]
    fn UnlockRect(&self, facetype: D3DCUBEMAP_FACES, level: u32) -> Result<()> {
        unsafe { self.target.UnlockRect(facetype, level) }
    }

    #[instrument(err, ret, level = "trace")]
    fn AddDirtyRect(&self, facetype: D3DCUBEMAP_FACES, pdirtyrect: *const RECT) -> Result<()> {
        unsafe { self.target.AddDirtyRect(facetype, pdirtyrect) }
    }
}

#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]
impl IDirect3DBaseTexture9_Impl for ProxyDirect3DCubeTexture9_Impl {
    #[instrument(ret, level = "trace")]
    fn SetLOD(&self, lodnew: u32) -> u32 {
        unsafe { self.target.SetLOD(lodnew) }
    }

    #[instrument(ret, level = "trace")]
    fn GetLOD(&self) -> u32 {
        unsafe { self.target.GetLOD() }
    }

    #[instrument(ret, level = "trace")]
    fn GetLevelCount(&self) -> u32 {
        unsafe { self.target.GetLevelCount() }
    }

    #[instrument(err, ret, level = "trace")]
    fn SetAutoGenFilterType(&self, filtertype: D3DTEXTUREFILTERTYPE) -> Result<()> {
        unsafe { self.target.SetAutoGenFilterType(filtertype) }
    }

    #[instrument(ret, level = "trace")]
    fn GetAutoGenFilterType(&self) -> D3DTEXTUREFILTERTYPE {
        unsafe { self.target.GetAutoGenFilterType() }
    }

    #[instrument(ret, level = "trace")]
    fn GenerateMipSubLevels(&self) {
        unsafe { self.target.GenerateMipSubLevels() }
    }
}

#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]
impl IDirect3DResource9_Impl for ProxyDirect3DCubeTexture9_Impl {
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
