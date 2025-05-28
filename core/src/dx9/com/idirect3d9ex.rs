//! [`IDirect3D9Ex`] proxy implementation.
//!
//! This module provides a proxy wrapper for the IDirect3D9Ex interface,
//! which extends IDirect3D9 with additional functionality for Windows Vista
//! and later, including improved device creation and display mode handling.

use super::*;
use std::ffi::c_void;
use windows::{
    Win32::{
        Foundation::*,
        Graphics::{Direct3D9::*, Gdi::*},
    },
    core::*,
};

/// Proxy wrapper for [`IDirect3D9Ex`] interface.
///
/// Extends [`IDirect3D9`] functionality with Windows Vista+ features while maintaining
/// compatibility. Intercepts Extended Direct3D 9 calls including CreateDeviceEx
/// and display mode operations, forwarding them to the underlying target interface.
///
/// Methods of [`IDirect3D9`] are delegated to the inner [`IDirect3D9`] proxy, which is implemented by [`ProxyDirect3D9`].
#[implement(IDirect3D9Ex)]
#[derive(Debug)]
pub struct ProxyDirect3D9Ex {
    proxy: ComObject<ProxyDirect3D9>,
    target: IDirect3D9Ex,
}

impl ProxyDirect3D9Ex {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret))]
    pub fn new(target: IDirect3D9Ex) -> Self {
        Self {
            proxy: ProxyDirect3D9::new(target.clone().into()).into(),
            target,
        }
    }
}

impl Drop for ProxyDirect3D9Ex {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret))]
    fn drop(&mut self) {}
}

impl_debug!(ProxyDirect3D9Ex_Impl);

#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]
impl IDirect3D9Ex_Impl for ProxyDirect3D9Ex_Impl {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, skip(ppreturneddeviceinterface)))]
    fn CreateDeviceEx(
        &self,
        adapter: u32,
        devicetype: D3DDEVTYPE,
        hfocuswindow: HWND,
        behaviorflags: u32,
        ppresentationparameters: *mut D3DPRESENT_PARAMETERS,
        pfullscreendisplaymode: *mut D3DDISPLAYMODEEX,
        ppreturneddeviceinterface: OutRef<IDirect3DDevice9Ex>,
    ) -> Result<()> {
        check_nullptr!(ppreturneddeviceinterface);

        let device = try_out_param(|out| unsafe {
            self.target
                .CreateDeviceEx(adapter, devicetype, hfocuswindow, behaviorflags, ppresentationparameters, pfullscreendisplaymode, out)
        })?;

        let config = DX9ProxyConfig;

        #[cfg(feature = "tracing")]
        tracing::debug!("Creating ProxyDirect3DDevice9Ex for {device:?} with config: {config:?}");

        let proxy = ProxyDirect3DDevice9Ex::new(device, config, self.to_interface());
        ppreturneddeviceinterface.write(Some(proxy.into()))
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "debug"))]
    fn EnumAdapterModesEx(&self, adapter: u32, pfilter: *const D3DDISPLAYMODEFILTER, mode: u32, pmode: *mut D3DDISPLAYMODEEX) -> Result<()> {
        unsafe { self.target.EnumAdapterModesEx(adapter, pfilter, mode, pmode) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "debug"))]
    fn GetAdapterDisplayModeEx(&self, adapter: u32, pmode: *mut D3DDISPLAYMODEEX, protation: *mut D3DDISPLAYROTATION) -> Result<()> {
        unsafe { self.target.GetAdapterDisplayModeEx(adapter, pmode, protation) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "debug"))]
    fn GetAdapterLUID(&self, adapter: u32, pluid: *mut LUID) -> Result<()> {
        unsafe { self.target.GetAdapterLUID(adapter, pluid) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "debug"))]
    fn GetAdapterModeCountEx(&self, adapter: u32, pfilter: *const D3DDISPLAYMODEFILTER) -> u32 {
        unsafe { self.target.GetAdapterModeCountEx(adapter, pfilter) }
    }
}

macro_rules! proxy_as_interface {
    ($this:ident) => {
        $this.proxy.as_interface::<IDirect3D9>()
    };
}

macro_rules! get_base_interface_fn {
    ($this:ident) => {
        || $this.to_interface::<IDirect3D9Ex>().into()
    };
}

/// Implementation of [`IDirect3D9`] for [`ProxyDirect3D9Ex`].
///
/// Most methods delegate to the inner [`IDirect3D9`] proxy. However, for methods that need to pass
/// a COM interface pointer of `self`, use the corresponding `*_Impl` methods from [`ProxyDirect3D9_Impl`]
/// when available. Check the base implementation to determine if a `*_Impl` variant exists before
/// delegating directly to avoid interface inconsistencies in inheritance scenarios.
///
/// We should not customize methods here, since [`ProxyDirect3D9Ex::proxy`] points to an [`IDirect3D9`] object of [`ProxyDirect3D9`].
#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]
impl IDirect3D9_Impl for ProxyDirect3D9Ex_Impl {
    fn RegisterSoftwareDevice(&self, pinitializefunction: *mut c_void) -> Result<()> {
        unsafe { proxy_as_interface!(self).RegisterSoftwareDevice(pinitializefunction) }
    }

