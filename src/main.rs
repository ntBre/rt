use rt::bindgen::{
    cursorshape, opt_title, setlocale, xw, False, XSetLocaleModifiers, LC_CTYPE,
};

use rt::x::xsetcursor;
use rt::{run, selinit, tnew, xinit, xsetenv};

fn main() {
    unsafe {
        xw.l = 0;
        xw.t = 0;
        xw.isfixed = False as i32;
        xsetcursor(cursorshape as i32);

        if opt_title.is_null() {
            opt_title = c"rt".as_ptr() as *mut _;
        }

        setlocale(LC_CTYPE as i32, c"".as_ptr());
        XSetLocaleModifiers(c"".as_ptr());

        let cols = 80;
        let rows = 24;

        tnew(cols, rows);
        xinit(cols, rows);
        xsetenv();
        selinit();
        run();
    }
}
