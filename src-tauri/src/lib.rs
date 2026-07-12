//! Popush Tauri binary library. Built by twostep.
//!
//! This is the presentation/IPC shell (D14). It contains **no business logic**;
//! it wires [`popush_core`]'s pure logic into Tauri commands and events and adds
//! the socket-level I/O (`russh` sessions, `git2` operations, the `notify`
//! watcher, the system keyring). Everything worth testing lives in `popush-core`
//! and is exercised there; this layer is glue.
//!
//! It links WebKitGTK and therefore only builds on the Linux target (D1). See
//! `docs/DECISIONS.md` for why the workspace is split this way.

pub mod adapters;
pub mod commands;
pub mod git;
pub mod ssh;
pub mod state;

use tauri::Manager;

/// Build and run the Tauri application. Called by `main.rs`.
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("popush=info")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .manage(state::AppState::new())
        .setup(|app| {
            // Load config on startup; a missing config is not an error — first
            // launch goes straight to the app (D2), showing the empty state.
            let handle = app.handle().clone();
            let state = handle.state::<state::AppState>();
            state.load_config_on_startup();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_servers,
            commands::test_connection,
            commands::list_sites,
            commands::get_site_status,
            commands::git_status,
            commands::start_deploy,
            commands::cancel_pipeline,
            commands::run_wizard_check,
            commands::apply_wizard_fix,
            commands::command_log,
            commands::app_credit,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Popush");
}
