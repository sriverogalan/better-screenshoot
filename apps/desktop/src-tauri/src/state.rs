use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use capture_core::CaptureProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub capture_area: String,
    pub capture_screen: String,
    pub capture_window: String,
    pub open_history: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            capture_area: "CommandOrControl+Shift+X".into(),
            capture_screen: "CommandOrControl+Shift+Option+S".into(),
            capture_window: "CommandOrControl+Shift+Option+W".into(),
            open_history: "CommandOrControl+Shift+H".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub save_directory: String,
    pub auto_copy: bool,
    pub auto_save: bool,
    pub allow_external_control: bool,
    #[serde(default)]
    pub replace_system_screenshots: bool,
    pub hotkeys: HotkeyConfig,
    pub tier: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        let save_directory = dirs::picture_dir()
            .map(|p| p.join("BetterScreenshoot"))
            .and_then(|p| p.to_str().map(String::from))
            .unwrap_or_else(|| ".".into());

        Self {
            save_directory,
            auto_copy: true,
            auto_save: true,
            allow_external_control: true,
            replace_system_screenshots: false,
            hotkeys: HotkeyConfig::default(),
            tier: "community".into(),
        }
    }
}

pub struct AppState {
    pub provider: Box<dyn CaptureProvider>,
    pub settings: Mutex<AppSettings>,
    /// JSON serializado de `SavedCapture` para el editor si pierde el evento.
    pub pending_capture: Mutex<Option<String>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            provider: capture_core::create_provider(),
            settings: Mutex::new(AppSettings::default()),
            pending_capture: Mutex::new(None),
        }
    }
}

pub fn settings_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("settings.json"))
}

pub fn load_settings(app: &AppHandle, state: &AppState) -> Result<(), String> {
    let path = settings_path(app)?;
    if path.exists() {
        let data = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let settings: AppSettings = serde_json::from_str(&data).map_err(|e| e.to_string())?;
        *state.settings.lock().map_err(|e| e.to_string())? = settings;
    }
    Ok(())
}

pub fn save_settings(app: &AppHandle, state: &AppState) -> Result<(), String> {
    let path = settings_path(app)?;
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    let data = serde_json::to_string_pretty(&*settings).map_err(|e| e.to_string())?;
    std::fs::write(path, data).map_err(|e| e.to_string())?;
    Ok(())
}
