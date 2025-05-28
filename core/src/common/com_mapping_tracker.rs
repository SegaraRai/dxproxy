//! COM interface mapping and proxy management utilities.
//!
//! This module provides types and utilities for managing bidirectional mappings
//! between original COM objects and their proxy wrappers, enabling efficient
//! lookup and lifecycle management in proxy scenarios.

use std::{
    any::type_name,
    collections::HashMap,
    ffi::c_void,
    fmt::Debug,
    marker::PhantomData,
    mem::{forget, transmute_copy},
    ptr::null_mut,
};
use windows::core::*;

/// Increments the reference count of a COM interface object.
///
/// # Safety
/// This function must only be called with valid COM interface objects.
unsafe fn add_ref<T: Interface>(obj: T) -> T {
    forget(obj.clone());
    obj
}

/// Trait for types that can provide an optional reference to a COM interface.
///
/// This trait enables working with both nullable and non-nullable COM interface
/// references in a unified way, particularly useful for proxy operations.
pub trait NullableInterfaceIn<T: Interface> {
    fn as_ref(&self) -> Option<&T>;
}

impl<T: Interface> NullableInterfaceIn<T> for Ref<'_, T> {
    fn as_ref(&self) -> Option<&T> {
        self.as_ref()
    }
}

impl<T: Interface> NullableInterfaceIn<T> for Option<&T> {
    fn as_ref(&self) -> Option<&T> {
        *self
    }
}

/// A nullable COM interface output parameter wrapper.
///
/// This type wraps a raw COM interface pointer and provides safe access
/// to nullable COM interface outputs, commonly used in COM method calls
/// that may return null pointers.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct NullableInterfaceOut<T: Interface>(*mut c_void, PhantomData<T>);

impl<T: Interface> NullableInterfaceOut<T> {
    fn new(ptr: *mut c_void) -> Self {
        Self(ptr, PhantomData)
    }

    fn as_raw(&self) -> *mut c_void {
        self.0
    }
}

impl<T: Interface> Param<T> for NullableInterfaceOut<T> {
    unsafe fn param(self) -> ParamValue<T> {
        ParamValue::Borrowed(self.0)
    }
}

/// Tracks bidirectional mappings between COM target objects and their proxy wrappers.
///
/// This tracker maintains two hash maps to enable efficient lookups in both directions:
/// - Target → Proxy: Find existing proxy for a given target object
/// - Proxy → Target: Find the original target object for a given proxy
///
/// Used to ensure consistent proxy relationships and prevent duplicate proxy creation.
///
/// # Weak Reference Semantics
///
/// **Important**: `ComMappingTracker` does NOT own the COM interfaces it tracks and does NOT
/// increase their reference counts. It holds raw pointers as weak references to prevent
/// circular reference cycles that would cause memory leaks. If the tracker held strong
/// references, it would create a cycle: target ↔ proxy ↔ tracker, preventing proper cleanup.
///
/// This design requires careful coordination with proxy lifecycle management to ensure
/// mappings are removed via [`on_proxy_destroy`] when proxies are dropped.
///
/// [`on_proxy_destroy`]: Self::on_proxy_destroy
#[derive(Default)]
pub struct ComMappingTracker {
    target_to_proxy: HashMap<*mut c_void, *mut c_void>,
    proxy_to_target: HashMap<*mut c_void, *mut c_void>,
}

unsafe impl Send for ComMappingTracker {}
unsafe impl Sync for ComMappingTracker {}

impl std::fmt::Debug for ComMappingTracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const VERBOSE: bool = true;

        if VERBOSE {
            f.debug_struct("ComMappingTracker")
                .field("target_to_proxy", &self.target_to_proxy)
                .field("proxy_to_target", &self.proxy_to_target)
                .finish()
        } else {
            f.debug_struct("ComMappingTracker")
                .field("target_to_proxy_count", &self.target_to_proxy.len())
                .field("proxy_to_target_count", &self.proxy_to_target.len())
                .finish()
        }
    }
}

