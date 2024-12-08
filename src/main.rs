use rt::bindgen::{
    cursorshape, opt_title, setlocale, xsetcursor, xw, False,
    XSetLocaleModifiers, LC_CTYPE,
};

use rt::{run, selinit, tnew, xinit, xsetenv};

// extern "C" {
//     static mut cols: c_uint;
//     static mut rows: c_uint;
// }

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
        // cols = max(cols, 1);
        // rows = max(rows, 1);
        tnew(cols as i32, rows as i32);
        xinit(cols as i32, rows as i32);
        xsetenv();
        selinit();
        run();
    }
}
