use std::{
    ffi::{c_int, c_uchar, c_void, CStr},
    ptr::{null, null_mut},
};

use libc::{getpid, memset, strtol, CLOCK_MONOTONIC};
use x11::xlib::{
    False, GCGraphicsExposures, PropModeReplace, XA_CARDINAL, XA_STRING,
};

use bindgen::{
    borderpx, colorname, dc, defaultbg, defaultfg, font, mousebg, mousefg,
    mouseshape, opt_embed, opt_font, sel, tabspaces, term, usedfont, win, xsel,
    xw, FcInit, GlyphFontSpec, Glyph_, Line, TCursor, Term, XGCValues,
};
use win::MODE_NUMLOCK;

pub mod bindgen;
pub mod win;
pub mod x;

pub use x::xsetenv;

// enum glyph_attribute
pub const ATTR_NULL: c_int = 0;
pub const ATTR_BOLD: c_int = 1 << 0;
pub const ATTR_FAINT: c_int = 1 << 1;
pub const ATTR_ITALIC: c_int = 1 << 2;
pub const ATTR_UNDERLINE: c_int = 1 << 3;
pub const ATTR_BLINK: c_int = 1 << 4;
pub const ATTR_REVERSE: c_int = 1 << 5;
pub const ATTR_INVISIBLE: c_int = 1 << 6;
pub const ATTR_STRUCK: c_int = 1 << 7;
pub const ATTR_WRAP: c_int = 1 << 8;
pub const ATTR_WIDE: c_int = 1 << 9;
pub const ATTR_WDUMMY: c_int = 1 << 10;
pub const ATTR_BOLD_FAINT: c_int = ATTR_BOLD | ATTR_FAINT;

// enum selection_mode
pub const SEL_IDLE: c_int = 0;
pub const SEL_EMPTY: c_int = 1;
pub const SEL_READY: c_int = 2;

// enum selection_type
pub const SEL_REGULAR: c_int = 1;
pub const SEL_RECTANGULAR: c_int = 2;

// enum term_mode
pub const MODE_WRAP: c_int = 1 << 0;
pub const MODE_INSERT: c_int = 1 << 1;
pub const MODE_ALTSCREEN: c_int = 1 << 2;
pub const MODE_CRLF: c_int = 1 << 3;
pub const MODE_ECHO: c_int = 1 << 4;
pub const MODE_PRINT: c_int = 1 << 5;
pub const MODE_UTF8: c_int = 1 << 6;

// enum cursor_movement
// TODO these are definitely used like rust enums
pub const CURSOR_SAVE: c_int = 0;
pub const CURSOR_LOAD: c_int = 1;

// enum cursor_state
pub const CURSOR_DEFAULT: c_int = 0;
pub const CURSOR_WRAPNEXT: c_int = 1;
pub const CURSOR_ORIGIN: c_int = 2;

// enum charset
pub const CS_GRAPHIC0: c_int = 0;
pub const CS_GRAPHIC1: c_int = 1;
pub const CS_UK: c_int = 2;
pub const CS_USA: c_int = 3;
pub const CS_MULTI: c_int = 4;
pub const CS_GER: c_int = 5;
pub const CS_FI: c_int = 6;

#[macro_export]
macro_rules! die {
    ($($t:tt)+) => {
        eprintln!($($t)+);
        std::process::exit(1);
    }
}

#[inline]
pub fn between<T>(x: T, a: T, b: T) -> bool
where
    T: PartialOrd,
{
    (a..=b).contains(&x)
}

