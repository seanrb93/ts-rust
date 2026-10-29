//! The Android activity (a proof of concept): the game runs from a folder the player can reach, `TimeSplitters/` in the device's
//! storage (needs "All files access" for the app, in Settings > Apps > TimeSplitters > Permissions), else from the app's own
//! folder `Android/data/org.tsrust.game/files/`; it is played by touch (`touch.rs`).  What the program prints goes to the
//! Android log (tag `ts-rust`: `adb logcat -s ts-rust`).

#[cfg(target_os = "android")]
fn send_stderr_to_log() {
    use std::io::{BufRead, BufReader};
    use std::os::fd::FromRawFd;
    let mut fds = [0i32; 2];
    unsafe {
        if libc::pipe(fds.as_mut_ptr()) != 0 {
            return;
        }
        libc::dup2(fds[1], 2);
        let reader = std::fs::File::from_raw_fd(fds[0]);
        std::thread::spawn(move || {
            for line in BufReader::new(reader).lines().map_while(Result::ok) {
                log::info!("{line}");
            }
        });
    }
}

/// A folder the program can write in, made if it is not there.
#[cfg(target_os = "android")]
fn usable(dir: &std::path::Path) -> bool {
    std::fs::create_dir_all(dir).is_ok() && std::fs::write(dir.join(".probe"), b"x").is_ok()
}

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: android_activity::AndroidApp) {
    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info).with_tag("ts-rust"));
    send_stderr_to_log();
    let shared = std::path::PathBuf::from("/storage/emulated/0/TimeSplitters");
    let own = app.external_data_path().or_else(|| app.internal_data_path());
    let dir = if usable(&shared) { Some(shared) } else { own };
    if let Some(dir) = dir {
        eprintln!("[android] the game's folder: {}", dir.display());
        let _ = std::fs::create_dir_all(&dir);
        let _ = crate::BASE_DIR.set(dir);
    }
    crate::window::set_android_app(app);
    crate::run();
}
