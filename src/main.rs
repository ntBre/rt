use x11::xlib;

use rt::bindgen::{cursorshape, opt_title, xw};

use rt::x::xsetcursor;
use rt::{run, selinit, tnew, xinit, xsetenv};

fn main() {
    unsafe {
        xw.l = 0;
        xw.t = 0;
        xw.isfixed = xlib::False as i32;
        xsetcursor(cursorshape as i32);

        if opt_title.is_null() {
            opt_title = c"rt".as_ptr() as *mut _;
        }

        libc::setlocale(libc::LC_CTYPE as i32, c"".as_ptr());
        xlib::XSetLocaleModifiers(c"".as_ptr());

        let cols = 80;
        let rows = 24;

        tnew(cols, rows);
        xinit(cols, rows);
        xsetenv();
        selinit();
        run();
    }
}
