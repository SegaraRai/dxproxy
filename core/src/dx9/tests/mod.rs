//! Testing module for DirectX 9 proxy functionality.
//!
//! This module provides test utilities for verifying DirectX 9 proxy behavior,
//! including mock objects for COM interfaces and test helpers.

use crate::common::*;
use crate::dx9::com::*;
use std::ffi::c_void;
use windows::{Win32::Graphics::Direct3D9::*, core::*};

/// Helper to create a mock DX9 device context for testing
fn create_test_context() -> DX9ProxyDeviceContext {
    DX9ProxyDeviceContext::new(DX9ProxyConfig::default())
}

#[cfg(test)]
mod mock {
    use super::*;
    
    /// Simple mock for IDirect3DQuery9 interface
    pub struct MockDirect3DQuery9 {
        query_type: D3DQUERYTYPE,
        data_size: u32,
        issued: bool,
    }
    
    impl MockDirect3DQuery9 {
        pub fn new(query_type: D3DQUERYTYPE, data_size: u32) -> Self {
            Self {
                query_type,
                data_size,
                issued: false,
            }
        }
        
        pub fn as_raw(&self) -> *mut c_void {
            self as *const _ as *mut c_void
        }
        
        pub fn get_type(&self) -> D3DQUERYTYPE {
            self.query_type
        }
        
        pub fn get_data_size(&self) -> u32 {
            self.data_size
        }
        
        pub fn issue(&mut self, _flags: u32) -> Result<()> {
            self.issued = true;
            Ok(())
        }
        
        pub fn get_data(&self, data: *mut c_void, _size: u32, _flags: u32) -> Result<()> {
            if !self.issued {
                return Err(D3DERR_DEVICELOST.into());
            }
            
            // For testing, we just write the query type as an integer to the data pointer
            if !data.is_null() {
                unsafe {
                    *(data as *mut i32) = self.query_type.0 as i32;
                }
            }
            
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::mock::*;
    
    #[test]
    fn test_com_mapping_with_dx9_objects() {
        let context = create_test_context();
        
        // Get access to the internal tracker for verification
        let tracker = &context.0.tracker;
        let mut tracker = tracker.lock().unwrap();
        
        // Test that pointers can be mapped - basic lifecycle test
        let target_ptr = Box::into_raw(Box::new(42)) as *mut c_void;
        let proxy_ptr = Box::into_raw(Box::new(24)) as *mut c_void;
        
        // Add mappings
        tracker.target_to_proxy.insert(target_ptr, proxy_ptr);
        tracker.proxy_to_target.insert(proxy_ptr, target_ptr);
        
        // Verify mappings
        assert_eq!(tracker.target_to_proxy.get(&target_ptr), Some(&proxy_ptr));
        assert_eq!(tracker.proxy_to_target.get(&proxy_ptr), Some(&target_ptr));
        
        // Cleanup
        unsafe {
            let _ = Box::from_raw(target_ptr as *mut i32);
            let _ = Box::from_raw(proxy_ptr as *mut i32);
        }
    }
}