extern crate libc;

use std::ffi::CStr;
use std::{ptr, mem, slice};

pub type ChromaprintContext = *mut ::libc::c_void;
pub type ChromaprintAlgorithm = ::libc::c_int;
pub const CHROMAPRINT_ALGORITHM_TEST1: ChromaprintAlgorithm = 0;
pub const CHROMAPRINT_ALGORITHM_TEST2: ChromaprintAlgorithm = 1;
pub const CHROMAPRINT_ALGORITHM_TEST3: ChromaprintAlgorithm = 2;
pub const CHROMAPRINT_ALGORITHM_TEST4: ChromaprintAlgorithm = 3;
pub const CHROMAPRINT_ALGORITHM_DEFAULT: ChromaprintAlgorithm  = CHROMAPRINT_ALGORITHM_TEST2;
#[link(name = "chromaprint")]
extern "C" {
    pub fn chromaprint_get_version() -> *const ::libc::c_char;
    pub fn chromaprint_new(algorithm: ChromaprintAlgorithm)
     -> *mut ChromaprintContext;
    pub fn chromaprint_free(ctx: *mut ChromaprintContext) -> ();
    pub fn chromaprint_get_algorithm(ctx: *mut ChromaprintContext)
     -> ChromaprintAlgorithm;
    pub fn chromaprint_set_option(ctx: *mut ChromaprintContext,
                                  name: *const ::libc::c_char,
                                  value: ::libc::c_int) -> ::libc::c_int;
    pub fn chromaprint_start(ctx: *mut ChromaprintContext,
                             sample_rate: ::libc::c_int,
                             num_channels: ::libc::c_int) -> ::libc::c_int;
    pub fn chromaprint_feed(ctx: *mut ChromaprintContext,
                            data: *const ::libc::c_void, size: ::libc::c_int)
     -> ::libc::c_int;
    pub fn chromaprint_finish(ctx: *mut ChromaprintContext) -> ::libc::c_int;
    pub fn chromaprint_get_fingerprint(ctx: *mut ChromaprintContext,
                                       fingerprint: *mut *mut ::libc::c_char)
     -> ::libc::c_int;
    pub fn chromaprint_get_raw_fingerprint(ctx: *mut ChromaprintContext,
                                           fingerprint:
                                               *mut *mut ::libc::c_void,
                                           size: *mut ::libc::c_int)
     -> ::libc::c_int;
    pub fn chromaprint_encode_fingerprint(fp: *const ::libc::c_void,
                                          size: ::libc::c_int,
                                          algorithm: ::libc::c_int,
                                          encoded_fp:
                                              *mut *mut ::libc::c_void,
                                          encoded_size: *mut ::libc::c_int,
                                          base64: ::libc::c_int)
     -> ::libc::c_int;
    pub fn chromaprint_decode_fingerprint(encoded_fp: *const ::libc::c_void,
                                          encoded_size: ::libc::c_int,
                                          fp: *mut *mut ::libc::c_void,
                                          size: *mut ::libc::c_int,
                                          algorithm: *mut ::libc::c_int,
                                          base64: ::libc::c_int)
     -> ::libc::c_int;
    pub fn chromaprint_dealloc(ptr: *mut ::libc::c_void) -> ();
}

pub struct Chromaprint {
    ctx: *mut ChromaprintContext,
}

impl Chromaprint {
    pub fn new() -> Chromaprint {
        unsafe {
            Chromaprint {
                ctx:  chromaprint_new(CHROMAPRINT_ALGORITHM_DEFAULT),
            }
        }
    }

    pub fn version() -> String {
        String::from_utf8(unsafe { CStr::from_ptr(chromaprint_get_version()).to_bytes().to_vec() }).unwrap()
    }

    pub fn algorithm(&self) -> ChromaprintAlgorithm {
        unsafe { chromaprint_get_algorithm(self.ctx) }
    }

    pub fn start(&mut self, sample_rate: ::libc::c_int, num_channels: ::libc::c_int) -> bool {
        unsafe { chromaprint_start(self.ctx, sample_rate, num_channels) == 1 }
    }

    pub fn feed(&mut self, data: &[i16]) -> bool {
        unsafe { chromaprint_feed(self.ctx, data.as_ptr() as *const ::libc::c_void, data.len() as ::libc::c_int) == 1 }
    }

    pub fn finish(&mut self) -> bool {
        unsafe { chromaprint_finish(self.ctx) == 1 }
    }

    pub fn fingerprint(&mut self) -> Option<String> {
        let mut fingerprint: *mut ::libc::c_char = ptr::null_mut();
        if unsafe { chromaprint_get_fingerprint(self.ctx, &mut fingerprint) } == 1 {
            let ret = String::from_utf8(unsafe { CStr::from_ptr(fingerprint) }.to_bytes().to_vec()).ok();
            unsafe { chromaprint_dealloc(fingerprint as *mut ::libc::c_void) }
            return ret;
        }
        None
    }

    pub fn raw_fingerprint(&mut self) -> Option<Vec<::libc::c_int>> {
        let mut array: *mut ::libc::c_int = ptr::null_mut();
        let mut size: ::libc::c_int = 0;
        if unsafe { chromaprint_get_raw_fingerprint(self.ctx, mem::transmute::<_, *mut *mut ::libc::c_void>(&mut array), &mut size) } == 1 {
            let fingerprint = unsafe { slice::from_raw_parts(array, size as usize).to_vec() };
            unsafe { chromaprint_dealloc(array as *mut ::libc::c_void) }
            return Some(fingerprint);
        }
        None
    }

    pub fn encode(raw_fingerprint: &[::libc::c_int], algorithm: ChromaprintAlgorithm, base64: bool) -> Option<Vec<u8>> {
        let mut array: *mut u8 = ptr::null_mut();
        let mut size: ::libc::c_int = 0;
        let result = unsafe { 
            chromaprint_encode_fingerprint(raw_fingerprint.as_ptr() as *const ::libc::c_void, 
                                           raw_fingerprint.len() as ::libc::c_int, algorithm, 
                                           mem::transmute::<_, *mut *mut ::libc::c_void>(&mut array), 
                                           &mut size, base64 as ::libc::c_int) 
        };
        if result == 1 {
            let encoded = unsafe { slice::from_raw_parts(array, size as usize).to_vec() };
            unsafe { chromaprint_dealloc(array as *mut ::libc::c_void) }
            return Some(encoded);
        }
        None
    }

    pub fn decode(encoded_fingerprint: &[u8], base64: bool) -> Option<(Vec<::libc::c_int>, ChromaprintAlgorithm)> {
        let mut array: *mut ::libc::c_int = ptr::null_mut();
        let mut size: ::libc::c_int = 0;
        let mut algorithm: ChromaprintAlgorithm = -1;
        let result = unsafe {
            chromaprint_decode_fingerprint(encoded_fingerprint.as_ptr() as *const ::libc::c_void, 
                                           encoded_fingerprint.len() as ::libc::c_int, 
                                           mem::transmute::<_, *mut *mut ::libc::c_void>(&mut array), 
                                           &mut size, &mut algorithm, base64 as ::libc::c_int)
        };
        if result == 1 {
            let decoded = unsafe { slice::from_raw_parts(array, size as usize).to_vec() };
            unsafe { chromaprint_dealloc(array as *mut ::libc::c_void) }
            return Some((decoded, algorithm));
        }
        None
    }
}

impl Drop for Chromaprint {
    fn drop(&mut self) {
        unsafe { chromaprint_free(self.ctx); }
    }
}
