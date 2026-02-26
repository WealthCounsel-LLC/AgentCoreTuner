// Core modules
pub mod aws;
pub mod aws_cli;
pub mod cache;
pub mod credentials;
pub mod error;
pub mod registry;
pub mod types;

// Tauri commands
pub mod commands;
pub mod state;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(state::AppState::default())
        .invoke_handler(tauri::generate_handler![
            // Auth commands
            commands::auth::list_profiles,
            commands::auth::check_aws_cli,
            commands::auth::get_caller_identity,
            commands::auth::sso_login,
            // Agent commands
            commands::agents::list_agents,
            commands::agents::get_agent_runtime,
            commands::agents::create_agent_runtime,
            commands::agents::update_agent_runtime,
            commands::agents::delete_agent_runtime,
            commands::agents::invoke_agent,
            commands::agents::list_agent_versions,
            // Model commands
            commands::models::list_models,
            // Memory commands
            commands::memory::list_memories,
            commands::memory::create_memory,
            commands::memory::delete_memory,
            commands::memory::list_sessions,
            // Recipe commands
            commands::recipes::list_recipes,
            commands::recipes::get_recipe,
            commands::recipes::save_recipe,
            commands::recipes::delete_recipe,
            commands::recipes::recipe_history,
            // Prompt commands
            commands::prompts::list_prompts,
            commands::prompts::get_prompt,
            commands::prompts::save_prompt,
            commands::prompts::delete_prompt,
            commands::prompts::prompt_history,
            // Log commands
            commands::logs::list_log_groups,
            commands::logs::get_logs,
            // Cache commands
            commands::cache::read_prefs,
            commands::cache::write_prefs,
            // Sync/Share commands
            commands::sync::get_sync_config,
            commands::sync::set_sync_config,
            commands::sync::publish_recipe,
            commands::sync::publish_prompt,
            commands::sync::list_shared_recipes,
            commands::sync::list_shared_prompts,
            commands::sync::import_shared_recipe,
            commands::sync::import_shared_prompt,
            commands::sync::list_sync_buckets,
            commands::sync::check_for_updates,
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Open devtools in debug mode
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
