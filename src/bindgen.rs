#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::approx_constant)]
#![allow(unused)]

use std::{
    ffi::{c_char, c_double, c_int},
    ptr::null_mut,
};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

unsafe extern "C" {
    pub static mut usedfontsize: c_double;
    pub static mut defaultfontsize: c_double;
    pub static mut shell: *mut c_char;
}

impl Term {
    pub(crate) fn line(ptr: *mut Term, i: c_int, j: c_int) -> *mut Glyph_ {
        unsafe { (*(*ptr).line.offset(i as isize)).offset(j as isize) }
    }
}
