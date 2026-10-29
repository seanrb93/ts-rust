//! On Windows the program's icon (the game's "TS" logo, `assets/ts-rust.ico`, made by `tools/render-icon.py`) is put into the executable.

fn main() {
    println!("cargo:rerun-if-changed=assets/ts-rust.ico");
    println!("cargo:rerun-if-changed=assets/ts-rust.rc");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android") {
        // the audio library (oboe) is C++: its runtime is linked into the library (nothing to load from the phone but libc)
        println!("cargo:rustc-link-arg-cdylib=-lc++_static");
        println!("cargo:rustc-link-arg-cdylib=-lc++abi");
    }
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile("assets/ts-rust.rc", embed_resource::NONE);
    }
}
