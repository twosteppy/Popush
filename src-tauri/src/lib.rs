//! Popush Tauri binary library. Built by twostep.
//!
//! This is the presentation/IPC shell. It contains **no business logic**;
//! it wires [`popush_core`]'s pure logic into Tauri commands and events and adds
//! the socket-level I/O (`russh` sessions, `git2` operations, the `notify`
//! watcher, the system keyring). Everything worth testing lives in `popush-core`
//! and is exercised there; this layer is glue.
//!
//! It links WebKitGTK and therefore only builds on the Linux target. See
//! `docs/DECISIONS.md` for why the workspace is split this way.

pub mod adapters;
pub mod commands;
pub mod git;
pub mod github;
pub mod logging;
pub mod pipeline;
pub mod ssh;
pub mod state;
pub mod wizard;

use tauri::Manager;

/// Build and run the Tauri application. Called by `main.rs`.
pub fn run() {
    // Logs are local-only and never transmitted, and every line is passed
    // through the credential redactor first so key material and tokens
    // never reach disk.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("popush=info")),
        )
        .with_writer(logging::RedactingMakeWriter)
        .init();

    // The default panic hook writes straight to raw stderr, bypassing the log
    // redactor. Route panic messages through the same redactor so a panic can
    // never print key material or a token.
    std::panic::set_hook(Box::new(|info| {
        eprintln!("{}", popush_core::redact::redact_line(&info.to_string()));
    }));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state::AppState::new())
        .setup(|app| {
            let handle = app.handle().clone();
            let state = handle.state::<state::AppState>();
            state.load_config_on_startup();

            // Set the window icon at runtime so the taskbar shows the Popush mark
            // even when running unpackaged (no installed .desktop entry).
            if let Some(window) = handle.get_webview_window("main") {
                if let Ok(icon) = tauri::image::Image::from_bytes(include_bytes!(
                    "../../packaging/desktop/icons/dev.popush.Popush-256.png"
                )) {
                    let _ = window.set_icon(icon);
                }
            }
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
            commands::add_server,
            commands::remove_server,
            commands::get_config,
            commands::config_file_path,
            commands::git_commit_and_push,
            commands::set_github_token,
            commands::clear_github_token,
            commands::github_repo_info,
            commands::command_log,
            commands::app_credit,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Popush");
}
