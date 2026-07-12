// Popush entry point. Built by twostep.
//
// Prevents an extra console window on non-Linux targets; harmless on Linux, where
// Popush is the only supported platform (D1).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // WebKitGTK workaround for Linux (D1 target). On many Fedora/KDE setups —
    // especially with newer WebKitGTK, Wayland, or certain GPU drivers — the
    // webview's DMABUF/GPU compositing path crashes the process the instant the
    // window would appear, exiting before anything renders. Disabling the DMABUF
    // renderer and hardware compositing is the well-established fix and costs only
    // a little GPU acceleration. We set these before Tauri creates the webview,
    // and only when the user has not already chosen a value, so an override still
    // wins.
    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
        if std::env::var_os("WEBKIT_DISABLE_COMPOSITING_MODE").is_none() {
            std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        }
    }

    popush_lib::run();
}
