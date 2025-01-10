use std::{
    ffi::{c_char, c_double, c_int, c_long, c_ushort, c_void, CStr},
    mem::MaybeUninit,
    ptr::null_mut,
};

use libc::strlen;
use x11::xlib::{
    False, InputHint, NorthEastGravity, NorthWestGravity, PBaseSize, PMaxSize,
    PMinSize, PResizeInc, PSize, PWinGravity, SouthEastGravity,
    SouthWestGravity, Success, USPosition, XIMPreeditNothing, XIMStatusNothing,
    XNegative, XUTF8StringStyle, XValue, YNegative, YValue,
};

use crate::{
    between,
    bindgen::{
        self, ascii_printable, borderpx, chscale, colorname, cwscale, dc,
        defaultbg, defaultfg, defaultfontsize, opt_class, opt_name, opt_title,
        termname, usedfontsize, win, xw, Color, FcChar8, FcConfigSubstitute,
        FcFontMatch, FcNameParse, FcPattern, FcPatternAddDouble,
        FcPatternAddInteger, FcPatternDel, FcPatternDestroy,
        FcPatternDuplicate, FcPatternGetDouble, FcPatternGetInteger, Font_,
        Glyph_, XAllocSizeHints, XClassHint, XCopyArea, XCreateIC, XICCallback,
        XIMCallback, XNDestroyCallback, XNPreeditAttributes, XPointer,
        XRenderColor, XSetForeground, XSetICValues, XSetIMValues,
        XVaCreateNestedList, XWMHints, XftColorAllocName, XftColorAllocValue,
        XftColorFree, XftDefaultSubstitute, XftFontOpenPattern,
        XftTextExtentsUtf8, XftXlfdParse, _FcMatchKind_FcMatchPattern,
        _FcResult_FcResultMatch, FC_PIXEL_SIZE, FC_SIZE, FC_SLANT,
        FC_SLANT_ITALIC, FC_SLANT_ROMAN, FC_WEIGHT, FC_WEIGHT_BOLD, XIC, XIM,
    },
    die, is_set, len,
    win::{MODE_REVERSE, MODE_VISIBLE},
    xmalloc,
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

pub(crate) fn xloadfonts(fontstr: *const c_char, fontsize: c_double) {
    unsafe {
        let pattern = if *fontstr.offset(0) == b'-' as c_char {
            XftXlfdParse(fontstr, False, False)
        } else {
            FcNameParse(fontstr as *const FcChar8)
        };

        if pattern.is_null() {
            die!("can't open font {:?}", CStr::from_ptr(fontstr));
        }

        let mut fontval = 0.0;
        if fontsize > 1.0 {
            FcPatternDel(pattern, FC_PIXEL_SIZE.as_ptr() as *const _);
            FcPatternDel(pattern, FC_SIZE.as_ptr() as *const _);
            FcPatternAddDouble(
                pattern,
                FC_PIXEL_SIZE.as_ptr() as *const _,
                fontsize,
            );
            usedfontsize = fontsize;
        } else {
            if FcPatternGetDouble(
                pattern,
                FC_PIXEL_SIZE.as_ptr().cast(),
                0,
                &mut fontval,
            ) == _FcResult_FcResultMatch
            {
                usedfontsize = fontval;
            } else if FcPatternGetDouble(
                pattern,
                FC_SIZE.as_ptr().cast(),
                0,
                &mut fontval,
            ) == _FcResult_FcResultMatch
            {
                usedfontsize = -1.0;
            } else {
                // Default font size is 12, if none given. This is to have a
                // known usedfontsize value.
                FcPatternAddDouble(
                    pattern,
                    FC_PIXEL_SIZE.as_ptr().cast(),
                    12.0,
                );
                usedfontsize = 12.0;
            }
            defaultfontsize = usedfontsize;
        }

        if xloadfont(&raw mut dc.font, pattern) != 0 {
            die!("can't open font {:?}", CStr::from_ptr(fontstr));
        }

        if usedfontsize < 0.0 {
            FcPatternGetDouble(
                (*dc.font.match_).pattern,
                FC_PIXEL_SIZE.as_ptr().cast(),
                0,
                &mut fontval,
            );
            usedfontsize = fontval;
            if fontsize == 0.0 {
                defaultfontsize = fontval;
            }
        }

        /* Setting character width and height. */
        win.cw = bindgen::ceilf(dc.font.width as f32 * cwscale) as i32;
        win.ch = bindgen::ceilf(dc.font.height as f32 * chscale) as i32;

        FcPatternDel(pattern, FC_SLANT.as_ptr().cast());
        FcPatternAddInteger(
            pattern,
            FC_SLANT.as_ptr().cast(),
            FC_SLANT_ITALIC as i32,
        );
        if xloadfont(&raw mut dc.ifont, pattern) != 0 {
            die!("can't open font {:?}", fontstr);
        }

        FcPatternDel(pattern, FC_WEIGHT.as_ptr().cast());
        FcPatternAddInteger(
            pattern,
            FC_WEIGHT.as_ptr().cast(),
            FC_WEIGHT_BOLD as i32,
        );
        if xloadfont(&raw mut dc.ibfont, pattern) != 0 {
            die!("can't open font {:?}", fontstr);
        }

        FcPatternDel(pattern, FC_SLANT.as_ptr().cast());
        FcPatternAddInteger(
            pattern,
            FC_SLANT.as_ptr().cast(),
            FC_SLANT_ROMAN as i32,
        );
        if xloadfont(&raw mut dc.bfont, pattern) != 0 {
            die!("can't open font {:?}", fontstr);
        }

        FcPatternDestroy(pattern);
    }
}

// TODO this returns an error code, 1 on failure and 0 on success so return a
// result instead
fn xloadfont(f: *mut Font_, pattern: *mut FcPattern) -> c_int {
    unsafe {
        // Manually configure instead of calling XftMatchFont so that we can use
        // the configured pattern for "missing glyph" lookups.
        let configured = FcPatternDuplicate(pattern);
        if configured.is_null() {
            return 1;
        }

        FcConfigSubstitute(null_mut(), configured, _FcMatchKind_FcMatchPattern);
        XftDefaultSubstitute(xw.dpy, xw.scr, configured);

        let mut result = MaybeUninit::uninit();
        let match_ = FcFontMatch(null_mut(), configured, result.as_mut_ptr());
        if match_.is_null() {
            FcPatternDestroy(configured);
            return 1;
        }

        (*f).match_ = XftFontOpenPattern(xw.dpy, match_);
        if (*f).match_.is_null() {
            FcPatternDestroy(configured);
            FcPatternDestroy(match_);
            return 1;
        }

        // this is a #define for XftPatternGetInteger from XftCompat.h, same
        // with the result (Xft -> Fc)
        let mut wantattr = 0;
        if FcPatternGetInteger(pattern, c"slant".as_ptr(), 0, &mut wantattr)
            == _FcResult_FcResultMatch
        {
            // Check if xft was unable to find a font with the appropriate slant
            // but gave us one anyway. Try to mitigate.
            let mut haveattr = 0;
            if (FcPatternGetInteger(
                (*(*f).match_).pattern,
                c"slant".as_ptr(),
                0,
                &mut haveattr,
            ) != _FcResult_FcResultMatch)
                || haveattr < wantattr
            {
                (*f).badslant = 1;
                eprintln!("font slant does not match");
            }
        }

        if FcPatternGetInteger(pattern, c"weight".as_ptr(), 0, &mut wantattr)
            == _FcResult_FcResultMatch
        {
            // Same check as above but for weights
            let mut haveattr = 0;
            if (FcPatternGetInteger(
                (*(*f).match_).pattern,
                c"weight".as_ptr(),
                0,
                &mut haveattr,
            ) != _FcResult_FcResultMatch)
                || haveattr != wantattr
            {
                (*f).badweight = 1;
                eprintln!("font weight does not match");
            }
        }

        let ap_len = strlen(ascii_printable.as_ptr().cast()) as c_int;

        let mut extents = MaybeUninit::uninit();
        XftTextExtentsUtf8(
            xw.dpy,
            (*f).match_,
            ascii_printable as *const FcChar8,
            ap_len,
            extents.as_mut_ptr(),
        );
        let extents = extents.assume_init();

        (*f).set = null_mut();
        (*f).pattern = configured;

        (*f).ascent = (*(*f).match_).ascent;
        (*f).descent = (*(*f).match_).descent;
        (*f).lbearing = 0;
        (*f).rbearing = (*(*f).match_).max_advance_width as i16;

        (*f).height = (*f).ascent + (*f).descent;
        (*f).width = divceil(extents.xOff as c_int, ap_len);

        0
    }
}

#[inline]
fn divceil(n: c_int, d: c_int) -> c_int {
    (n + d - 1) / d
}

/// Load colors.
pub(crate) fn xloadcols() {
    unsafe {
        // TODO LazyLock
        static mut LOADED: bool = false;

        if LOADED {
            let mut cp = dc.col;
            while cp < dc.col.add(dc.collen) {
                XftColorFree(xw.dpy, xw.vis, xw.cmap, cp);
                cp = cp.offset(1);
            }
        } else {
            dc.collen = std::cmp::max(len(&raw const colorname), 256);
            dc.col = xmalloc(dc.collen * size_of::<Color>()).cast();
        }

        // TODO fix this, pretty hard with dc being a mutable static, though.
        #[allow(clippy::needless_range_loop)]
        for i in 0..dc.collen {
            if xloadcolor(i as c_int, null_mut(), dc.col.add(i)) == 0 {
                if !colorname[i].is_null() {
                    die!(
                        "could not allocate color {:?}",
                        CStr::from_ptr(colorname[i])
                    );
                } else {
                    die!("could not allocate color {i}");
                }
            }
        }
    }
}

fn xloadcolor(i: c_int, mut name: *const c_char, ncolor: *mut Color) -> c_int {
    unsafe {
        let mut color =
            XRenderColor { red: 0, green: 0, blue: 0, alpha: 0xffff };

        if name.is_null() {
            if between(i, 16, 255) {
                // 256 color
                if i < 6 * 6 * 6 + 16 {
                    // same colors as xterm
                    color.red = sixd_to_16bit(((i - 16) / 36) % 6);
                    color.green = sixd_to_16bit(((i - 16) / 6) % 6);
                    color.blue = sixd_to_16bit((i - 16) % 6);
                } else {
                    // greyscale
                    color.red = 0x0808 + 0x0a0a * (i as u16 - (6 * 6 * 6 + 16));
                    color.green = color.red;
                    color.blue = color.red;
                }
                return XftColorAllocValue(
                    xw.dpy,
                    xw.vis,
                    xw.cmap,
                    &raw mut color,
                    ncolor,
                );
            } else {
                name = colorname[i as usize];
            }
        }

        XftColorAllocName(xw.dpy, xw.vis, xw.cmap, name, ncolor)
    }
}

fn sixd_to_16bit(x: c_int) -> c_ushort {
    if x == 0 {
        0
    } else {
        (0x3737 + 0x2828 * x) as c_ushort
    }
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

pub(crate) extern "C" fn ximinstantiate(
    dpy: *mut bindgen::Display,
    _client: bindgen::XPointer,
    _call: bindgen::XPointer,
) {
    unsafe {
        if ximopen(dpy) != 0 {
            bindgen::XUnregisterIMInstantiateCallback(
                xw.dpy,
                null_mut(),
                null_mut(),
                null_mut(),
                Some(ximinstantiate),
                null_mut(),
            );
        }
    }
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

pub fn startdraw() -> bool {
    is_set(MODE_VISIBLE)
}

// DUMMY
pub(crate) fn drawcursor(
    cx: i32,
    cy: i32,
    g: Glyph_,
    ox: i32,
    oy: i32,
    og: Glyph_,
) {
    unsafe { bindgen::xdrawcursor(cx, cy, g, ox, oy, og) }
}

pub(crate) fn finishdraw() {
    unsafe {
        XCopyArea(
            xw.dpy,
            xw.buf,
            xw.win,
            dc.gc,
            0,
            0,
            win.w as u32,
            win.h as u32,
            0,
            0,
        );
        XSetForeground(
            xw.dpy,
            dc.gc,
            (*dc.col.add(if is_set(MODE_REVERSE) {
                defaultfg
            } else {
                defaultbg
            } as usize))
            .pixel,
        );
    }
}

pub(crate) fn ximspot(x: i32, y: i32) {
    unsafe {
        if xw.ime.xic.is_null() {
            return;
        }

        xw.ime.spot.x = (borderpx + x * win.cw) as i16;
        xw.ime.spot.y = (borderpx + (y + 1) * win.ch) as i16;

        XSetICValues(
            xw.ime.xic,
            XNPreeditAttributes,
            xw.ime.spotlist,
            null_mut::<c_void>(),
        );
    }
}

// DUMMY
pub(crate) fn drawline(line: *mut Glyph_, x1: i32, y1: i32, x2: i32) {
    unsafe {
        bindgen::xdrawline(line, x1, y1, x2);
    }
}
