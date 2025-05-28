//! [`IDirect3D9`] proxy implementation.
//!
//! This module provides a proxy wrapper for the IDirect3D9 interface,
//! which is the main entry point for Direct3D 9 functionality including
//! adapter enumeration, device creation, and capability queries.

use super::*;
use std::ffi::c_void;
use windows::{
    Win32::{
        Foundation::*,
        Graphics::{Direct3D9::*, Gdi::*},
    },
    core::*,
};

/// Proxy wrapper for [`IDirect3D9`]s interface.
///
/// Intercepts and instruments all [`IDirect3D9`] method calls while forwarding
/// them to the underlying target interface. Provides logging and potential
/// modification of Direct3D 9 initialization and enumeration operations.
#[implement(IDirect3D9)]
#[derive(Debug)]
pub struct ProxyDirect3D9 {
    target: IDirect3D9,
}

impl ProxyDirect3D9 {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret))]
    pub fn new(target: IDirect3D9) -> Self {
        Self { target }
    }

    /// Creates a new proxy container or upgrades to an Ex version if available.
    ///
    /// This function checks if the provided `target` can be cast to [`IDirect3D9Ex`].
    /// If it can, it creates a [`ProxyDirect3D9Ex`] instance; otherwise, it falls back to
    /// creating a regular [`ProxyDirect3D9`].
    ///
    /// It is recommended to use this method rather than [`new`] directly, as it handles both
    /// cases seamlessly, ensuring that the correct interface is returned based on the target's type.
    ///
    /// # Arguments
    /// * `target` - The target container to wrap.
    ///
    /// # Returns
    /// An [`IDirect3D9`] instance, which may be a proxy for either
    /// [`IDirect3D9Ex`] or [`IDirect3D9`], depending on the target's type.
    ///
    /// [`new`]: Self::new
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret))]
    pub fn new_or_upgrade(target: IDirect3D9) -> IDirect3D9 {
        if let Ok(ex_target) = target.cast::<IDirect3D9Ex>() {
            let ex_interface: IDirect3D9Ex = ProxyDirect3D9Ex::new(ex_target).into();
            ex_interface.into()
        } else {
            // If the target is not an Ex version, we downgrade to the regular container.
            Self::new(target).into()
        }
    }
}

impl Drop for ProxyDirect3D9 {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret))]
    fn drop(&mut self) {}
}

impl_debug!(ProxyDirect3D9_Impl);

/// Implementation block providing `*_Impl` methods that accept a COM interface getter function.
///
/// Since [`IDirect3D9`] may be inherited by [`IDirect3D9Ex`], directly exposing the Direct3D
/// instance could cause inconsistencies such as upgrade issues or pointer changes.
/// These methods take an additional `get_self_interface` parameter that accepts a COM pointer
/// to expose only the necessary interface instances, ensuring proper type consistency.
#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref, clippy::too_many_arguments)]
impl ProxyDirect3D9_Impl {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "trace", skip(get_self_interface, ppreturneddeviceinterface)))]
    pub(super) unsafe fn CreateDevice_Impl<F: FnOnce() -> IDirect3D9>(
        &self,
        get_self_interface: F,
        adapter: u32,
        devicetype: D3DDEVTYPE,
        hfocuswindow: HWND,
        behaviorflags: u32,
        ppresentationparameters: *mut D3DPRESENT_PARAMETERS,
        ppreturneddeviceinterface: OutRef<IDirect3DDevice9>,
    ) -> Result<()> {
        check_nullptr!(ppreturneddeviceinterface);

        let device = try_out_param(|out| unsafe { self.target.CreateDevice(adapter, devicetype, hfocuswindow, behaviorflags, ppresentationparameters, out) })?;

        let config = DX9ProxyConfig;

        #[cfg(feature = "tracing")]
        tracing::debug!("Creating ProxyDirect3DDevice9 for {device:?} with config: {config:?}");

        let proxy = ProxyDirect3DDevice9::new_or_upgrade(device, config, get_self_interface());
        ppreturneddeviceinterface.write(Some(proxy))
    }
}

