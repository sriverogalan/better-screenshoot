use tauri::{AppHandle, State};

use crate::shortcuts::register_hotkeys;
use crate::state::{load_settings, save_settings, AppSettings, AppState};

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
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
