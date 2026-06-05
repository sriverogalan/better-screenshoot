use std::fs;
use std::path::PathBuf;

use base64::{engine::general_purpose::STANDARD, Engine};
use capture_core::encode::{downscale_for_preview, encode_jpeg_preview};
use capture_core::types::{CaptureImage, DisplayInfo, Region, WindowInfo};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};
use tauri_plugin_clipboard_manager::ClipboardExt;
use uuid::Uuid;

use crate::state::AppState;
use crate::window_front::bring_window_to_front;

#[derive(Debug, Serialize, Clone)]
pub struct OverlayPreview {
    pub preview_path: String,
    pub width: u32,
    pub height: u32,
    pub source_width: u32,
    pub source_height: u32,
    pub display_id: u32,
    pub scale_factor: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SavedCapture {
    pub id: String,
    pub file_path: String,
    pub width: u32,
    pub height: u32,
    pub created_at: String,
    pub data_url: String,
}

fn copy_rgba_to_clipboard(
    app: &AppHandle,
    rgba: &[u8],
    width: u32,
    height: u32,
) -> Result<(), String> {
    let clipboard_image = tauri::image::Image::new(rgba, width, height);
    app.clipboard()
        .write_image(&clipboard_image)
        .map_err(|e| e.to_string())
}

fn try_copy_to_clipboard(app: &AppHandle, image: &CaptureImage) {
    if image.rgba_bytes.is_empty() {
        return;
    }

    let copy_result =
        copy_rgba_to_clipboard(app, &image.rgba_bytes, image.width, image.height);

    if let Err(error) = copy_result {
        eprintln!("clipboard copy failed: {error}");
        let _ = app.emit("capture-warning", error);
    }
}

fn cache_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().cache_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn save_png_to_dir(dir: &PathBuf, image: &CaptureImage) -> Result<(String, String), String> {
    let id = Uuid::new_v4().to_string();
    let filename = format!("capture-{}.png", &id[..8]);
    let file_path = dir.join(&filename);
    fs::write(&file_path, &image.png_bytes).map_err(|e| e.to_string())?;
    Ok((id, file_path.to_string_lossy().to_string()))
}

fn save_temp_png(app: &AppHandle, image: &CaptureImage) -> Result<(String, String), String> {
    let dir = cache_dir(app)?.join("captures");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    save_png_to_dir(&dir, image)
}

fn capture_from_png_bytes(bytes: Vec<u8>) -> Result<CaptureImage, String> {
    let img = image::load_from_memory(&bytes).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    Ok(CaptureImage {
        width: rgba.width(),
        height: rgba.height(),
        png_bytes: bytes,
        rgba_bytes: rgba.as_raw().to_vec(),
    })
}

fn save_overlay_preview_jpeg(app: &AppHandle, jpeg_bytes: &[u8]) -> Result<String, String> {
    let dir = cache_dir(app)?.join("overlay-previews");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("preview-{}.jpg", Uuid::new_v4()));
    fs::write(&path, jpeg_bytes).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

fn show_main_hub(app: &AppHandle) {
    let Some(main) = app.get_webview_window("main") else {
        return;
    };
    let main_handle = main.clone();
    tauri::async_runtime::spawn(async move {
        bring_window_to_front(&main_handle).await;
        let _ = main_handle.emit("navigate", "/history");
    });
}

fn editor_open_failed(app: &AppHandle, record: &SavedCapture) {
    eprintln!(
        "editor window failed to open for capture {}",
        &record.id[..8.min(record.id.len())]
    );
    let _ = app.emit(
        "capture-error",
        "No se pudo abrir el editor. Abre la captura desde Historial.",
    );
    show_main_hub(app);
}

async fn present_editor_window(editor: &WebviewWindow, fullscreen: bool) {
    // Resetear estado mientras la ventana sigue oculta para no mostrar la barra de título.
    #[cfg(target_os = "macos")]
    let _ = editor.set_simple_fullscreen(false);
    #[cfg(not(target_os = "macos"))]
    let _ = editor.set_fullscreen(false);

    if fullscreen {
        #[cfg(target_os = "macos")]
        let _ = editor.set_simple_fullscreen(true);
        #[cfg(not(target_os = "macos"))]
        let _ = editor.set_fullscreen(true);
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    bring_window_to_front(editor).await;
}

async fn deliver_capture_to_editor(editor: &WebviewWindow, record: &SavedCapture) {
    for delay_ms in [250_u64, 400] {
        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
        let _ = editor.emit("capture-complete", record);
    }
}

fn show_editor_with_capture(app: &AppHandle, record: &SavedCapture, fullscreen: bool) {
    let Some(editor) = app.get_webview_window("editor") else {
        editor_open_failed(app, record);
        return;
    };

    let app_bg = app.clone();
    let record_bg = record.clone();
    tauri::async_runtime::spawn(async move {
        present_editor_window(&editor, fullscreen).await;

        if !editor.is_visible().unwrap_or(false) {
            bring_window_to_front(&editor).await;
            tokio::time::sleep(tokio::time::Duration::from_millis(120)).await;
        }

        if !editor.is_visible().unwrap_or(false) {
            editor_open_failed(&app_bg, &record_bg);
            return;
        }

        deliver_capture_to_editor(&editor, &record_bg).await;
    });
}

async fn finalize_capture(
    app: &AppHandle,
    state: &State<'_, AppState>,
    image: CaptureImage,
) -> Result<SavedCapture, String> {
    let auto_copy = {
        let settings = state.settings.lock().map_err(|e| e.to_string())?;
        settings.auto_copy
    };

    // Siempre temporal hasta que el usuario guarde; descartar borra sin historial.
    let (id, file_path) = save_temp_png(app, &image)?;

    let image_width = image.width;
    let image_height = image.height;

    let record = SavedCapture {
        id: id.clone(),
        file_path: file_path.clone(),
        width: image_width,
        height: image_height,
        created_at: Utc::now().to_rfc3339(),
        data_url: String::new(),
    };

    {
        let json = serde_json::to_string(&record).map_err(|e| e.to_string())?;
        *state.pending_capture.lock().map_err(|e| e.to_string())? = Some(json);
    }

    let _ = app.emit("capture-complete", &record);
    show_editor_with_capture(app, &record, true);

    if auto_copy {
        let app_bg = app.clone();
        tauri::async_runtime::spawn(async move {
            try_copy_to_clipboard(&app_bg, &image);
        });
    }

    Ok(record)
}

#[tauri::command]
pub async fn list_displays(state: State<'_, AppState>) -> Result<Vec<DisplayInfo>, String> {
    state
        .provider
        .list_displays()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_windows(state: State<'_, AppState>) -> Result<Vec<WindowInfo>, String> {
    state
        .provider
        .list_windows()
        .map_err(|e| e.to_string())
}

pub async fn capture_overlay_preview_internal(
    app: &AppHandle,
    display_id: Option<u32>,
) -> Result<OverlayPreview, String> {
    let state = app.state::<AppState>();
    let displays = state.provider.list_displays().map_err(|e| e.to_string())?;
    let display = match display_id {
        Some(id) => displays
            .iter()
            .find(|d| d.id == id)
            .ok_or_else(|| format!("display {id} not found"))?,
        None => displays
            .iter()
            .find(|d| d.is_primary)
            .or(displays.first())
            .ok_or_else(|| "no displays found".to_string())?,
    };

    let capture_display_id = display.id;
    let app_capture = app.clone();
    let (jpeg_bytes, preview_width, preview_height, source_width, source_height) =
        tauri::async_runtime::spawn_blocking(move || -> Result<(Vec<u8>, u32, u32, u32, u32), String> {
            let state = app_capture.state::<AppState>();
            let rgba = state
                .provider
                .capture_display_rgba(capture_display_id)
                .map_err(|e| e.to_string())?;
            let source_width = rgba.width();
            let source_height = rgba.height();
            let preview = downscale_for_preview(&rgba);
            let preview_width = preview.width();
            let preview_height = preview.height();
            let jpeg_bytes = encode_jpeg_preview(&preview).map_err(|e| e.to_string())?;
            Ok((
                jpeg_bytes,
                preview_width,
                preview_height,
                source_width,
                source_height,
            ))
        })
        .await
        .map_err(|e| e.to_string())??;

    let preview_path = save_overlay_preview_jpeg(app, &jpeg_bytes)?;

    Ok(OverlayPreview {
        preview_path,
        width: preview_width,
        height: preview_height,
        source_width,
        source_height,
        display_id: display.id,
        scale_factor: display.scale_factor,
    })
}

pub async fn capture_area_interactive_internal(app: AppHandle) -> Result<SavedCapture, String> {
    #[cfg(target_os = "macos")]
    {
        let path = crate::system_capture::capture_interactive_area(&app).await?;
        let bytes = fs::read(&path).map_err(|e| e.to_string())?;
        let _ = fs::remove_file(&path);
        let state = app.state::<AppState>();
        let capture = capture_from_png_bytes(bytes)?;
        return finalize_capture(&app, &state, capture).await;
    }

    #[cfg(target_os = "linux")]
    {
        return capture_via_portal(app).await;
    }

    #[cfg(target_os = "windows")]
    {
        Err("usa el selector de región integrado en Windows".into())
    }
}

#[tauri::command]
pub async fn capture_area_interactive(
    app: AppHandle,
) -> Result<SavedCapture, String> {
    capture_area_interactive_internal(app).await
}

pub async fn capture_screen_internal(
    app: AppHandle,
    display_id: Option<u32>,
) -> Result<SavedCapture, String> {
    let state = app.state::<AppState>();
    let image = match display_id {
        Some(id) => state
            .provider
            .capture_display(id)
            .map_err(|e| e.to_string())?,
        None => state
            .provider
            .capture_primary_display()
            .map_err(|e| e.to_string())?,
    };
    finalize_capture(&app, &state, image).await
}

#[tauri::command]
pub async fn capture_screen(
    app: AppHandle,
    state: State<'_, AppState>,
    display_id: Option<u32>,
) -> Result<SavedCapture, String> {
    let image = match display_id {
        Some(id) => state
            .provider
            .capture_display(id)
            .map_err(|e| e.to_string())?,
        None => state
            .provider
            .capture_primary_display()
            .map_err(|e| e.to_string())?,
    };
    finalize_capture(&app, &state, image).await
}

#[tauri::command]
pub async fn capture_window(
    app: AppHandle,
    state: State<'_, AppState>,
    window_id: u64,
) -> Result<SavedCapture, String> {
    let image = state
        .provider
        .capture_window(window_id)
        .map_err(|e| e.to_string())?;
    finalize_capture(&app, &state, image).await
}

#[tauri::command]
pub async fn capture_region(
    app: AppHandle,
    state: State<'_, AppState>,
    display_id: u32,
    region: Region,
) -> Result<SavedCapture, String> {
    let image = state
        .provider
        .capture_region(display_id, region)
        .map_err(|e| e.to_string())?;
    finalize_capture(&app, &state, image).await
}

#[tauri::command]
pub async fn get_overlay_preview(
    app: AppHandle,
    display_id: Option<u32>,
) -> Result<OverlayPreview, String> {
    capture_overlay_preview_internal(&app, display_id).await
}

#[tauri::command]
pub async fn complete_area_capture(
    app: AppHandle,
    state: State<'_, AppState>,
    display_id: u32,
    region: Region,
) -> Result<SavedCapture, String> {
    if !region.validate() {
        return Err("invalid region".into());
    }

    if region.x < 0 || region.y < 0 {
        return Err("region coordinates must be non-negative".into());
    }

    crate::capture_prep::hide_overlay_before_capture(&app).await;
    crate::capture_prep::hide_app_windows_before_capture(&app).await;

    let app_capture = app.clone();
    let image = tauri::async_runtime::spawn_blocking(move || {
        let state = app_capture.state::<AppState>();
        state
            .provider
            .capture_region(display_id, region)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;

    finalize_capture(&app, &state, image).await
}

#[tauri::command]
pub async fn capture_via_portal(app: AppHandle) -> Result<SavedCapture, String> {
    #[cfg(not(target_os = "linux"))]
    let _ = &app;

    #[cfg(target_os = "linux")]
    {
        use ashpd::desktop::screenshot::Screenshot;

        let response = Screenshot::request()
            .interactive(true)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .response()
            .map_err(|e| e.to_string())?;

        let uri = response.uri();
        let path = uri
            .to_file_path()
            .map_err(|_| "invalid portal screenshot path".to_string())?;
        let bytes = fs::read(&path).map_err(|e| e.to_string())?;
        let _ = fs::remove_file(&path);

        let image = image::load_from_memory(&bytes)
            .map_err(|e| e.to_string())?
            .to_rgba8();

        let capture = CaptureImage {
            width: image.width(),
            height: image.height(),
            png_bytes: bytes,
            rgba_bytes: image.as_raw().to_vec(),
        };

        let state = app.state::<AppState>();
        return finalize_capture(&app, state, capture).await;
    }

    #[cfg(not(target_os = "linux"))]
    {
        Err("portal capture is only available on Linux".into())
    }
}

#[tauri::command]
pub async fn open_capture_in_editor(
    app: AppHandle,
    state: State<'_, AppState>,
    capture_id: String,
) -> Result<(), String> {
    let history_record = crate::commands::history::get_record_by_id(&app, &capture_id)?;

    if !PathBuf::from(&history_record.file_path).exists() {
        return Err("El archivo de captura ya no existe en disco".into());
    }

    let record = SavedCapture {
        id: history_record.id,
        file_path: history_record.file_path,
        width: history_record.width as u32,
        height: history_record.height as u32,
        created_at: history_record.created_at,
        data_url: String::new(),
    };

    {
        let json = serde_json::to_string(&record).map_err(|e| e.to_string())?;
        *state.pending_capture.lock().map_err(|e| e.to_string())? = Some(json);
    }

    show_editor_with_capture(&app, &record, false);
    Ok(())
}

#[tauri::command]
pub fn take_pending_capture(state: State<'_, AppState>) -> Result<Option<SavedCapture>, String> {
    let json = state
        .pending_capture
        .lock()
        .map_err(|e| e.to_string())?
        .take();
    match json {
        Some(raw) => {
            let capture: SavedCapture =
                serde_json::from_str(&raw).map_err(|e| e.to_string())?;
            Ok(Some(capture))
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub fn clear_pending_capture(state: State<'_, AppState>) -> Result<(), String> {
    *state
        .pending_capture
        .lock()
        .map_err(|e| e.to_string())? = None;
    Ok(())
}

#[tauri::command]
pub async fn read_capture_data_url(file_path: String) -> Result<String, String> {
    let bytes = fs::read(&file_path)
        .map_err(|e| format!("No se pudo leer la imagen: {e}"))?;
    if bytes.is_empty() {
        return Err("El archivo de captura está vacío".into());
    }
    Ok(format!(
        "data:image/png;base64,{}",
        STANDARD.encode(&bytes)
    ))
}

#[tauri::command]
pub async fn discard_capture(
    app: AppHandle,
    capture_id: String,
    file_path: String,
) -> Result<(), String> {
    crate::commands::history::delete_record_if_exists(&app, &capture_id)?;

    if !file_path.is_empty() {
        let path = PathBuf::from(&file_path);
        if path.exists() {
            fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
    }

    let _ = app.emit("history-changed", ());
    Ok(())
}

#[tauri::command]
pub async fn copy_image_to_clipboard(app: AppHandle, png_base64: String) -> Result<(), String> {
    let bytes = STANDARD.decode(png_base64).map_err(|e| e.to_string())?;
    let img = image::load_from_memory(&bytes).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    copy_rgba_to_clipboard(&app, rgba.as_raw(), rgba.width(), rgba.height())
}

#[tauri::command]
pub async fn save_image_to_disk(
    app: AppHandle,
    state: State<'_, AppState>,
    png_base64: String,
) -> Result<SavedCapture, String> {
    let bytes = STANDARD.decode(png_base64).map_err(|e| e.to_string())?;
    let img = image::load_from_memory(&bytes).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let capture = CaptureImage {
        width: rgba.width(),
        height: rgba.height(),
        png_bytes: bytes,
        rgba_bytes: rgba.as_raw().to_vec(),
    };
    finalize_capture(&app, &state, capture).await
}

#[tauri::command]
pub async fn save_image_with_dialog(
    app: AppHandle,
    png_base64: String,
) -> Result<Option<SavedCapture>, String> {
    let bytes = STANDARD.decode(png_base64).map_err(|e| e.to_string())?;
    let default_name = format!("capture-{}.png", &Uuid::new_v4().to_string()[..8]);

    let picked_path = tauri::async_runtime::spawn_blocking(move || {
        rfd::FileDialog::new()
            .set_title("Guardar captura")
            .add_filter("Imagen PNG", &["png"])
            .set_file_name(&default_name)
            .save_file()
    })
    .await
    .map_err(|e| e.to_string())?;

    let Some(mut path) = picked_path else {
        return Ok(None);
    };

    if path.extension().is_none() {
        path.set_extension("png");
    }

    fs::write(&path, &bytes).map_err(|e| e.to_string())?;

    let img = image::load_from_memory(&bytes).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    let file_path = path.to_string_lossy().to_string();

    crate::commands::history::insert_record(
        &app,
        &id,
        &file_path,
        rgba.width(),
        rgba.height(),
        &created_at,
    )
    .await?;

    Ok(Some(SavedCapture {
        id,
        file_path,
        width: rgba.width(),
        height: rgba.height(),
        created_at,
        data_url: String::new(),
    }))
}
