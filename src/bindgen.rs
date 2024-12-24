#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::approx_constant)]
#![allow(unused)]

use std::{ffi::c_char, ptr::null_mut};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

impl Default for Term {
    fn default() -> Self {
        Term {
            c: TCursor {
                attr: Glyph_ { u: 0, mode: 0, fg: 0, bg: 0 },
                x: 0,
                y: 0,
                state: 0,
            },
            // not mentioned in st
            row: 0,
            col: 0,
            line: null_mut(),
            alt: null_mut(),
            dirty: null_mut(),
            ocx: 0,
            ocy: 0,
            top: 0,
            bot: 0,
            mode: 0,
            esc: 0,
            trantbl: [0 as c_char; 4],
            charset: 0,
            icharset: 0,
            tabs: null_mut(),
            lastc: 0,
        }
    }
}
