use std::ffi::{c_char, c_double, c_int, c_long};

use crate::{
    between,
    bindgen::{self, win, xw},
};

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

// DUMMY
pub fn xhints() {
    unsafe { bindgen::xhints() }
}

// DUMMY
pub(crate) fn xloadfonts(fontstr: *const c_char, fontsize: c_double) {
    unsafe { bindgen::xloadfonts(fontstr, fontsize) }
}

// DUMMY
pub(crate) fn xloadcols() {
    unsafe { bindgen::xloadcols() }
}

// DUMMY
pub(crate) fn ximopen(dpy: *mut bindgen::Display) -> c_int {
    unsafe { bindgen::ximopen(dpy) }
}

// DUMMY
pub(crate) extern "C" fn ximinstantiate(
    dpy: *mut bindgen::Display,
    client: bindgen::XPointer,
    call: bindgen::XPointer,
) {
    unsafe { bindgen::ximinstantiate(dpy, client, call) }
}

/// Set the `WINDOWID` environment variable to `xw.win`.
pub fn xsetenv() {
    unsafe {
        // TODO this can probably just be:
        // std::env::set_var("WINDOWID", xw.win.to_string());
        let mut buf = [0; c_long::BITS as usize + 1];
        libc::snprintf(
            buf.as_mut_ptr(),
            size_of_val(&buf),
            c"%lu".as_ptr(),
            xw.win,
        );
        libc::setenv(c"WINDOWID".as_ptr(), buf.as_ptr(), 1);
    }
}
