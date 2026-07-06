use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::state::{AppSettings, HotkeyConfig, SystemCaptureMode};

const PLIST_BUDDY: &str = "/usr/libexec/PlistBuddy";
const ACTIVATE_SETTINGS: &str =
    "/System/Library/PrivateFrameworks/SystemAdministration.framework/Resources/activateSettings";
const BACKUP_FILE_NAME: &str = "system-screenshot-shortcuts-backup.json";
const BACKUP_VERSION: u32 = 1;

const PRIMARY_SYSTEM_HOTKEY_IDS: [u32; 3] = [28, 30, 184];

#[derive(Debug, Clone, Copy)]
struct ScreenshotHotkey {
    id: u32,
    label: &'static str,
    parameters: [i64; 3],
}

const SCREENSHOT_HOTKEYS: &[ScreenshotHotkey] = &[
    ScreenshotHotkey {
        id: 28,
        label: "Capture screen (⌘⇧3)",
        parameters: [51, 20, 1_179_648],
    },
    ScreenshotHotkey {
        id: 29,
        label: "Copy screen to clipboard (⌃⌘⇧3)",
        parameters: [51, 20, 1_441_792],
    },
    ScreenshotHotkey {
        id: 30,
        label: "Capture region (⌘⇧4)",
        parameters: [52, 21, 1_179_648],
    },
    ScreenshotHotkey {
        id: 31,
        label: "Copy region to clipboard (⌃⌘⇧4)",
        parameters: [52, 21, 1_441_792],
    },
    ScreenshotHotkey {
        id: 184,
        label: "Capture options (⌘⇧5)",
        parameters: [53, 23, 1_179_648],
    },
];