/// Return the length of a raw slice without using slice::len, which requires a
/// & reference.
#[inline]
pub(crate) fn len<T>(arr: *const [T]) -> usize {
    unsafe { size_of_val(&*arr) / size_of::<T>() }
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

/// Resize the terminal to `col` x `row`.
pub fn tresize(col: c_int, row: c_int) {
    unsafe {
        let minrow = row.min(term.row);
        let mincol = col.min(term.col);

        if col < 1 || row < 1 {
            eprintln!("tresize: error resizing to {col}x{row}");
            return;
        }

        // NOTE(st) slide screen to keep cursor where we expect it - tscrollup
        // would work here, but we can optimize to memmove because we're freeing
        // the earlier lines
        let mut i = 0;
        for i_ in 0..=term.c.y - row {
            libc::free(term.line.offset(i_ as isize).cast());
            libc::free(term.alt.offset(i_ as isize).cast());
            i = i_;
        }

        // ensure that both src and dst are not NULL
        if i > 0 {
            libc::memmove(
                term.line.cast(),
                term.line.offset(i as isize).cast(),
                row as usize * size_of::<Line>(),
            );
            libc::memmove(
                term.alt.cast(),
                term.alt.offset(i as isize).cast(),
                row as usize * size_of::<Line>(),
            );
        }

        for i_ in i + row..term.row {
            libc::free(term.line.offset(i_ as isize).cast());
            libc::free(term.alt.offset(i_ as isize).cast());
        }

        // resize to new height
        term.line =
            xrealloc(term.line.cast(), row as usize * size_of::<Line>()).cast();
        term.alt =
            xrealloc(term.alt.cast(), row as usize * size_of::<Line>()).cast();
        term.dirty = xrealloc(
            term.dirty.cast(),
            row as usize * size_of_val(&*term.dirty),
        )
        .cast();
        term.tabs =
            xrealloc(term.tabs.cast(), col as usize * size_of_val(&*term.tabs))
                .cast();

        // resize each row to new width, zero-pad if needed
        for i_ in 0..minrow {
            i = i_;
            *term.line.offset(i as isize) = xrealloc(
                term.line.offset(i as isize).cast(),
                col as usize * size_of::<Glyph_>(),
            )
            .cast();
            *term.alt.offset(i as isize) = xrealloc(
                term.alt.offset(i as isize).cast(),
                col as usize * size_of::<Glyph_>(),
            )
            .cast();
        }

        // allocate any new rows
        for i_ in minrow..row {
            i = i_;
            *term.line.offset(i as isize) =
                xmalloc(col as usize * size_of::<Glyph_>()).cast();
            *term.alt.offset(i as isize) =
                xmalloc(col as usize * size_of::<Glyph_>()).cast();
        }

        if col > term.col {
            let mut bp = term.tabs.offset(term.col as isize);
            memset(
                bp.cast(),
                0,
                size_of_val(&*term.tabs) * (col - term.col) as usize,
            );

            // looping backwards over term.tabs, trimming zero values. emulating
            // this C code:
            //
            // while (--bp > term.tabs && !*bp)
            while !std::ptr::addr_eq(bp, term.tabs) && *bp == 0 {
                bp = bp.offset(-1);
            }

            // now looping forwards again, emulating this C code:
            //
            // for (bp += tabspaces; bp < term.tabs + col; bp += tabspaces)
            //     *bp = 1;
            bp = bp.offset(tabspaces as isize);
            while !std::ptr::addr_eq(bp, term.tabs.offset(col as isize)) {
                *bp = 1;
                bp = bp.offset(tabspaces as isize);
            }
        }

        // update terminal size
        term.col = col;
        term.row = row;
        // reset scrolling region
        tsetscroll(0, row - 1);
        // make use of the LIMIT in tmoveto
        tmoveto(term.c.x, term.c.y);
        // clear both screens (it makes dirty all lines)
        let c = term.c;
        for _ in 0..2 {
            if mincol < col && 0 < minrow {
                tclearregion(mincol, 0, col - 1, minrow - 1);
            }
            if 0 < col && minrow < row {
                tclearregion(0, minrow, col - 1, row - 1);
            }
            tswapscreen();
            tcursor(CURSOR_LOAD);
        }
        term.c = c;
    }
}

fn tsetscroll(t: c_int, b: c_int) {
    unsafe {
        let mut t = t.clamp(0, term.row - 1);
        let mut b = b.clamp(0, term.row - 1);
        if t > b {
            (t, b) = (b, t);
        }
        term.top = t;
        term.bot = b;
    }
}

/// Call `malloc`, dying on any errors to avoid returning NULL.
fn xmalloc(len: usize) -> *mut c_void {
    unsafe {
        let p = libc::malloc(len);

        if p.is_null() {
            let s = CStr::from_ptr(libc::strerror(*libc::__errno_location()));
            die!("malloc: {}", s.to_str().unwrap_or("Unknown error"));
        }

        p
    }
}

/// Call `realloc`, dying on any errors to avoid returning NULL.
fn xrealloc(p: *mut c_void, len: usize) -> *mut c_void {
    unsafe {
        let p = libc::realloc(p, len);

        if p.is_null() {
            let s = CStr::from_ptr(libc::strerror(*libc::__errno_location()));
            die!("realloc: {}", s.to_str().unwrap_or("Unknown error"));
        }

        p
    }
}

pub fn treset() {
    unsafe {
        term.c = TCursor {
            attr: Glyph_ {
                u: 0,
                mode: ATTR_NULL as u16,
                fg: defaultfg,
                bg: defaultbg,
            },
            x: 0,
            y: 0,
            state: CURSOR_DEFAULT as i8,
        };
        libc::memset(
            term.tabs.cast(),
            0,
            term.col as usize * size_of_val(&*term.tabs),
        );

        let mut i = tabspaces as i32;
        while i < term.col {
            *term.tabs.offset(i as isize) = 1;
            i += tabspaces as i32;
        }

        term.top = 0;
        term.bot = term.row - 1;
        term.mode = MODE_WRAP | MODE_UTF8;

        libc::memset(
            &raw mut term.trantbl as *mut _,
            CS_USA,
            // TODO this is supposed to be sizeof(term.trantbl), but I can't
            // figure out how to call size_of_val without creating a reference
            // to a mutable static. I thought it was 4 (term.trantbl.len()) *
            // the size of a c_char, but after a segfault and checking sizeof on
            // a similar array in C, it's just the length of the array
            4,
        );

        term.charset = 0;

        for _ in 0..2 {
            tmoveto(0, 0);
            tcursor(CURSOR_SAVE);
            tclearregion(0, 0, term.col - 1, term.row - 1);
            tswapscreen();
        }
    }
}

/// Move the cursor to `x, y`, clamping to the dimensions of the window.
pub fn tmoveto(x: c_int, y: c_int) {
    unsafe {
        let (miny, maxy) = if term.c.state & CURSOR_ORIGIN as i8 != 0 {
            (term.top, term.bot)
        } else {
            (0, term.row - 1)
        };
        term.c.state &= !CURSOR_WRAPNEXT as i8;
        term.c.x = x.clamp(0, term.col - 1);
        term.c.y = y.clamp(miny, maxy);
    }
}

#[inline]
fn is_set(flag: c_int) -> bool {
    unsafe { term.mode & flag != 0 }
}

/// Load or save cursor state depending on the value of `mode`, which should be
/// either `CURSOR_SAVE` or `CURSOR_LOAD`.
pub fn tcursor(mode: c_int) {
    static mut C: [TCursor; 2] = [TCursor {
        attr: Glyph_ { u: 0, mode: 0, fg: 0, bg: 0 },
        x: 0,
        y: 0,
        state: 0,
    }; 2];
    let alt = is_set(MODE_ALTSCREEN) as usize;
    unsafe {
        if mode == CURSOR_SAVE {
            C[alt] = term.c;
        } else if mode == CURSOR_LOAD {
            term.c = C[alt];
            tmoveto(C[alt].x, C[alt].y);
        }
    }
}

/// Clear the region defined by the provided coordinates, clamping to the bounds
/// of the window.
///
/// If the bounds are provided in the "wrong" order (eg x1 > x2), these are
/// swapped as well.
pub fn tclearregion(
    mut x1: c_int,
    mut y1: c_int,
    mut x2: c_int,
    mut y2: c_int,
) {
    unsafe {
        if x1 > x2 {
            std::mem::swap(&mut x1, &mut x2);
        }
        if y1 > y2 {
            std::mem::swap(&mut y1, &mut y2);
        }

        let x1 = x1.clamp(0, term.col - 1);
        let x2 = x2.clamp(0, term.col - 1);
        let y1 = y1.clamp(0, term.row - 1);
        let y2 = y2.clamp(0, term.row - 1);

        for y in y1..=y2 {
            *term.dirty.offset(y as isize) = 1;
            for x in x1..=x2 {
                let gp: &mut Glyph_ =
                    &mut *(*term.line.offset(y as isize)).offset(x as isize);
                if selected(x, y) != 0 {
                    selclear();
                }
                gp.fg = term.c.attr.fg;
                gp.bg = term.c.attr.bg;
                gp.mode = 0;
                gp.u = ' ' as u32;
            }
        }
    }
}

/// Return whether or not the character at `x`,`y` is selected.
///
/// TODO bool
fn selected(x: c_int, y: c_int) -> c_int {
    unsafe {
        if sel.mode == SEL_EMPTY
            || sel.ob.x == -1
            || sel.alt != is_set(MODE_ALTSCREEN) as c_int
        {
            return 0;
        }
        if sel.type_ == SEL_RECTANGULAR {
            return (between(y, sel.nb.y, sel.ne.y)
                && between(x, sel.nb.y, sel.ne.x)) as c_int;
        }

        (between(y, sel.nb.y, sel.ne.y)
            && (y != sel.nb.y || x >= sel.nb.x)
            && (y != sel.ne.y || x <= sel.ne.x)) as c_int
    }
}

/// Clear the current selection.
fn selclear() {
    unsafe {
        if sel.ob.x == -1 {
            return;
        }
        sel.mode = SEL_IDLE;
        sel.ob.x = -1;
        tsetdirt(sel.nb.y, sel.ne.y);
    }
}

/// Mark the rows between `top` and `bot` dirty.
fn tsetdirt(top: c_int, bot: c_int) {
    unsafe {
        let top = top.clamp(0, term.row - 1);
        let bot = bot.clamp(0, term.row - 1);

        for i in top..=bot {
            // TODO term.dirty is obviously a dynamic array of bool with size
            // equal to the number of rows in the terminal. thus, refactoring
            // can happen in two steps: first, make *mut bool, then make it a
            // vec that can be resized instead of xrealloc in C
            *term.dirty.offset(i as isize) = 1;
        }
    }
}

/// Swap the current and alt screens and mark the whole terminal dirty;
pub fn tswapscreen() {
    unsafe {
        (term.line, term.alt) = (term.alt, term.line);
        term.mode ^= MODE_ALTSCREEN;
        tfulldirt();
    }
}

/// Mark the whole terminal dirty.
fn tfulldirt() {
    unsafe {
        tsetdirt(0, term.row - 1);
    }
}

pub fn xinit(cols: c_int, rows: c_int) {
    unsafe {
        xw.dpy = bindgen::XOpenDisplay(null());
        if xw.dpy.is_null() {
            die!("can't open display");
        }
        xw.scr = bindgen::XDefaultScreen(xw.dpy);
        xw.vis = bindgen::XDefaultVisual(xw.dpy, xw.scr);

        // font
        if FcInit() == 0 {
            die!("could not init fontconfig");
        }

        usedfont = if opt_font.is_null() {
            font.as_ptr() as *const i8
        } else {
            opt_font
        };
        x::xloadfonts(usedfont, 0.0);

        // colors
        xw.cmap = bindgen::XDefaultColormap(xw.dpy, xw.scr);
        x::xloadcols();

        // adjust fixed window geometry
        win.w = 2 * borderpx + cols * win.cw;
        win.h = 2 * borderpx + rows * win.ch;
        if xw.gm & bindgen::XNegative as i32 != 0 {
            xw.l += bindgen::XDisplayWidth(xw.dpy, xw.scr) - win.w - 2;
        }
        if xw.gm & bindgen::YNegative as i32 != 0 {
            xw.t += bindgen::XDisplayHeight(xw.dpy, xw.scr) - win.h - 2;
        }

        // Events
        xw.attrs.background_pixel = (*dc.col.add(defaultbg as usize)).pixel;
        xw.attrs.border_pixel = (*dc.col.add(defaultbg as usize)).pixel;
        xw.attrs.bit_gravity = bindgen::NorthWestGravity as i32;
        xw.attrs.event_mask = (bindgen::FocusChangeMask
            | bindgen::KeyPressMask
            | bindgen::KeyReleaseMask
            | bindgen::ExposureMask
            | bindgen::VisibilityChangeMask
            | bindgen::StructureNotifyMask
            | bindgen::ButtonMotionMask
            | bindgen::ButtonPressMask
            | bindgen::ButtonReleaseMask) as i64;
        xw.attrs.colormap = xw.cmap;

        let root = bindgen::XRootWindow(xw.dpy, xw.scr);
        let mut parent;
        if !opt_embed.is_null() {
            parent = strtol(opt_embed, null_mut(), 0);
            if parent == 0 {
                parent = root as i64;
            }
        } else {
            parent = root as i64;
        }
        xw.win = bindgen::XCreateWindow(
            xw.dpy,
            root,
            xw.l,
            xw.t,
            win.w as u32,
            win.h as u32,
            0,
            bindgen::XDefaultDepth(xw.dpy, xw.scr),
            bindgen::InputOutput,
            xw.vis,
            (bindgen::CWBackPixel
                | bindgen::CWBorderPixel
                | bindgen::CWBitGravity
                | bindgen::CWEventMask
                | bindgen::CWColormap) as u64,
            &raw mut xw.attrs,
        );

        if parent != root as i64 {
            bindgen::XReparentWindow(xw.dpy, xw.win, parent as u64, xw.l, xw.t);
        }

        let mut gcvalues = XGCValues {
            function: 0,
            plane_mask: 0,
            foreground: 0,
            background: 0,
            line_width: 0,
            line_style: 0,
            cap_style: 0,
            join_style: 0,
            fill_style: 0,
            fill_rule: 0,
            arc_mode: 0,
            tile: 0,
            stipple: 0,
            ts_x_origin: 0,
            ts_y_origin: 0,
            font: 0,
            subwindow_mode: 0,
            graphics_exposures: 0,
            clip_x_origin: 0,
            clip_y_origin: 0,
            clip_mask: 0,
            dash_offset: 0,
            dashes: 0,
        };
        gcvalues.graphics_exposures = False;
        dc.gc = bindgen::XCreateGC(
            xw.dpy,
            xw.win,
            GCGraphicsExposures as u64,
            &mut gcvalues,
        );
        xw.buf = bindgen::XCreatePixmap(
            xw.dpy,
            xw.win,
            win.w as u32,
            win.h as u32,
            bindgen::XDefaultDepth(xw.dpy, xw.scr) as u32,
        );

        bindgen::XSetForeground(
            xw.dpy,
            dc.gc,
            (*dc.col.add(defaultbg as usize)).pixel,
        );
        bindgen::XFillRectangle(
            xw.dpy,
            xw.buf,
            dc.gc,
            0,
            0,
            win.w as u32,
            win.h as u32,
        );

        // font spec buffer
        xw.specbuf = xmalloc(cols as usize * size_of::<GlyphFontSpec>()).cast();

        // Xft rendering context
        xw.draw = bindgen::XftDrawCreate(xw.dpy, xw.buf, xw.vis, xw.cmap);

        // input methods
        if x::ximopen(xw.dpy) == 0 {
            bindgen::XRegisterIMInstantiateCallback(
                xw.dpy,
                null_mut(),
                null_mut(),
                null_mut(),
                Some(x::ximinstantiate),
                null_mut(),
            );
        }

        // white cursor, black outline
        let cursor = bindgen::XCreateFontCursor(xw.dpy, mouseshape);
        bindgen::XDefineCursor(xw.dpy, xw.win, cursor);

        let mut xmousefg = bindgen::XColor {
            pixel: 0,
            red: 0,
            green: 0,
            blue: 0,
            flags: 0,
            pad: 0,
        };
        let mut xmousebg = bindgen::XColor {
            pixel: 0,
            red: 0,
            green: 0,
            blue: 0,
            flags: 0,
            pad: 0,
        };
        if bindgen::XParseColor(
            xw.dpy,
            xw.cmap,
            colorname[mousefg as usize],
            &mut xmousefg,
        ) == 0
        {
            xmousefg.red = 0xffff;
            xmousefg.green = 0xffff;
            xmousefg.blue = 0xffff;
        }
        if bindgen::XParseColor(
            xw.dpy,
            xw.cmap,
            colorname[mousebg as usize],
            &mut xmousebg,
        ) == 0
        {
            xmousebg.red = 0x0000;
            xmousebg.green = 0x0000;
            xmousebg.blue = 0x0000;
        }

        bindgen::XRecolorCursor(xw.dpy, cursor, &mut xmousefg, &mut xmousebg);

        xw.xembed = bindgen::XInternAtom(xw.dpy, c"_XEMBED".as_ptr(), False);
        xw.wmdeletewin =
            bindgen::XInternAtom(xw.dpy, c"WM_DELETE_WINDOW".as_ptr(), False);
        xw.netwmname =
            bindgen::XInternAtom(xw.dpy, c"_NET_WM_NAME".as_ptr(), False);
        xw.netwmiconname =
            bindgen::XInternAtom(xw.dpy, c"_NET_WM_ICON_NAME".as_ptr(), False);
        bindgen::XSetWMProtocols(xw.dpy, xw.win, &raw mut xw.wmdeletewin, 1);

        xw.netwmpid =
            bindgen::XInternAtom(xw.dpy, c"_NET_WM_PID".as_ptr(), False);
        let thispid = getpid();
        bindgen::XChangeProperty(
            xw.dpy,
            xw.win,
            xw.netwmpid,
            XA_CARDINAL,
            32,
            PropModeReplace,
            &raw const thispid as *const c_uchar,
            1,
        );

        win.mode = MODE_NUMLOCK;
        resettitle();
        x::xhints();
        bindgen::XMapWindow(xw.dpy, xw.win);
        bindgen::XSync(xw.dpy, False);

        bindgen::clock_gettime(CLOCK_MONOTONIC, &raw mut xsel.tclick1);
        bindgen::clock_gettime(CLOCK_MONOTONIC, &raw mut xsel.tclick2);
        xsel.primary = null_mut();
        xsel.clipboard = null_mut();
        xsel.xtarget = bindgen::XInternAtom(xw.dpy, c"UTF8_STRING".as_ptr(), 0);
        if xsel.xtarget == bindgen::None as u64 {
            xsel.xtarget = XA_STRING;
        }
    }
}

/// Initialize the global selection in `sel`.
pub fn selinit() {
    unsafe {
        sel.mode = SEL_IDLE;
        sel.snap = 0;
        sel.ob.x = -1;
    }
}

pub fn resettitle() {
    x::xsettitle(null_mut());
}

// DUMMY
pub fn run() {
    unsafe { bindgen::run() }
}
