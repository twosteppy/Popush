pub mod adapters;
pub mod commands;
pub mod git;
pub mod github;
pub mod logging;
pub mod ops;
pub mod pipeline;
pub mod ssh;
pub mod state;
pub mod wizard;

use tauri::Manager;

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("popush=info")),
        )
        .with_writer(logging::RedactingMakeWriter)
        .init();

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
            commands::site_action,
            commands::get_site_logs,
            commands::set_ssh_password,
            commands::ssh_password_saved,
            commands::git_status,
            commands::start_deploy,
            commands::cancel_pipeline,
            commands::run_wizard_check,
            commands::apply_wizard_fix,
            commands::add_server,
            commands::add_site,
            commands::import_config,
            commands::config_error,
            commands::remove_server,
            commands::remove_site,
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
