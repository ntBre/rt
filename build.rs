use std::{
    env,
    path::{Path, PathBuf},
};

fn main() {
    println!("cargo:rerun-if-changed=st/st.c");

    println!("cargo:rerun-if-changed=st/config.h");
    println!("cargo:rerun-if-changed=st/st.h");
    println!("cargo:rerun-if-changed=wrapper.h");

    println!("cargo:rustc-link-arg=-Lst");
    println!("cargo:rustc-link-arg=-lst");
    println!("cargo:rustc-link-arg=-lfontconfig");
    println!("cargo:rustc-link-arg=-lX11");

    let st = Path::new("st").canonicalize().unwrap();
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", st.display());

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-I/usr/include/freetype2")
        .clang_arg("-I/usr/include/X11/extensions")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .blocklist_var("FP_NAN")
        .blocklist_var("FP_INFINITE")
        .blocklist_var("FP_ZERO")
        .blocklist_var("FP_SUBNORMAL")
        .blocklist_var("FP_NORMAL")
        .blocklist_var("rows")
        .blocklist_var("cols")
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
