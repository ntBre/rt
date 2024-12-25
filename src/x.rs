use std::{
    ffi::{c_char, c_double, c_int, c_long},
    mem::MaybeUninit,
};

use x11::xlib::{Success, XUTF8StringStyle};

use crate::{
    between,
    bindgen::{self, opt_title, win, xw},
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

pub(crate) fn xsettitle(mut p: *mut c_char) {
    unsafe {
        p = if p.is_null() { opt_title } else { p };

        if *p == b'\0' as c_char {
            p = opt_title;
        }

        let mut prop = MaybeUninit::uninit();
        if bindgen::Xutf8TextListToTextProperty(
            xw.dpy,
            &raw mut p,
            1,
            XUTF8StringStyle as u32,
            prop.as_mut_ptr(),
        ) != Success as i32
        {
            return;
        }

        bindgen::XSetWMName(xw.dpy, xw.win, prop.as_mut_ptr());
        bindgen::XSetTextProperty(
            xw.dpy,
            xw.win,
            prop.as_mut_ptr(),
            xw.netwmname,
        );
        let prop = prop.assume_init();
        bindgen::XFree(prop.value.cast());
    }
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
