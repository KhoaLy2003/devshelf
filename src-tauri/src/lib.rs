use tauri::Manager;

mod commands;
mod config;
mod db;
mod editors;
mod notes;
mod projects;
mod vault;
mod workspaces;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let folder = app.path().app_data_dir().unwrap_or_default();
            app.manage(commands::storage::StorageState::open(folder));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::storage::storage_status,
            commands::projects::projects_list,
            commands::projects::projects_get,
            commands::projects::projects_register,
            commands::projects::projects_check_path,
            commands::projects::projects_set_pinned,
            commands::projects::projects_pick_folder,
            commands::editors::editor_profiles_list,
            commands::editors::editor_profiles_save,
            commands::editors::editor_profiles_set_default,
            commands::editors::editor_open,
            commands::editors::editor_pick_executable,
            commands::editors::editor_pick_item,
            commands::workspaces::workspaces_list,
            commands::workspaces::workspaces_get,
            commands::workspaces::workspaces_create,
            commands::workspaces::workspaces_rename,
            commands::workspaces::workspaces_delete,
            commands::workspaces::workspace_items_add,
            commands::workspaces::workspace_items_update,
            commands::workspaces::workspace_items_remove,
            commands::workspaces::workspace_items_move,
            commands::workspaces::workspaces_pick_item,
            commands::workspaces::workspace_restore,
            commands::workspaces::workspace_items_relink,
            commands::config::config_files_list,
            commands::config::config_files_add,
            commands::config::config_files_relink,
            commands::config::config_files_remove,
            commands::config::config_files_open,
            commands::config::config_files_pick,
            commands::home::home_overview
        ])
        .run(tauri::generate_context!())
        .expect("error while running DevShelf");
}