    fn GetAdapterCount(&self) -> u32 {
        unsafe { proxy_as_interface!(self).GetAdapterCount() }
    }

    fn GetAdapterIdentifier(&self, adapter: u32, flags: u32, pidentifier: *mut D3DADAPTER_IDENTIFIER9) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetAdapterIdentifier(adapter, flags, pidentifier) }
    }

    fn GetAdapterModeCount(&self, adapter: u32, format: D3DFORMAT) -> u32 {
        unsafe { proxy_as_interface!(self).GetAdapterModeCount(adapter, format) }
    }

    fn EnumAdapterModes(&self, adapter: u32, format: D3DFORMAT, mode: u32, pmode: *mut D3DDISPLAYMODE) -> Result<()> {
        unsafe { proxy_as_interface!(self).EnumAdapterModes(adapter, format, mode, pmode) }
    }

    fn GetAdapterDisplayMode(&self, adapter: u32, pmode: *mut D3DDISPLAYMODE) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetAdapterDisplayMode(adapter, pmode) }
    }

    fn CheckDeviceType(&self, adapter: u32, devtype: D3DDEVTYPE, adapterformat: D3DFORMAT, backbufferformat: D3DFORMAT, bwindowed: BOOL) -> Result<()> {
        unsafe { proxy_as_interface!(self).CheckDeviceType(adapter, devtype, adapterformat, backbufferformat, bwindowed.into()) }
    }

    fn CheckDeviceFormat(&self, adapter: u32, devicetype: D3DDEVTYPE, adapterformat: D3DFORMAT, usage: u32, rtype: D3DRESOURCETYPE, checkformat: D3DFORMAT) -> Result<()> {
        unsafe { proxy_as_interface!(self).CheckDeviceFormat(adapter, devicetype, adapterformat, usage, rtype, checkformat) }
    }

    fn CheckDeviceMultiSampleType(&self, adapter: u32, devicetype: D3DDEVTYPE, surfaceformat: D3DFORMAT, windowed: BOOL, multisampletype: D3DMULTISAMPLE_TYPE, pqualitylevels: *mut u32) -> Result<()> {
        unsafe { proxy_as_interface!(self).CheckDeviceMultiSampleType(adapter, devicetype, surfaceformat, windowed.into(), multisampletype, pqualitylevels) }
    }

    fn CheckDepthStencilMatch(&self, adapter: u32, devicetype: D3DDEVTYPE, adapterformat: D3DFORMAT, rendertargetformat: D3DFORMAT, depthstencilformat: D3DFORMAT) -> Result<()> {
        unsafe { proxy_as_interface!(self).CheckDepthStencilMatch(adapter, devicetype, adapterformat, rendertargetformat, depthstencilformat) }
    }

    fn CheckDeviceFormatConversion(&self, adapter: u32, devicetype: D3DDEVTYPE, sourceformat: D3DFORMAT, targetformat: D3DFORMAT) -> Result<()> {
        unsafe { proxy_as_interface!(self).CheckDeviceFormatConversion(adapter, devicetype, sourceformat, targetformat) }
    }

    fn GetDeviceCaps(&self, adapter: u32, devicetype: D3DDEVTYPE, pcaps: *mut D3DCAPS9) -> Result<()> {
        unsafe { proxy_as_interface!(self).GetDeviceCaps(adapter, devicetype, pcaps) }
    }

    fn GetAdapterMonitor(&self, adapter: u32) -> HMONITOR {
        unsafe { proxy_as_interface!(self).GetAdapterMonitor(adapter) }
    }

    fn CreateDevice(
        &self,
        adapter: u32,
        devicetype: D3DDEVTYPE,
        hfocuswindow: HWND,
        behaviorflags: u32,
        ppresentationparameters: *mut D3DPRESENT_PARAMETERS,
        ppreturneddeviceinterface: OutRef<IDirect3DDevice9>,
    ) -> Result<()> {
        // Here, we call `CreateDevice_Impl` instead of `CreateDevice` to avoid exposing internal proxy of `ProxyDirect3D9`.
        unsafe {
            self.proxy.CreateDevice_Impl(
                get_base_interface_fn!(self),
                adapter,
                devicetype,
                hfocuswindow,
                behaviorflags,
                ppresentationparameters,
                ppreturneddeviceinterface,
            )
        }
    }
}
