// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Work around WebKitGTK Wayland protocol errors on some compositors (e.g. KDE Plasma).
    // Force X11 backend for GDK to use XWayland, which is stable for WebKitGTK rendering.
    #[cfg(target_os = "linux")]
    // SAFETY: set_var is unsafe in Rust 1.83+ because it is not thread-safe.
    // We call it in main() before any threads are spawned and before Tauri
    // initialization, so there are no concurrent readers of the environment.
    unsafe {
        std::env::set_var("GDK_BACKEND", "x11");
        // Disable WebKitGTK DMA-BUF renderer — GBM buffer creation fails on newer NVIDIA GPUs,
        // causing the webview to render as a black screen.
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    tauri_app_lib::run()
}
