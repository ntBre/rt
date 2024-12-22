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
    defaultbg, defaultfg, sel, selection_mode_SEL_IDLE, snprintf, tabspaces,
    term, xw, Glyph_, TCursor, Term,
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
            term.trantbl.as_mut_ptr().cast(),
            CS_USA,
            size_of_val(&term.trantbl),
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

fn selected(x: c_int, y: c_int) -> c_int {
    unsafe { bindgen::selected(x, y) }
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
        std::mem::swap(&mut term.line, &mut term.alt);
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