/// Implementation of [`IDirect3D9`] for [`ProxyDirect3D9`].
///
/// Methods that need to pass a COM interface pointer of `self` should use the corresponding
/// `*_Impl` methods from the implementation block above. This ensures proper type consistency
/// when dealing with interface inheritance (e.g., [`IDirect3D9Ex`] extending [`IDirect3D9`]).
#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]
impl IDirect3D9_Impl for ProxyDirect3D9_Impl {
    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret))]
    fn RegisterSoftwareDevice(&self, pinitializefunction: *mut c_void) -> Result<()> {
        unsafe { self.target.RegisterSoftwareDevice(pinitializefunction) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "debug"))]
    fn GetAdapterCount(&self) -> u32 {
        unsafe { self.target.GetAdapterCount() }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "debug"))]
    fn GetAdapterIdentifier(&self, adapter: u32, flags: u32, pidentifier: *mut D3DADAPTER_IDENTIFIER9) -> Result<()> {
        unsafe { self.target.GetAdapterIdentifier(adapter, flags, pidentifier) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "debug"))]
    fn GetAdapterModeCount(&self, adapter: u32, format: D3DFORMAT) -> u32 {
        unsafe { self.target.GetAdapterModeCount(adapter, format) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "debug"))]
    fn EnumAdapterModes(&self, adapter: u32, format: D3DFORMAT, mode: u32, pmode: *mut D3DDISPLAYMODE) -> Result<()> {
        unsafe { self.target.EnumAdapterModes(adapter, format, mode, pmode) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "debug"))]
    fn GetAdapterDisplayMode(&self, adapter: u32, pmode: *mut D3DDISPLAYMODE) -> Result<()> {
        unsafe { self.target.GetAdapterDisplayMode(adapter, pmode) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "debug"))]
    fn CheckDeviceType(&self, adapter: u32, devtype: D3DDEVTYPE, adapterformat: D3DFORMAT, backbufferformat: D3DFORMAT, bwindowed: BOOL) -> Result<()> {
        unsafe { self.target.CheckDeviceType(adapter, devtype, adapterformat, backbufferformat, bwindowed.into()) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "debug"))]
    fn CheckDeviceFormat(&self, adapter: u32, devicetype: D3DDEVTYPE, adapterformat: D3DFORMAT, usage: u32, rtype: D3DRESOURCETYPE, checkformat: D3DFORMAT) -> Result<()> {
        unsafe { self.target.CheckDeviceFormat(adapter, devicetype, adapterformat, usage, rtype, checkformat) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "debug"))]
    fn CheckDeviceMultiSampleType(&self, adapter: u32, devicetype: D3DDEVTYPE, surfaceformat: D3DFORMAT, windowed: BOOL, multisampletype: D3DMULTISAMPLE_TYPE, pqualitylevels: *mut u32) -> Result<()> {
        unsafe {
            self.target
                .CheckDeviceMultiSampleType(adapter, devicetype, surfaceformat, windowed.into(), multisampletype, pqualitylevels)
        }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "debug"))]
    fn CheckDepthStencilMatch(&self, adapter: u32, devicetype: D3DDEVTYPE, adapterformat: D3DFORMAT, rendertargetformat: D3DFORMAT, depthstencilformat: D3DFORMAT) -> Result<()> {
        unsafe { self.target.CheckDepthStencilMatch(adapter, devicetype, adapterformat, rendertargetformat, depthstencilformat) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "debug"))]
    fn CheckDeviceFormatConversion(&self, adapter: u32, devicetype: D3DDEVTYPE, sourceformat: D3DFORMAT, targetformat: D3DFORMAT) -> Result<()> {
        unsafe { self.target.CheckDeviceFormatConversion(adapter, devicetype, sourceformat, targetformat) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, level = "debug"))]
    fn GetDeviceCaps(&self, adapter: u32, devicetype: D3DDEVTYPE, pcaps: *mut D3DCAPS9) -> Result<()> {
        unsafe { self.target.GetDeviceCaps(adapter, devicetype, pcaps) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(ret, level = "debug"))]
    fn GetAdapterMonitor(&self, adapter: u32) -> HMONITOR {
        unsafe { self.target.GetAdapterMonitor(adapter) }
    }

    #[cfg_attr(feature = "tracing-instrument", tracing::instrument(err, ret, skip(ppreturneddeviceinterface)))]
    fn CreateDevice(
        &self,
        adapter: u32,
        devicetype: D3DDEVTYPE,
        hfocuswindow: HWND,
        behaviorflags: u32,
        ppresentationparameters: *mut D3DPRESENT_PARAMETERS,
        ppreturneddeviceinterface: OutRef<IDirect3DDevice9>,
    ) -> Result<()> {
        unsafe {
            self.CreateDevice_Impl(
                || self.to_interface(),
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
