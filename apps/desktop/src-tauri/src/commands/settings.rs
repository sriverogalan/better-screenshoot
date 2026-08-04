use std::path::PathBuf;

use tauri::{AppHandle, State};

use crate::shortcuts::register_hotkeys;
use crate::state::{load_settings, save_settings, AppSettings, AppState};

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}

#[tauri::command]
pub async fn pick_directory(current: Option<String>) -> Result<Option<String>, String> {
    let picked = tauri::async_runtime::spawn_blocking(move || {
        let mut dialog = rfd::FileDialog::new().set_title("Choose folder");
        if let Some(path) = current.filter(|value| !value.is_empty()) {
            let path = PathBuf::from(path);
            if path.is_dir() {
                dialog = dialog.set_directory(path);
            } else if let Some(parent) = path.parent() {
                if parent.is_dir() {
                    dialog = dialog.set_directory(parent);
                }
            }
        }
        dialog.pick_folder()
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(picked.map(|path| path.to_string_lossy().to_string()))
}

#[tauri::command]
pub async fn update_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    {
        let mut current = state.settings.lock().map_err(|e| e.to_string())?;
        *current = settings;
    }
    save_settings(&app, &state)?;
    register_hotkeys(&app)?;
    Ok(state.settings.lock().map_err(|e| e.to_string())?.clone())
}

#[tauri::command]
pub async fn reload_settings(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<AppSettings, String> {
    load_settings(&app, &state)?;
    register_hotkeys(&app)?;
    Ok(state.settings.lock().map_err(|e| e.to_string())?.clone())
}
