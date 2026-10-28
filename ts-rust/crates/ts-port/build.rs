//! On Windows the program's icon (the game's "TS" logo, `assets/ts-rust.ico`, made by `tools/render-icon.py`) is put into the executable.

fn main() {
    println!("cargo:rerun-if-changed=assets/ts-rust.ico");
    println!("cargo:rerun-if-changed=assets/ts-rust.rc");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile("assets/ts-rust.rc", embed_resource::NONE);
    }
}
