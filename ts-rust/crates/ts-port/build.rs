//! On Windows the program's icon (the game's "TS" logo, `assets/ts-rust.ico`, made by `tools/render-icon.py`) is put into the executable.

fn main() {
    build_id();
    println!("cargo:rerun-if-changed=assets/ts-rust.ico");
    println!("cargo:rerun-if-changed=assets/ts-rust.rc");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile("assets/ts-rust.rc", embed_resource::NONE);
    }
}

/// The commit the program is built from (`TS_BUILD_ID`), with `+` when the files differ from it: the machines of a network game must run the same program, and the
/// version number alone does not change between commits.  A build made without git has "unknown".
fn build_id() {
    let git = |args: &[&str]| std::process::Command::new("git").args(args).output().ok().filter(|o| o.status.success()).map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    let id = match git(&["rev-parse", "--short=12", "HEAD"]) {
        Some(commit) => {
            let changed = git(&["status", "--porcelain", "--untracked-files=no"]).map_or(false, |s| !s.is_empty());
            if changed { format!("{commit}+") } else { commit }
        }
        None => "unknown".to_string(),
    };
    println!("cargo:rustc-env=TS_BUILD_ID={id}");
    // (made again when a commit is made or checked out)
    if let Some(dir) = git(&["rev-parse", "--absolute-git-dir"]) {
        println!("cargo:rerun-if-changed={dir}/HEAD");
        println!("cargo:rerun-if-changed={dir}/index");
    }
}
