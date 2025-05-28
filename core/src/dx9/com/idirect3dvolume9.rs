//! [`IDirect3DVolume9`] proxy implementation.

use super::*;
use std::ffi::c_void;
use tracing::instrument;
use windows::{Win32::Graphics::Direct3D9::*, core::*};

#[implement(IDirect3DVolume9)]
#[derive(Debug)]
pub struct ProxyDirect3DVolume9 {
    target: IDirect3DVolume9,
    context: DX9ProxyDeviceContext,
    proxy_device: IDirect3DDevice9,
    proxy_container: IDirect3DVolumeTexture9,
}

impl ProxyDirect3DVolume9 {
    #[instrument(ret, level = "debug")]
    pub fn new(target: IDirect3DVolume9, context: DX9ProxyDeviceContext, proxy_device: IDirect3DDevice9, proxy_container: IDirect3DVolumeTexture9) -> Self {
        Self {
            target,
            context,
            proxy_device,
            proxy_container,
        }
    }
}

impl Drop for ProxyDirect3DVolume9 {
    #[instrument(ret, level = "debug")]
    fn drop(&mut self) {
        self.context.on_proxy_destroy(&self.target);
    }
}

impl_debug!(ProxyDirect3DVolume9_Impl);

#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]
impl IDirect3DVolume9_Impl for ProxyDirect3DVolume9_Impl {
    #[instrument(err, ret, level = "trace")]
    fn GetDevice(&self) -> Result<IDirect3DDevice9> {
        Ok(self.proxy_device.clone())
    }

    #[instrument(err, ret, level = "trace")]
    fn GetContainer(&self, riid: *const GUID, ppcontainer: *mut *mut c_void) -> Result<()> {
        check_nullptr!(riid);
        check_nullptr!(ppcontainer);

        if unsafe { *riid } != IDirect3DVolumeTexture9::IID {
            return Err(D3DERR_INVALIDCALL.into());
        }

        unsafe { ppcontainer.write(self.proxy_container.clone().into_raw()) };
        Ok(())
    }

    #[instrument(err, ret, level = "trace")]
    fn GetDesc(&self, pdesc: *mut D3DVOLUME_DESC) -> Result<()> {
        unsafe { self.target.GetDesc(pdesc) }
    }

    #[instrument(err, ret, level = "trace")]
    fn LockBox(&self, plockedvolume: *mut D3DLOCKED_BOX, pbox: *const D3DBOX, flags: u32) -> Result<()> {
        unsafe { self.target.LockBox(plockedvolume, pbox, flags) }
    }

    #[instrument(err, ret, level = "trace")]
    fn UnlockBox(&self) -> Result<()> {
        unsafe { self.target.UnlockBox() }
    }

    #[instrument(err, ret, level = "trace")]
    fn FreePrivateData(&self, refguid: *const GUID) -> Result<()> {
        unsafe { self.target.FreePrivateData(refguid) }
    }

    #[instrument(err, ret, level = "trace")]
    fn GetPrivateData(&self, refguid: *const GUID, pdata: *mut c_void, psizeofdata: *mut u32) -> Result<()> {
        unsafe { self.target.GetPrivateData(refguid, pdata, psizeofdata) }
    }

    #[instrument(err, ret, level = "trace")]
    fn SetPrivateData(&self, refguid: *const GUID, pdata: *const c_void, sizeofdata: u32, flags: u32) -> Result<()> {
        unsafe { self.target.SetPrivateData(refguid, pdata, sizeofdata, flags) }
    }
}
