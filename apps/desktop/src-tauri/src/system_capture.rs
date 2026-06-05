use std::path::PathBuf;
use std::process::{Command, Stdio};

use tauri::{AppHandle, Manager};

#[cfg(target_os = "macos")]
pub async fn capture_interactive_area(app: &AppHandle) -> Result<PathBuf, String> {
    use uuid::Uuid;

    let dir = app
        .path()
        .cache_dir()
        .map_err(|e| e.to_string())?
        .join("system-captures");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let path = dir.join(format!("capture-{}.png", Uuid::new_v4()));
    let capture_path = path.clone();

    let status = tauri::async_runtime::spawn_blocking(move || {
        Command::new("/usr/sbin/screencapture")
            .args(["-i", "-x", "-t", "png"])
            .arg(&capture_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| format!("no se pudo ejecutar la captura del sistema: {e}"))?;

    if !status.success() {
        let _ = std::fs::remove_file(&path);
        return Err("Captura cancelada".into());
    }

    let file_size = path.metadata().map(|meta| meta.len()).unwrap_or(0);
    if file_size == 0 {
        let _ = std::fs::remove_file(&path);
        return Err("Captura cancelada".into());
    }

    Ok(path)
}

#[cfg(not(target_os = "macos"))]
pub async fn capture_interactive_area(_app: &AppHandle) -> Result<PathBuf, String> {
    Err("la captura interactiva del sistema solo está disponible en macOS".into())
}
