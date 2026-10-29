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
    // the folder that has the game's files wins; with none, the shared one when it can be used
    let has_game = |d: &std::path::Path| d.join("PAK").is_dir();
    let dir = if has_game(&shared) {
        Some(shared)
    } else if own.as_deref().is_some_and(has_game) {
        own
    } else if usable(&shared) {
        Some(shared)
    } else {
        own
    };
    if let Some(dir) = dir {
        eprintln!("[android] the game's folder: {}", dir.display());
        let _ = std::fs::create_dir_all(&dir);
        let _ = crate::BASE_DIR.set(dir);
    }
    crate::window::set_android_app(app);
    crate::run();
}

/// Hide the status bar and the navigation bar (the buttons at the bottom): the picture is the whole screen and a swipe from the edge shows
/// the bars for a moment.  Called when the app starts, comes back and gets the focus.
#[cfg(target_os = "android")]
pub fn immersive() {
    use jni::objects::JObject;
    let Some(app) = crate::window::ACTIVITY.lock().unwrap().clone() else { return };
    // (the window flag of a full screen can be set from any thread)
    app.set_window_flags(android_activity::WindowManagerFlags::FULLSCREEN, android_activity::WindowManagerFlags::empty());
    unsafe {
        let Ok(vm) = jni::JavaVM::from_raw(app.vm_as_ptr() as *mut _) else { return };
        let Ok(mut env) = vm.attach_current_thread() else { return };
        let activity = JObject::from_raw(app.activity_as_ptr() as *mut _);
        // (the system bars are hidden for good, as far as a swipe from the edge: IMMERSIVE_STICKY | LAYOUT_* | HIDE_NAVIGATION | FULLSCREEN)
        let flags: i32 = 0x1000 | 0x0400 | 0x0200 | 0x0100 | 0x0004 | 0x0002;
        let result = (|| -> jni::errors::Result<()> {
            let window = env.call_method(&activity, "getWindow", "()Landroid/view/Window;", &[])?.l()?;
            let decor = env.call_method(&window, "getDecorView", "()Landroid/view/View;", &[])?.l()?;
            env.call_method(&decor, "setSystemUiVisibility", "(I)V", &[flags.into()])?;
            Ok(())
        })();
        if let Err(e) = result {
            // (a call from another thread than the app's may be refused: the full screen flag has hidden the status bar all the same)
            eprintln!("[android] the navigation bar could not be hidden: {e}");
            let _ = env.exception_clear();
        }
    }
}