impl ComMappingTracker {
    /// Ensures a proxy exists for the given target COM object, creating one if necessary.
    ///
    /// This method first checks if a proxy already exists for the target object. If found,
    /// it returns the existing proxy (with proper reference counting). If not found, it
    /// creates a new proxy using the provided creation function and stores the mapping.
    ///
    /// # Type Parameters
    /// * `T` - The COM interface type that implements `Interface + Debug`
    ///
    /// # Arguments
    /// * `target` - The target COM object to create or find a proxy for
    /// * `try_create_proxy_fn` - A function that attempts to create a new proxy from the target object
    ///
    /// # Returns
    /// * `Ok(T)` - The proxy object (either existing or newly created)
    /// * `Err(E)` - Error from the proxy creation function if creation fails
    ///
    /// # Reference Counting
    /// - If an existing proxy is found: target's ref count is decreased (via drop), proxy's ref count is increased
    /// - If a new proxy is created: target's reference is moved to the proxy, proxy ref count remains 1
    ///
    /// # Example
    /// ```ignore
    /// let proxy = tracker.try_ensure_proxy(d3d_device, |target| {
    ///     Ok(ProxyDevice::new(target))
    /// })?;
    /// ```
    pub fn try_ensure_proxy<T: Interface + Debug>(&mut self, target: T, try_create_proxy_fn: impl FnOnce(T) -> Result<T>) -> Result<T> {
        let target_ptr = target.as_raw();
        if let Some(proxy_ptr) = self.target_to_proxy.get(&target_ptr) {
            // If we already have a proxy for this org surface, return it
            // - Decrease ref count of target via drop
            // - Increase ref count of proxy
            tracing::debug!("Found existing {} proxy: {proxy_ptr:?} (<=> {target_ptr:?})", type_name::<T>());
            return Ok(unsafe { add_ref(T::from_raw(*proxy_ptr)) });
        }

        // Create a new proxy if it doesn't exist
        // - Move the target reference to a proxy
        // - Keep ref count of proxy 1
        let proxy = try_create_proxy_fn(target)?;
        let proxy_ptr = proxy.as_raw();

        // Store the new proxy in the storage
        self.target_to_proxy.insert(target_ptr, proxy_ptr);
        self.proxy_to_target.insert(proxy_ptr, target_ptr);

        tracing::debug!("Created new {} proxy: {proxy_ptr:p} (<=> {target_ptr:p})", type_name::<T>());
        tracing::trace!("Current maps: {self:?}");

        // Return the pointer to the new proxy
        Ok(proxy)
    }

    /// Ensures a proxy exists for the given target COM object, creating one if necessary.
    ///
    /// This is a convenience wrapper around [`try_ensure_proxy`] that always returns a proxy.
    ///
    /// # Type Parameters
    /// * `T` - The COM interface type that implements `Interface + Debug`
    ///
    /// # Arguments
    /// * `target` - The target COM object to create or find a proxy for
    /// * `create_proxy_fn` - A function that creates a new proxy from the target object
    ///
    /// # Returns
    /// The proxy object (either existing or newly created)
    ///
    /// # Reference Counting
    /// Same as [`try_ensure_proxy`]
    ///
    /// [`try_ensure_proxy`]: Self::try_ensure_proxy
    pub fn ensure_proxy<T: Interface + Debug>(&mut self, target: T, create_proxy_fn: impl FnOnce(T) -> T) -> T {
        self.try_ensure_proxy(target, |target| Ok(create_proxy_fn(target))).unwrap()
    }

    /// Retrieves an existing proxy for the given target COM object.
    ///
    /// Unlike [`try_ensure_proxy`] and [`ensure_proxy`], this method only looks up
    /// existing proxies and does not create new ones. Returns `None` if no proxy
    /// exists for the target object.
    ///
    /// # Type Parameters
    /// * `T` - The COM interface type that implements `Interface + Debug`
    ///
    /// # Arguments
    /// * `target` - The target COM object to find a proxy for
    ///
    /// # Returns
    /// * `Some(T)` - The existing proxy object if found
    /// * `None` - If no proxy exists for the target object
    ///
    /// # Reference Counting
    /// - Target's ref count is decreased (via drop)
    /// - Proxy's ref count is increased if found
    ///
    /// [`try_ensure_proxy`]: Self::try_ensure_proxy
    /// [`ensure_proxy`]: Self::ensure_proxy
    pub fn get_proxy<T: Interface + Debug>(&mut self, target: T) -> Option<T> {
        // - Decrease ref count of target via drop
        // - Increase ref count of proxy
        let target_ptr = target.as_raw();
        let result = self.target_to_proxy.get(&target_ptr).map(|proxy_ptr| unsafe { add_ref(transmute_copy::<_, T>(proxy_ptr)) });
        match &result {
            Some(proxy) => tracing::debug!("Retrieved {} proxy: {:p} (<=> {target_ptr:p})", type_name::<T>(), proxy.as_raw()),
            None => tracing::warn!("No {} proxy found: NOTFOUND (<=> {target_ptr:p})", type_name::<T>()),
        };
        result
    }

