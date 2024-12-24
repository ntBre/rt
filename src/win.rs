use std::ffi::c_int;

// enum win_mode
pub const MODE_VISIBLE: c_int = 1 << 0;
pub const MODE_FOCUSED: c_int = 1 << 1;
pub const MODE_APPKEYPAD: c_int = 1 << 2;
pub const MODE_MOUSEBTN: c_int = 1 << 3;
pub const MODE_MOUSEMOTION: c_int = 1 << 4;
pub const MODE_REVERSE: c_int = 1 << 5;
pub const MODE_KBDLOCK: c_int = 1 << 6;
pub const MODE_HIDE: c_int = 1 << 7;
pub const MODE_APPCURSOR: c_int = 1 << 8;
pub const MODE_MOUSESGR: c_int = 1 << 9;
pub const MODE_8BIT: c_int = 1 << 10;
pub const MODE_BLINK: c_int = 1 << 11;
pub const MODE_FBLINK: c_int = 1 << 12;
pub const MODE_FOCUS: c_int = 1 << 13;
pub const MODE_MOUSEX10: c_int = 1 << 14;
pub const MODE_MOUSEMANY: c_int = 1 << 15;
pub const MODE_BRCKTPASTE: c_int = 1 << 16;
pub const MODE_NUMLOCK: c_int = 1 << 17;
pub const MODE_MOUSE: c_int =
    MODE_MOUSEBTN | MODE_MOUSEMOTION | MODE_MOUSEX10 | MODE_MOUSEMANY;
