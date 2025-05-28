//! DirectX 9 proxy device context for managing COM object lifecycles and mappings.
//!
//! This module provides the core context used by DirectX 9 proxy objects to manage
//! the relationship between original DirectX objects and their proxy wrappers.
//! It handles configuration, COM object mapping, and thread-safe access to shared state.

use super::*;
use crate::{ComMappingTracker, NullableInterfaceIn, NullableInterfaceOut};
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};
use windows::core::*;

/// Internal implementation of the DirectX 9 proxy device context.
///
/// Contains the configuration and COM mapping tracker, protected by
/// appropriate synchronization primitives for thread-safe access.
#[derive(Debug)]
pub struct DX9ProxyDeviceContextImpl {
    config: DX9ProxyConfig,
    tracker: Mutex<ComMappingTracker>,
}

unsafe impl Send for DX9ProxyDeviceContextImpl {}
unsafe impl Sync for DX9ProxyDeviceContextImpl {}

/// Thread-safe DirectX 9 proxy device context.
///
/// This context is shared among DirectX 9 proxy objects and provides:
/// - Access to proxy configuration settings
/// - COM object mapping and lifecycle management
/// - Thread-safe operations for concurrent DirectX usage
///
/// The context is reference-counted and can be safely cloned and shared
/// across multiple proxy objects.
#[derive(Debug, Clone)]
pub struct DX9ProxyDeviceContext(Arc<DX9ProxyDeviceContextImpl>);

impl DX9ProxyDeviceContext {
    /// Creates a new DirectX 9 proxy device context with the specified configuration.
    pub fn new(config: DX9ProxyConfig) -> Self {
        Self(Arc::new(DX9ProxyDeviceContextImpl {
            config,
            tracker: Mutex::new(ComMappingTracker::default()),
        }))
    }

    /// Returns a reference to the underlying configuration.
    pub fn get_config(&self) -> &DX9ProxyConfig {
        &self.0.config
    }

    /// See [`ComMappingTracker::ensure_proxy`].
    pub fn ensure_proxy<T: Interface + Debug>(&self, target: T, create_proxy_fn: impl FnOnce(T) -> T) -> T {
        let mut storage = self.0.tracker.lock().unwrap();
        storage.ensure_proxy(target, create_proxy_fn)
    }

    /// See [`ComMappingTracker::try_ensure_proxy`].
    pub fn try_ensure_proxy<T: Interface + Debug>(&self, target: T, try_create_proxy_fn: impl FnOnce(T) -> Result<T>) -> Result<T> {
        let mut storage = self.0.tracker.lock().unwrap();
        storage.try_ensure_proxy(target, try_create_proxy_fn)
    }

    /// See [`ComMappingTracker::get_proxy`].
    pub fn get_proxy<T: Interface + Debug>(&self, target: T) -> Option<T> {
        let mut storage = self.0.tracker.lock().unwrap();
        storage.get_proxy(target)
    }

    /// See [`ComMappingTracker::get_target`].
    pub fn get_target<T: Interface + Debug, K: NullableInterfaceIn<T>>(&self, proxy: K) -> Option<NullableInterfaceOut<T>> {
        let mut storage = self.0.tracker.lock().unwrap();
        storage.get_target(proxy)
    }

    /// See [`ComMappingTracker::get_target_nullable`].
    pub fn get_target_nullable<T: Interface + Debug, K: NullableInterfaceIn<T>>(&self, proxy: K) -> Option<NullableInterfaceOut<T>> {
        let mut storage = self.0.tracker.lock().unwrap();
        storage.get_target_nullable(proxy)
    }

    /// See [`ComMappingTracker::on_proxy_destroy`].
    pub fn on_proxy_destroy<T: Interface + Debug>(&self, target: &T) {
        let mut storage = self.0.tracker.lock().unwrap();
        storage.on_proxy_destroy(target);
    }
}
