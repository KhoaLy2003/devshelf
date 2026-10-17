use tauri::Manager;

mod commands;
mod config;
mod db;
mod editors;
mod notes;
mod projects;
mod secrets;
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
            app.manage(commands::notes::NoteKeys(Box::new(
                notes::CredentialKeyStore,
            )));
            app.manage(vault::VaultSession::new());
            app.manage(commands::secrets::SecretClipboard(Box::new(
                secrets::SystemClipboard,
            )));
            // Locks an idle vault without depending on UI calls; same rule as the lazy check.
            let handle = app.handle().clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
                handle.state::<vault::VaultSession>().expire_if_due();
            });
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
            commands::projects::projects_removal_preview,
            commands::projects::projects_unregister,
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
            commands::home::home_overview,
            commands::notes::notes_list,
            commands::notes::notes_get,
            commands::notes::notes_create,
            commands::notes::notes_save,
            commands::notes::notes_delete,
            commands::vault::vault_status,
            commands::vault::vault_setup,
            commands::vault::vault_unlock,
            commands::vault::vault_lock,
            commands::vault::vault_touch,
            commands::vault::vault_set_auto_lock,
            commands::vault::vault_delete_preview,
            commands::vault::vault_delete,
            commands::secrets::secrets_list,
            commands::secrets::secrets_create,
            commands::secrets::secrets_update,
            commands::secrets::secrets_reveal,
            commands::secrets::secrets_copy,
            commands::secrets::secrets_delete
        ])
        .run(tauri::generate_context!())
        .expect("error while running DevShelf");
}
