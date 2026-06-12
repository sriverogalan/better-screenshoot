use tauri::{AppHandle, Manager, State};

use crate::shortcuts::register_hotkeys;
use crate::state::{save_settings, AppState, SystemCaptureMode};
use crate::system_shortcuts::{self, SystemCaptureModeResult, SystemCaptureStatus};

fn app_data_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_system_capture_status(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SystemCaptureStatus, String> {
    let app_data_dir = app_data_dir(&app)?;
    let settings = state.settings.lock().map_err(|error| error.to_string())?;
    system_shortcuts::build_status(&app_data_dir, &settings)
}

#[tauri::command]
pub async fn set_system_capture_mode(
    app: AppHandle,
    state: State<'_, AppState>,
    mode: SystemCaptureMode,
) -> Result<SystemCaptureModeResult, String> {
    let app_data_dir = app_data_dir(&app)?;

    let result = {
        let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
        system_shortcuts::apply_mode(&app_data_dir, &mut settings, mode)?
    };

    save_settings(&app, &state)?;
    register_hotkeys(&app)?;

    Ok(result)
}

pub fn reconcile_system_capture(app: &AppHandle, state: &AppState) -> Result<Option<String>, String> {
    let app_data_dir = app_data_dir(app)?;
    let settings = state.settings.lock().map_err(|error| error.to_string())?;
    let status = system_shortcuts::build_status(&app_data_dir, &settings)?;

    if !status.drift_detected {
        return Ok(None);
    }

    if settings.system_capture_mode == SystemCaptureMode::Independent {
        drop(settings);
        let result = {
            let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
            system_shortcuts::apply_mode(
                &app_data_dir,
                &mut settings,
                SystemCaptureMode::Independent,
            )?
        };
        save_settings(app, state)?;
        register_hotkeys(app)?;
        return Ok(Some(result.message_code));
    }

    Ok(status.message_code)
}
