pub mod storage;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct AppInfoArgs {}

#[derive(Debug, Serialize)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
}

fn build_app_info() -> AppInfo {
    AppInfo {
        name: env!("CARGO_PKG_NAME").to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

#[tauri::command]
pub fn app_info(_args: AppInfoArgs) -> AppInfo {
    build_app_info()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_info_reports_package_metadata() {
        let info = build_app_info();
        assert_eq!(info.name, "devshelf");
        assert!(!info.version.is_empty());
    }
}
