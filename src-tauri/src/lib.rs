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
        .invoke_handler(tauri::generate_handler![commands::app_info])
        .run(tauri::generate_context!())
        .expect("error while running DevShelf");
}
