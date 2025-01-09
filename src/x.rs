use std::{
    ffi::{c_char, c_double, c_int, c_long, c_void},
    mem::MaybeUninit,
    ptr::null_mut,
};

use x11::xlib::{
    InputHint, NorthEastGravity, NorthWestGravity, PBaseSize, PMaxSize,
    PMinSize, PResizeInc, PSize, PWinGravity, SouthEastGravity,
    SouthWestGravity, Success, USPosition, XIMPreeditNothing, XIMStatusNothing,
    XNegative, XUTF8StringStyle, XValue, YNegative, YValue,
};

use crate::{
    between,
    bindgen::{
        self, borderpx, opt_class, opt_name, opt_title, termname, win, xw,
        XAllocSizeHints, XClassHint, XCreateIC, XICCallback, XIMCallback,
        XNDestroyCallback, XPointer, XSetIMValues, XVaCreateNestedList,
        XWMHints, XIC, XIM,
    },
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

pub fn xhints() {
    unsafe {
        let mut class = XClassHint {
            res_name: if !opt_name.is_null() { opt_name } else { termname },
            res_class: if !opt_class.is_null() { opt_class } else { termname },
        };
        let mut wm = XWMHints {
            flags: InputHint,
            input: 1,
            initial_state: 0,
            icon_pixmap: 0,
            icon_window: 0,
            icon_x: 0,
            icon_y: 0,
            icon_mask: 0,
            window_group: 0,
        };

        let sizeh = XAllocSizeHints();
        (*sizeh).flags = PSize | PResizeInc | PBaseSize | PMinSize;
        (*sizeh).height = win.h;
        (*sizeh).width = win.w;
        (*sizeh).height_inc = win.ch;
        (*sizeh).width_inc = win.cw;
        (*sizeh).base_height = 2 * borderpx;
        (*sizeh).base_width = 2 * borderpx;
        (*sizeh).min_height = win.ch + 2 * borderpx;
        (*sizeh).min_width = win.cw + 2 * borderpx;

        if xw.isfixed != 0 {
            (*sizeh).flags |= PMaxSize;
            (*sizeh).min_width = win.w;
            (*sizeh).max_width = win.w;
            (*sizeh).min_height = win.h;
            (*sizeh).max_height = win.h;
        }
        if (xw.gm & (XValue | YValue)) != 0 {
            (*sizeh).flags |= USPosition | PWinGravity;
            (*sizeh).x = xw.l;
            (*sizeh).y = xw.t;
            (*sizeh).win_gravity = xgeommasktogravity(xw.gm);
        }

        bindgen::XSetWMProperties(
            xw.dpy,
            xw.win,
            null_mut(),
            null_mut(),
            null_mut(),
            0,
            sizeh,
            &raw mut wm,
            &raw mut class,
        );
        bindgen::XFree(sizeh.cast());
    }
}

fn xgeommasktogravity(mask: c_int) -> c_int {
    #[allow(non_upper_case_globals)]
    match mask & (XNegative | YNegative) {
        0 => NorthWestGravity,
        XNegative => NorthEastGravity,
        YNegative => SouthWestGravity,
        _ => SouthEastGravity,
    }
}

// DUMMY
pub(crate) fn xloadfonts(fontstr: *const c_char, fontsize: c_double) {
    unsafe { bindgen::xloadfonts(fontstr, fontsize) }
}

// DUMMY
pub(crate) fn xloadcols() {
    unsafe { bindgen::xloadcols() }
}

pub(crate) fn ximopen(_dpy: *mut bindgen::Display) -> c_int {
    unsafe {
        let mut imdestroy =
            XIMCallback { client_data: null_mut(), callback: Some(ximdestroy) };
        let icdestroy =
            XICCallback { client_data: null_mut(), callback: Some(xicdestroy) };

        xw.ime.xim =
            bindgen::XOpenIM(xw.dpy, null_mut(), null_mut(), null_mut());
        if xw.ime.xim.is_null() {
            return 0;
        }

        if !XSetIMValues(
            xw.ime.xim,
            XNDestroyCallback,
            &raw mut imdestroy,
            null_mut::<c_void>(),
        )
        .is_null()
        {
            eprintln!("XSetIMValues: Could not set XNDestroyCallback.");
        }

        // NOTE these variadic functions take key-value pairs, terminated by
        // NULL. the types of the keys must be *exactly* right or this will
        // segfault. using the keys from x11 causes a segfault, so these must be
        // the bindgen versions even if they superficially look compatible. the
        // bindgen versions seem to be [u8], while the x11 versions are &str.
        xw.ime.spotlist = XVaCreateNestedList(
            0,
            bindgen::XNSpotLocation,
            &raw mut xw.ime.spot,
            null_mut::<c_void>(),
        );

        if xw.ime.xic.is_null() {
            xw.ime.xic = XCreateIC(
                xw.ime.xim,
                bindgen::XNInputStyle,
                XIMPreeditNothing | XIMStatusNothing,
                bindgen::XNClientWindow,
                xw.win,
                bindgen::XNDestroyCallback,
                &icdestroy,
                null_mut::<c_void>(),
            );
        }

        if xw.ime.xic.is_null() {
            eprintln!("XCreateIC: Could not create input context.");
        }

        1
    }
}

extern "C" fn ximdestroy(_xim: XIM, _client: XPointer, _call: XPointer) {
    unsafe {
        xw.ime.xim = null_mut();
        bindgen::XRegisterIMInstantiateCallback(
            xw.dpy,
            null_mut(),
            null_mut(),
            null_mut(),
            Some(ximinstantiate),
            null_mut(),
        );
        bindgen::XFree(xw.ime.spotlist);
    }
}

extern "C" fn xicdestroy(
    _xim: XIC,
    _client: XPointer,
    _call: XPointer,
) -> c_int {
    unsafe {
        xw.ime.xic = null_mut();
        1
    }
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
