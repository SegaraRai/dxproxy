//! [`IDirect3DVolumeTexture9`] proxy implementation.

use super::*;
use std::ffi::c_void;
use windows::{Win32::Graphics::Direct3D9::*, core::*};

#[implement(IDirect3DVolumeTexture9)]
#[derive(Debug)]
pub struct ProxyDirect3DVolumeTexture9 {
    target: IDirect3DVolumeTexture9,
    context: DX9ProxyDeviceContext,
    proxy_device: IDirect3DDevice9,
}

impl ProxyDirect3DVolumeTexture9 {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "debug"))]
    pub fn new(target: IDirect3DVolumeTexture9, context: DX9ProxyDeviceContext, proxy_device: IDirect3DDevice9) -> Self {
        Self { target, context, proxy_device }
    }
}

impl Drop for ProxyDirect3DVolumeTexture9 {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "debug"))]
    fn drop(&mut self) {
        self.context.on_proxy_destroy(&self.target);
    }
}

impl_debug!(ProxyDirect3DVolumeTexture9_Impl);

#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]
impl IDirect3DVolumeTexture9_Impl for ProxyDirect3DVolumeTexture9_Impl {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn GetLevelDesc(&self, level: u32, pdesc: *mut D3DVOLUME_DESC) -> Result<()> {
        unsafe { self.target.GetLevelDesc(level, pdesc) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn GetVolumeLevel(&self, level: u32) -> Result<IDirect3DVolume9> {
        let target = unsafe { self.target.GetVolumeLevel(level) }?;
        Ok(self.context.ensure_proxy(target, |target| {
            ProxyDirect3DVolume9::new(target, self.context.clone(), self.proxy_device.clone(), self.to_interface()).into()
        }))
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn LockBox(&self, level: u32, plockedvolume: *mut D3DLOCKED_BOX, pbox: *const D3DBOX, flags: u32) -> Result<()> {
        unsafe { self.target.LockBox(level, plockedvolume, pbox, flags) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn UnlockBox(&self, level: u32) -> Result<()> {
        unsafe { self.target.UnlockBox(level) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn AddDirtyBox(&self, pdirtybox: *const D3DBOX) -> Result<()> {
        unsafe { self.target.AddDirtyBox(pdirtybox) }
    }
}

#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]
impl IDirect3DBaseTexture9_Impl for ProxyDirect3DVolumeTexture9_Impl {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "trace"))]
    fn SetLOD(&self, lodnew: u32) -> u32 {
        unsafe { self.target.SetLOD(lodnew) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "trace"))]
    fn GetLOD(&self) -> u32 {
        unsafe { self.target.GetLOD() }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "trace"))]
    fn GetLevelCount(&self) -> u32 {
        unsafe { self.target.GetLevelCount() }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn SetAutoGenFilterType(&self, filtertype: D3DTEXTUREFILTERTYPE) -> Result<()> {
        unsafe { self.target.SetAutoGenFilterType(filtertype) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "trace"))]
    fn GetAutoGenFilterType(&self) -> D3DTEXTUREFILTERTYPE {
        unsafe { self.target.GetAutoGenFilterType() }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "trace"))]
    fn GenerateMipSubLevels(&self) {
        unsafe { self.target.GenerateMipSubLevels() }
    }
}

#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]
impl IDirect3DResource9_Impl for ProxyDirect3DVolumeTexture9_Impl {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn GetDevice(&self) -> Result<IDirect3DDevice9> {
        Ok(self.proxy_device.clone())
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn SetPrivateData(&self, refguid: *const GUID, pdata: *const c_void, sizeofdata: u32, flags: u32) -> Result<()> {
        unsafe { self.target.SetPrivateData(refguid, pdata, sizeofdata, flags) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn GetPrivateData(&self, refguid: *const GUID, pdata: *mut c_void, psizeofdata: *mut u32) -> Result<()> {
        unsafe { self.target.GetPrivateData(refguid, pdata, psizeofdata) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace"))]
    fn FreePrivateData(&self, refguid: *const GUID) -> Result<()> {
        unsafe { self.target.FreePrivateData(refguid) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "trace"))]
    fn SetPriority(&self, prioritynew: u32) -> u32 {
        unsafe { self.target.SetPriority(prioritynew) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "trace"))]
    fn GetPriority(&self) -> u32 {
        unsafe { self.target.GetPriority() }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "trace"))]
    fn PreLoad(&self) {
        unsafe { self.target.PreLoad() }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "trace"))]
    fn GetType(&self) -> D3DRESOURCETYPE {
        unsafe { self.target.GetType() }
    }
}
