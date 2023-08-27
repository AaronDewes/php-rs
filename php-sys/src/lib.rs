#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use std::os::raw::{c_char, c_uchar, c_void};

extern "C" {
    pub fn sg_request_info() -> *mut sapi_request_info;
    pub fn sg_server_context() -> *mut c_void;
    pub fn sg_set_server_context(context: *mut c_void);
    pub fn sg_sapi_headers() -> *mut sapi_headers_struct;
    pub fn sg_headers_sent() -> c_uchar;
    pub fn sg_set_headers_sent(is_sent: c_uchar);
    pub fn zend_tsrmls_cache_update();
    pub fn phprpm_fopen(filename: *const c_char, mode: *const c_char) -> *mut FILE;
    pub fn _zend_string_init(str: *const c_char, len: usize, persistent: bool) -> *mut zend_string;
    pub fn _zend_string_release(s: *mut zend_string);
    pub fn _zend_string_release_ex(s: *mut zend_string, persistent: bool);
    pub fn _zend_string_realloc(s: *mut zend_string, len: usize, persistent: bool) -> *mut zend_string;
    pub fn _zend_string_extend(s: *mut zend_string, len: usize, persistent: bool) -> *mut zend_string;
    pub fn _zend_string_truncate(s: *mut zend_string, len: usize, persistent: bool) -> *mut zend_string;
    pub fn _zend_string_dup(s: *mut zend_string, persistent: bool) -> *mut zend_string;
    pub fn _zend_string_copy(s: *mut zend_string) -> *mut zend_string;
}

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
