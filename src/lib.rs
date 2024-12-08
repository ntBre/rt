use libc::c_int;

pub mod bindgen {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(improper_ctypes)]
    #![allow(clippy::upper_case_acronyms)]
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
}

use bindgen::{defaultfg, term, treset, tresize, Glyph_, TCursor, Term};

/// Initialize the global terminal in `term` to the given size and with default
/// foreground and background colors.
pub fn tnew(col: c_int, row: c_int) {
    unsafe {
        term = Term {
            c: TCursor {
                attr: Glyph_ { u: 0, mode: 0, fg: defaultfg, bg: defaultfg },
                x: 0,
                y: 0,
                state: 0,
            },
            ..Default::default()
        };
        tresize(col, row);
        treset();
    }
}