    /// Retrieves the original target COM object for a given proxy.
    ///
    /// This method performs the reverse lookup from proxy to target object.
    /// Returns `None` if the proxy is null or if no target mapping exists.
    ///
    /// # Type Parameters
    /// * `T` - The COM interface type that implements `Interface + Debug`
    /// * `K` - Type that can provide an optional reference to the COM interface
    ///
    /// # Arguments
    /// * `proxy` - The proxy object to find the target for (can be nullable)
    ///
    /// # Returns
    /// * `Some(NullableInterfaceOut<T>)` - Wrapper containing the target object pointer if found
    /// * `None` - If proxy is null or no target mapping exists
    ///
    /// # Reference Counting
    /// No reference count changes occur - both input and output are references
    ///
    /// # Note
    /// This method treats null proxy inputs as an error condition and returns `None`.
    /// For cases where null proxies should map to null targets, use [`get_target_nullable`].
    ///
    /// [`get_target_nullable`]: Self::get_target_nullable
    pub fn get_target<T: Interface + Debug, K: NullableInterfaceIn<T>>(&mut self, proxy: K) -> Option<NullableInterfaceOut<T>> {
        // - No ref count changes here, both input and output are references
        let proxy_ptr = match proxy.as_ref() {
            Some(obj_ref) => obj_ref.as_raw(),
            None => {
                tracing::warn!("Attempted to get target for a null proxy reference of type {}, treating as not found", type_name::<T>());
                return None;
            }
        };
        let result = self.proxy_to_target.get(&proxy_ptr).map(|target_ptr| NullableInterfaceOut::new(*target_ptr));
        match &result {
            Some(target) => tracing::debug!("Retrieved {} target of proxy: {proxy_ptr:p} (<=> {:p})", type_name::<T>(), target.as_raw()),
            None => tracing::warn!("No target found for {} proxy: {proxy_ptr:p} (<=> NOTFOUND)", type_name::<T>()),
        };
        result
    }

    /// Retrieves the original target COM object for a given proxy, handling null proxies gracefully.
    ///
    /// This method performs the reverse lookup from proxy to target object, but unlike
    /// [`get_target`], it treats null proxy inputs as valid and maps them to null targets.
    /// This is useful for COM method parameters that may legitimately be null.
    ///
    /// # Type Parameters
    /// * `T` - The COM interface type that implements `Interface + Debug`
    /// * `K` - Type that can provide an optional reference to the COM interface
    ///
    /// # Arguments
    /// * `proxy` - The proxy object to find the target for (can be nullable)
    ///
    /// # Returns
    /// * `Some(NullableInterfaceOut<T>)` - Wrapper containing the target object pointer if found,
    ///   or a null pointer if the proxy was null
    /// * `None` - Only if a non-null proxy has no target mapping
    ///
    /// # Reference Counting
    /// No reference count changes occur - both input and output are references
    ///
    /// # Difference from get_target
    /// - [`get_target`]: null proxy → `None`
    /// - [`get_target_nullable`]: null proxy → `Some(null_target)`
    ///
    /// [`get_target`]: Self::get_target
    /// [`get_target_nullable`]: Self::get_target_nullable
    pub fn get_target_nullable<T: Interface + Debug, K: NullableInterfaceIn<T>>(&mut self, proxy: K) -> Option<NullableInterfaceOut<T>> {
        // - No ref count changes here, both input and output are references
        let proxy_ptr = match proxy.as_ref() {
            Some(obj_ref) => obj_ref.as_raw(),
            None => {
                tracing::debug!("Returning nullptr for null proxy reference of type {}", type_name::<T>());
                return Some(NullableInterfaceOut::new(null_mut()));
            }
        };
        let result = self.proxy_to_target.get(&proxy_ptr).map(|target_ptr| NullableInterfaceOut::new(*target_ptr));
        match &result {
            Some(target) => tracing::debug!("Retrieved {} target of proxy: {proxy_ptr:p} (<=> {:p})", type_name::<T>(), target.as_raw()),
            None => tracing::warn!("No target found for {} proxy pointer: {proxy_ptr:p} (<=> NOTFOUND)", type_name::<T>()),
        };
        result
    }

    /// Removes the mapping for a proxy that is being destroyed.
    ///
    /// This method should be called when a proxy object is being destroyed to clean up
    /// the bidirectional mappings in the tracker. It removes both the target→proxy and
    /// proxy → target mappings to prevent memory leaks and stale references.
    ///
    /// # Type Parameters
    /// * `T` - The COM interface type that implements `Interface + Debug`
    ///
    /// # Arguments
    /// * `target` - Reference to the target object whose proxy is being destroyed
    ///
    /// # Note
    /// This method takes a reference to the target object rather than the proxy object
    /// because it's typically called from the proxy's `Drop` implementation where the
    /// proxy already has access to its target.
    ///
    /// # Reference Counting
    /// No reference count changes occur - this only removes mappings from internal hash maps
    ///
    /// # Example
    /// ```ignore
    /// impl Drop for ProxyDevice {
    ///     fn drop(&mut self) {
    ///         tracker.on_proxy_destroy(&self.target);
    ///     }
    /// }
    /// ```
    pub fn on_proxy_destroy<T: Interface + Debug>(&mut self, target: &T) {
        let target_ptr = target.as_raw();
        if let Some(proxy_ptr) = self.target_to_proxy.remove(&target_ptr) {
            self.proxy_to_target.remove(&proxy_ptr);
            tracing::debug!("{} proxy destroyed: {proxy_ptr:p} (<=> {target_ptr:p})", type_name::<T>());
        } else {
            tracing::warn!("{} proxy destroyed, but no entry found in storage for target pointer: NOTFOUND (<=> {target_ptr:p})", type_name::<T>());
        }
    }
}