#[derive(Debug, Serialize, Deserialize, Clone)]
struct HotkeyBackupEntry {
    existed: bool,
    enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppHotkeysBackup {
    pub capture_area: String,
    pub capture_screen: String,
    pub capture_window: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct HotkeyBackupV1 {
    version: u32,
    created_at: String,
    system_entries: HashMap<String, HotkeyBackupEntry>,
    app_hotkeys: AppHotkeysBackup,
    previous_mode: SystemCaptureMode,
}

#[derive(Debug, Serialize, Deserialize)]
struct HotkeyBackupLegacy {
    entries: HashMap<String, HotkeyBackupEntry>,
    app_hotkeys: Option<AppHotkeysBackup>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum HotkeyBackupFile {
    V1(HotkeyBackupV1),
    Legacy(HotkeyBackupLegacy),
}

pub fn system_replacement_hotkeys() -> HotkeyConfig {
    HotkeyConfig {
        capture_screen: "Command+Shift+3".into(),
        capture_area: "Command+Shift+4".into(),
        capture_window: "Command+Shift+5".into(),
        open_history: "CommandOrControl+Shift+H".into(),
    }
}

pub fn default_app_hotkeys() -> HotkeyConfig {
    HotkeyConfig::default()
}

#[derive(Debug, Serialize, Clone)]
pub struct SystemScreenshotShortcut {
    pub id: u32,
    pub label: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct SystemCaptureStatus {
    pub platform_supported: bool,
    pub mode: SystemCaptureMode,
    pub effective_mode: SystemCaptureMode,
    pub drift_detected: bool,
    pub can_restore: bool,
    pub system_shortcuts: Vec<SystemScreenshotShortcut>,
    pub app_hotkeys: HotkeyConfig,
    #[serde(rename = "messageCode", skip_serializing_if = "Option::is_none")]
    pub message_code: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SystemCaptureModeResult {
    #[serde(rename = "messageCode")]
    pub message_code: String,
    pub status: SystemCaptureStatus,
    pub settings: AppSettings,
}

fn symbolic_hotkeys_plist() -> Result<PathBuf, String> {
    dirs::home_dir()
        .map(|home| home.join("Library/Preferences/com.apple.symbolichotkeys.plist"))
        .ok_or_else(|| crate::errors::app_error("homeDirectoryNotFound"))
}

fn run_plist_buddy(plist: &Path, command: &str) -> Result<String, String> {
    let output = Command::new(PLIST_BUDDY)
        .arg("-c")
        .arg(command)
        .arg(plist)
        .output()
        .map_err(|_| crate::errors::app_error("plistBuddyFailed"))?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(if stderr.is_empty() {
        format!("PlistBuddy failed: {command}")
    } else {
        stderr
    })
}

fn hotkey_exists(plist: &Path, id: u32) -> bool {
    run_plist_buddy(plist, &format!("Print :AppleSymbolicHotKeys:{id}")).is_ok()
}

fn read_hotkey_enabled(plist: &Path, id: u32) -> Result<bool, String> {
    if !hotkey_exists(plist, id) {
        return Ok(true);
    }

    let enabled = run_plist_buddy(plist, &format!("Print :AppleSymbolicHotKeys:{id}:enabled"))?;
    Ok(matches!(enabled.as_str(), "true" | "1" | "yes"))
}

fn set_hotkey_enabled(plist: &Path, id: u32, enabled: bool) -> Result<(), String> {
    let value = if enabled { "true" } else { "false" };
    let command = format!("Set :AppleSymbolicHotKeys:{id}:enabled {value}");
    run_plist_buddy(plist, &command).map(|_| ())
}

fn create_hotkey_entry(id: u32, parameters: [i64; 3], enabled: bool) -> Result<(), String> {
    let enabled_xml = if enabled { "true" } else { "false" };
    let xml = format!(
        "<dict><key>enabled</key><{enabled_xml}/><key>value</key><dict><key>type</key><string>standard</string><key>parameters</key><array><integer>{}</integer><integer>{}</integer><integer>{}</integer></array></dict></dict>",
        parameters[0], parameters[1], parameters[2]
    );

    let status = Command::new("defaults")
        .arg("write")
        .arg("com.apple.symbolichotkeys")
        .arg("AppleSymbolicHotKeys")
        .arg("-dict-add")
        .arg(id.to_string())
        .arg(&xml)
        .status()
        .map_err(|_| crate::errors::app_error("systemShortcutsWriteFailed"))?;

    if status.success() {
        Ok(())
    } else {
        Err(crate::errors::app_error("systemShortcutsModifyFailed"))
    }
}

fn create_disabled_hotkey(id: u32, parameters: [i64; 3]) -> Result<(), String> {
    create_hotkey_entry(id, parameters, false)
}

fn delete_hotkey(plist: &Path, id: u32) -> Result<(), String> {
    run_plist_buddy(plist, &format!("Delete :AppleSymbolicHotKeys:{id}")).map(|_| ())
}

fn apply_hotkey_changes() -> Result<(), String> {
    let status = Command::new(ACTIVATE_SETTINGS)
        .arg("-u")
        .status()
        .map_err(|_| crate::errors::app_error("systemChangesFailed"))?;

    if status.success() {
        Ok(())
    } else {
        Err("macOS did not apply shortcut changes.".into())
    }
}

fn backup_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(BACKUP_FILE_NAME)
}

fn read_backup_file(app_data_dir: &Path) -> Result<Option<HotkeyBackupV1>, String> {
    let path = backup_path(app_data_dir);
    if !path.exists() {
        return Ok(None);
    }

    let contents = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
    let parsed: HotkeyBackupFile =
        serde_json::from_str(&contents).map_err(|error| error.to_string())?;

    Ok(Some(match parsed {
        HotkeyBackupFile::V1(backup) => backup,
        HotkeyBackupFile::Legacy(legacy) => HotkeyBackupV1 {
            version: BACKUP_VERSION,
            created_at: Utc::now().to_rfc3339(),
            system_entries: legacy.entries,
            app_hotkeys: legacy.app_hotkeys.unwrap_or_else(|| AppHotkeysBackup {
                capture_area: default_app_hotkeys().capture_area,
                capture_screen: default_app_hotkeys().capture_screen,
                capture_window: default_app_hotkeys().capture_window,
            }),
            previous_mode: SystemCaptureMode::Independent,
        },
    }))
}

fn write_backup(app_data_dir: &Path, backup: &HotkeyBackupV1) -> Result<(), String> {
    std::fs::create_dir_all(app_data_dir).map_err(|error| error.to_string())?;
    let contents = serde_json::to_string_pretty(backup).map_err(|error| error.to_string())?;
    std::fs::write(backup_path(app_data_dir), contents).map_err(|error| error.to_string())
}

fn remove_backup(app_data_dir: &Path) -> Result<(), String> {
    let path = backup_path(app_data_dir);
    if path.exists() {
        std::fs::remove_file(path).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn collect_system_shortcuts(plist: &Path) -> Result<Vec<SystemScreenshotShortcut>, String> {
    let mut shortcuts = Vec::with_capacity(SCREENSHOT_HOTKEYS.len());

    for hotkey in SCREENSHOT_HOTKEYS {
        shortcuts.push(SystemScreenshotShortcut {
            id: hotkey.id,
            label: hotkey.label.to_string(),
            enabled: read_hotkey_enabled(plist, hotkey.id)?,
        });
    }

    Ok(shortcuts)
}

fn primary_system_shortcuts_enabled(plist: &Path) -> Result<bool, String> {
    for id in PRIMARY_SYSTEM_HOTKEY_IDS {
        if read_hotkey_enabled(plist, id)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn system_shortcuts_need_restore(plist: &Path) -> Result<bool, String> {
    Ok(!primary_system_shortcuts_enabled(plist)?)
}

fn effective_mode_from_plist(plist: &Path, has_backup: bool) -> Result<SystemCaptureMode, String> {
    if primary_system_shortcuts_enabled(plist)? {
        return Ok(SystemCaptureMode::Independent);
    }

    if has_backup {
        return Ok(SystemCaptureMode::ReplaceSystem);
    }

    Ok(SystemCaptureMode::ReplaceSystem)
}

fn detect_drift(
    configured_mode: SystemCaptureMode,
    effective_mode: SystemCaptureMode,
    has_backup: bool,
) -> (bool, Option<String>) {
    if configured_mode == effective_mode {
        return (false, None);
    }

    let message_code = match (configured_mode, effective_mode) {
        (SystemCaptureMode::ReplaceSystem, SystemCaptureMode::Independent) => {
            Some("driftReplaceIndependent".into())
        }
        (SystemCaptureMode::Independent, SystemCaptureMode::ReplaceSystem) => {
            if has_backup {
                Some("driftIndependentReplace".into())
            } else {
                Some("driftNoBackup".into())
            }
        }
        _ => Some("driftGeneric".into()),
    };

    (true, message_code)
}

#[cfg(target_os = "macos")]
pub fn build_status(
    app_data_dir: &Path,
    settings: &AppSettings,
) -> Result<SystemCaptureStatus, String> {
    let plist = symbolic_hotkeys_plist()?;
    let backup = read_backup_file(app_data_dir)?;
    let has_backup = backup.is_some();
    let system_shortcuts = collect_system_shortcuts(&plist)?;
    let effective_mode = effective_mode_from_plist(&plist, has_backup)?;
    let (drift_detected, message_code) =
        detect_drift(settings.system_capture_mode, effective_mode, has_backup);

    Ok(SystemCaptureStatus {
        platform_supported: true,
        mode: settings.system_capture_mode,
        effective_mode,
        drift_detected,
        can_restore: has_backup || settings.system_capture_mode == SystemCaptureMode::ReplaceSystem,
        system_shortcuts,
        app_hotkeys: settings.hotkeys.clone(),
        message_code,
    })
}

#[cfg(not(target_os = "macos"))]
pub fn build_status(
    _app_data_dir: &Path,
    settings: &AppSettings,
) -> Result<SystemCaptureStatus, String> {
    Ok(SystemCaptureStatus {
        platform_supported: false,
        mode: settings.system_capture_mode,
        effective_mode: settings.system_capture_mode,
        drift_detected: false,
        can_restore: false,
        system_shortcuts: Vec::new(),
        app_hotkeys: settings.hotkeys.clone(),
        message_code: None,
    })
}

#[cfg(target_os = "macos")]
fn disable_system_shortcuts(
    app_data_dir: &Path,
    app_hotkeys: AppHotkeysBackup,
    previous_mode: SystemCaptureMode,
) -> Result<(), String> {
    let plist = symbolic_hotkeys_plist()?;
    let mut system_entries = HashMap::new();

    for hotkey in SCREENSHOT_HOTKEYS {
        let existed = hotkey_exists(&plist, hotkey.id);
        let enabled = read_hotkey_enabled(&plist, hotkey.id)?;
        system_entries.insert(
            hotkey.id.to_string(),
            HotkeyBackupEntry { existed, enabled },
        );

        if existed {
            set_hotkey_enabled(&plist, hotkey.id, false)?;
        } else {
            create_disabled_hotkey(hotkey.id, hotkey.parameters)?;
        }
    }

    let backup = HotkeyBackupV1 {
        version: BACKUP_VERSION,
        created_at: Utc::now().to_rfc3339(),
        system_entries,
        app_hotkeys,
        previous_mode,
    };

    write_backup(app_data_dir, &backup)?;
    apply_hotkey_changes()
}

#[cfg(target_os = "macos")]
fn restore_system_shortcuts_from_backup(backup: &HotkeyBackupV1) -> Result<(), String> {
    let plist = symbolic_hotkeys_plist()?;

    for hotkey in SCREENSHOT_HOTKEYS {
        let Some(entry) = backup.system_entries.get(&hotkey.id.to_string()) else {
            continue;
        };

        if entry.existed {
            set_hotkey_enabled(&plist, hotkey.id, entry.enabled)?;
        } else if hotkey_exists(&plist, hotkey.id) {
            // Entry created when disabling: return to implicit (active) state or delete it.
            if entry.enabled {
                set_hotkey_enabled(&plist, hotkey.id, true)?;
            } else {
                let _ = delete_hotkey(&plist, hotkey.id);
            }
        }
    }

    apply_hotkey_changes()
}

#[cfg(target_os = "macos")]
fn restore_system_shortcuts_to_defaults() -> Result<(), String> {
    let plist = symbolic_hotkeys_plist()?;

    for hotkey in SCREENSHOT_HOTKEYS {
        if hotkey_exists(&plist, hotkey.id) {
            set_hotkey_enabled(&plist, hotkey.id, true)?;
        } else {
            create_hotkey_entry(hotkey.id, hotkey.parameters, true)?;
        }
    }

    apply_hotkey_changes()
}

fn uses_system_replacement_hotkeys(settings: &AppSettings) -> bool {
    let replacement = system_replacement_hotkeys();
    settings.hotkeys.capture_screen == replacement.capture_screen
        && settings.hotkeys.capture_area == replacement.capture_area
        && settings.hotkeys.capture_window == replacement.capture_window
}

#[cfg(target_os = "macos")]
fn restore_to_independent(app_data_dir: &Path, settings: &mut AppSettings) -> Result<(), String> {
    let backup = read_backup_file(app_data_dir)?;
    let plist = symbolic_hotkeys_plist()?;
    let shortcuts_disabled = system_shortcuts_need_restore(&plist)?;
    let coming_from_replace = settings.system_capture_mode == SystemCaptureMode::ReplaceSystem;

    if backup.is_none() && !coming_from_replace && !shortcuts_disabled {
        settings.system_capture_mode = SystemCaptureMode::Independent;
        return Ok(());
    }

    if let Some(backup) = &backup {
        restore_system_shortcuts_from_backup(backup)?;
        settings.hotkeys.capture_area = backup.app_hotkeys.capture_area.clone();
        settings.hotkeys.capture_screen = backup.app_hotkeys.capture_screen.clone();
        settings.hotkeys.capture_window = backup.app_hotkeys.capture_window.clone();
        remove_backup(app_data_dir)?;
    } else {
        restore_system_shortcuts_to_defaults()?;
        if coming_from_replace || uses_system_replacement_hotkeys(settings) {
            let defaults = default_app_hotkeys();
            settings.hotkeys.capture_area = defaults.capture_area;
            settings.hotkeys.capture_screen = defaults.capture_screen;
            settings.hotkeys.capture_window = defaults.capture_window;
        }
    }

    settings.system_capture_mode = SystemCaptureMode::Independent;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn apply_mode(
    app_data_dir: &Path,
    settings: &mut AppSettings,
    target_mode: SystemCaptureMode,
) -> Result<SystemCaptureModeResult, String> {
    if settings.system_capture_mode == target_mode {
        if target_mode == SystemCaptureMode::Independent {
            let plist = symbolic_hotkeys_plist()?;
            if system_shortcuts_need_restore(&plist)? {
                restore_to_independent(app_data_dir, settings)?;
                let status = build_status(app_data_dir, settings)?;
                return Ok(SystemCaptureModeResult {
                    message_code: "systemCapturesRestored".into(),
                    status,
                    settings: settings.clone(),
                });
            }
        }

        let status = build_status(app_data_dir, settings)?;
        let message_code = match target_mode {
            SystemCaptureMode::Independent => "alreadyIndependent".into(),
            SystemCaptureMode::ReplaceSystem => "alreadyReplaced".into(),
        };
        return Ok(SystemCaptureModeResult {
            message_code,
            status,
            settings: settings.clone(),
        });
    }

    match target_mode {
        SystemCaptureMode::ReplaceSystem => {
            let app_hotkeys = AppHotkeysBackup {
                capture_area: settings.hotkeys.capture_area.clone(),
                capture_screen: settings.hotkeys.capture_screen.clone(),
                capture_window: settings.hotkeys.capture_window.clone(),
            };
            disable_system_shortcuts(app_data_dir, app_hotkeys, settings.system_capture_mode)?;

            let replacement = system_replacement_hotkeys();
            settings.system_capture_mode = SystemCaptureMode::ReplaceSystem;
            settings.hotkeys.capture_screen = replacement.capture_screen;
            settings.hotkeys.capture_area = replacement.capture_area;
            settings.hotkeys.capture_window = replacement.capture_window;
        }
        SystemCaptureMode::Independent => {
            restore_to_independent(app_data_dir, settings)?;
        }
    }

    let status = build_status(app_data_dir, settings)?;
    let message_code = match target_mode {
        SystemCaptureMode::Independent => "systemCapturesRestoredFull".into(),
        SystemCaptureMode::ReplaceSystem => "systemCapturesReplaced".into(),
    };

    Ok(SystemCaptureModeResult {
        message_code,
        status,
        settings: settings.clone(),
    })
}

#[cfg(not(target_os = "macos"))]
pub fn apply_mode(
    _app_data_dir: &Path,
    settings: &mut AppSettings,
    target_mode: SystemCaptureMode,
) -> Result<SystemCaptureModeResult, String> {
    if target_mode == SystemCaptureMode::ReplaceSystem {
        return Err(crate::errors::app_error("macosOnly"));
    }

    settings.system_capture_mode = SystemCaptureMode::Independent;
    let status = build_status(_app_data_dir, settings)?;
    Ok(SystemCaptureModeResult {
        message_code: "independentModeActive".into(),
        status,
        settings: settings.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screenshot_hotkeys_include_primary_shortcuts() {
        let ids: Vec<u32> = SCREENSHOT_HOTKEYS.iter().map(|hotkey| hotkey.id).collect();
        assert!(ids.contains(&28));
        assert!(ids.contains(&30));
        assert!(ids.contains(&184));
    }

    #[test]
    fn detects_drift_when_modes_differ() {
        let (drift, message) = detect_drift(
            SystemCaptureMode::ReplaceSystem,
            SystemCaptureMode::Independent,
            true,
        );
        assert!(drift);
        assert!(message.is_some());
    }

    #[test]
    fn no_drift_when_modes_match() {
        let (drift, _) = detect_drift(
            SystemCaptureMode::Independent,
            SystemCaptureMode::Independent,
            false,
        );
        assert!(!drift);
    }

    #[test]
    fn migrates_legacy_backup_format() {
        let legacy = r#"{
            "entries": {
                "28": { "existed": false, "enabled": true }
            },
            "app_hotkeys": {
                "capture_area": "CommandOrControl+Shift+X",
                "capture_screen": "CommandOrControl+Shift+Option+S",
                "capture_window": "CommandOrControl+Shift+Option+W"
            }
        }"#;

        let parsed: HotkeyBackupFile = serde_json::from_str(legacy).expect("parse backup");
        match parsed {
            HotkeyBackupFile::Legacy(legacy) => {
                assert_eq!(legacy.entries.len(), 1);
                assert!(legacy.app_hotkeys.is_some());
            }
            HotkeyBackupFile::V1(_) => panic!("expected legacy backup"),
        }
    }
}
