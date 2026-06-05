use std::sync::Mutex;

use serde::{Deserialize, Deserializer, Serialize};
use tauri::{AppHandle, Manager};

use capture_core::CaptureProvider;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SystemCaptureMode {
    #[default]
    Independent,
    ReplaceSystem,
}

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

#[derive(Debug, Clone, Serialize)]
pub struct AppSettings {
    pub save_directory: String,
    pub auto_copy: bool,
    pub auto_save: bool,
    pub allow_external_control: bool,
    pub system_capture_mode: SystemCaptureMode,
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
            system_capture_mode: SystemCaptureMode::Independent,
            hotkeys: HotkeyConfig::default(),
            tier: "community".into(),
        }
    }
}

impl<'de> Deserialize<'de> for AppSettings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct AppSettingsRaw {
            save_directory: String,
            #[serde(default = "default_true")]
            auto_copy: bool,
            #[serde(default = "default_true")]
            auto_save: bool,
            #[serde(default = "default_true")]
            allow_external_control: bool,
            #[serde(default)]
            system_capture_mode: Option<SystemCaptureMode>,
            #[serde(default)]
            replace_system_screenshots: bool,
            #[serde(default)]
            hotkeys: HotkeyConfig,
            #[serde(default = "default_tier")]
            tier: String,
        }

        fn default_true() -> bool {
            true
        }

        fn default_tier() -> String {
            "community".into()
        }

        let raw = AppSettingsRaw::deserialize(deserializer)?;
        let system_capture_mode = raw.system_capture_mode.unwrap_or(if raw.replace_system_screenshots {
            SystemCaptureMode::ReplaceSystem
        } else {
            SystemCaptureMode::Independent
        });

        Ok(Self {
            save_directory: raw.save_directory,
            auto_copy: raw.auto_copy,
            auto_save: raw.auto_save,
            allow_external_control: raw.allow_external_control,
            system_capture_mode,
            hotkeys: raw.hotkeys,
            tier: raw.tier,
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_legacy_replace_system_screenshots_flag() {
        let json = r#"{
            "save_directory": "/tmp",
            "auto_copy": true,
            "auto_save": true,
            "allow_external_control": true,
            "replace_system_screenshots": true,
            "hotkeys": {
                "capture_area": "CommandOrControl+Shift+X",
                "capture_screen": "CommandOrControl+Shift+Option+S",
                "capture_window": "CommandOrControl+Shift+Option+W",
                "open_history": "CommandOrControl+Shift+H"
            },
            "tier": "community"
        }"#;

        let settings: AppSettings = serde_json::from_str(json).expect("parse settings");
        assert_eq!(settings.system_capture_mode, SystemCaptureMode::ReplaceSystem);
    }
}
