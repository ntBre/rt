use std::ffi::c_long;

use libc::{c_int, setenv};

pub mod bindgen {
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
}

use bindgen::{
    defaultfg, sel, selection_mode_SEL_IDLE, snprintf, term, xw, Glyph_,
    TCursor, Term,
};

#[inline]
pub fn between<T>(x: T, a: T, b: T) -> bool
where
    T: PartialOrd,
{
    (a..=b).contains(&x)
}

pub mod x {
    use std::ffi::c_int;

    use crate::{between, bindgen::win};

    // NOTE returns bool?
    pub fn xsetcursor(cursor: c_int) -> c_int {
        // NOTE(st): 7: st extension
        if !between(cursor, 0, 7) {
            return 1;
        }
        unsafe {
            win.cursor = cursor;
        }

        0
    }
}

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

pub fn tresize(col: c_int, row: c_int) {
    unsafe { bindgen::tresize(col, row) }
}

pub fn treset() {
    unsafe { bindgen::treset() }
}

pub fn xinit(col: c_int, row: c_int) {
    unsafe { bindgen::xinit(col, row) }
}

/// Set the `WINDOWID` environment variable to `xw.win`.
pub fn xsetenv() {
    unsafe {
        // TODO this can probably just be:
        // std::env::set_var("WINDOWID", xw.win.to_string());
        let mut buf = [0; c_long::BITS as usize + 1];
        snprintf(
            buf.as_mut_ptr(),
            size_of_val(&buf) as u64,
            c"%lu".as_ptr(),
            xw.win,
        );
        setenv(c"WINDOWID".as_ptr(), buf.as_ptr(), 1);
    }
}

/// Initialize the global selection in `sel`.
pub fn selinit() {
    unsafe {
        sel.mode = selection_mode_SEL_IDLE as i32;
        sel.snap = 0;
        sel.ob.x = -1;
    }
}

pub fn run() {
    unsafe { bindgen::run() }
}
