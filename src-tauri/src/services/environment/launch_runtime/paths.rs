use tauri::AppHandle;

use crate::{core::error::Result, services::file_system::StorageService};

use super::types::LaunchPaths;

pub(super) fn get_launch_paths(app: &AppHandle) -> Result<LaunchPaths> {
    let defaults = StorageService::get_storage_default_paths(app)?;
    let storage = crate::infrastructure::persistence::tauri_store::get_store_key(app, "storage");
    let storage_obj = storage.as_ref().and_then(serde_json::Value::as_object);

    let profiles = storage_obj
        .and_then(|obj| obj.get("profilesPath"))
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or(defaults.profiles);

    let cache = storage_obj
        .and_then(|obj| obj.get("cachePath"))
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or(defaults.cache);

    Ok(LaunchPaths {
        profiles_path: profiles,
        cache_path: cache,
    })
}
