//! The Android activity (a proof of concept): the game runs from the app's own folder, `Android/data/<package>/files/` on the
//! device (put the game's `PAK/` and `MUSIC/` there from a computer), and is played by touch (`touch.rs`) or a controller.

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: android_activity::AndroidApp) {
    // the folder the player can reach from a computer, else the app's private one
    if let Some(dir) = app.external_data_path().or_else(|| app.internal_data_path()) {
        let _ = std::fs::create_dir_all(&dir);
        let _ = crate::BASE_DIR.set(dir);
    }
    crate::window::set_android_app(app);
    crate::run();
}
