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
            commands::projects::projects_pick_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running DevShelf");
}
