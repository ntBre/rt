use bindgen::{
    cursorshape, opt_title, run, selinit, setlocale, tnew, xinit, xsetcursor,
    xsetenv, xw, False, XSetLocaleModifiers, LC_CTYPE,
};

mod bindgen {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(improper_ctypes)]
    #![allow(clippy::upper_case_acronyms)]
    #![allow(unused)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

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
