use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::LazyLock;

use tokio::sync::Mutex;

use base64::{engine::general_purpose::STANDARD, Engine};
use capture_core::encode::{downscale_for_preview, encode_jpeg_preview};
use capture_core::types::{CaptureImage, DisplayInfo, Region, WindowInfo};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};
use tauri_plugin_clipboard_manager::ClipboardExt;
use uuid::Uuid;

use crate::capture_session;
use crate::errors::{app_error, AppErrorPayload};
use crate::state::AppState;
use crate::window_activation::activate_app_for_window;
use crate::window_front::{bring_editor_to_front_quiet, bring_window_to_front};

static EDITOR_PRESENT_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

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

async fn copy_png_to_clipboard_async(app: AppHandle, png_bytes: Vec<u8>) {
    let decoded =
        tauri::async_runtime::spawn_blocking(move || -> Result<(Vec<u8>, u32, u32), String> {
            let img = image::load_from_memory(&png_bytes).map_err(|e| e.to_string())?;
            let rgba = img.to_rgba8();
            let width = rgba.width();
            let height = rgba.height();
            Ok((rgba.into_raw(), width, height))
        })
        .await;

    match decoded {
        Ok(Ok((raw, width, height))) => {
            if let Err(error) = copy_rgba_to_clipboard(&app, &raw, width, height) {
                let _ = app.emit(
                    "capture-warning",
                    AppErrorPayload::new("clipboardCopyFailed").to_emit_value(),
                );
                let _ = error;
            }
        }
        Ok(Err(_message)) => {
            let _ = app.emit(
                "capture-warning",
                AppErrorPayload::new("clipboardCopyFailed").to_emit_value(),
            );
        }
        Err(_join_error) => {
            let _ = app.emit(
                "capture-warning",
                AppErrorPayload::new("clipboardCopyFailed").to_emit_value(),
            );
        }
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

fn png_dimensions(bytes: &[u8]) -> Result<(u32, u32), String> {
    image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| e.to_string())?
        .into_dimensions()
        .map_err(|e| e.to_string())
}

fn capture_from_png_bytes(bytes: Vec<u8>) -> Result<CaptureImage, String> {
    let (width, height) = png_dimensions(&bytes)?;
    Ok(CaptureImage {
        width,
        height,
        png_bytes: bytes,
        rgba_bytes: Vec::new(),
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
    if capture_session::is_active_fresh() {
        return;
    }

    let Some(main) = app.get_webview_window("main") else {
        return;
    };

    let hub_epoch = capture_session::current_hub_show_epoch();
    let main_handle = main.clone();
    tauri::async_runtime::spawn(async move {
        if !capture_session::should_show_hub(hub_epoch) {
            return;
        }
        bring_window_to_front(&main_handle).await;
        if !capture_session::should_show_hub(hub_epoch) {
            return;
        }
        let _ = main_handle.emit("navigate", "/history");
    });
}

fn notify_editor_opened(app: &AppHandle, capture_id: &str) {
    // Do not clear `pending_capture` here: if the editor misses the `capture-complete` event,
    // it must be able to recover the capture on focus. The editor clears it when loaded.
    let _ = app.emit("editor-opened", capture_id);
}

const EDITOR_CONFIRM_RETRIES: u32 = 4;
const EDITOR_CONFIRM_RETRY_MS: u64 = 60;
const EDITOR_NAVIGATE_SETTLE_MS: u64 = 120;
const HUB_MAINTAIN_MS: u64 = 200;
const HUB_MAINTAIN_MS_URGENT: u64 = 100;
const HUB_MAINTAIN_MAX_TICKS: u32 = 1200;

fn capture_surface_presentation_ok(surface_visible: bool, detached_editor_visible: bool) -> bool {
    surface_visible && !detached_editor_visible
}

fn capture_surface_state(app: &AppHandle, surface: &WebviewWindow) -> (bool, bool) {
    let surface_visible = surface.is_visible().unwrap_or(false);
    let detached_editor_visible = app
        .get_webview_window("editor")
        .map(|editor| editor.is_visible().unwrap_or(false))
        .unwrap_or(false);
    (surface_visible, detached_editor_visible)
}

async fn hide_detached_editor_window(app: &AppHandle) {
    if let Some(editor) = app.get_webview_window("editor") {
        let _ = crate::window_layout::reset_editor_fullscreen_state(&editor);
        let _ = editor.hide();
    }
}

async fn prepare_capture_surface(app: &AppHandle) -> Result<WebviewWindow, String> {
    let main = app
        .get_webview_window("main")
        .ok_or_else(|| app_error("mainWindowNotFound"))?;

    hide_detached_editor_window(app).await;
    crate::window_layout::set_main_editor_mode(true);
    let _ = main.emit("navigate", "/editor");
    tokio::time::sleep(tokio::time::Duration::from_millis(
        EDITOR_NAVIGATE_SETTLE_MS,
    ))
    .await;

    Ok(main)
}

async fn confirm_editor_presented(app: &AppHandle, surface: &WebviewWindow) -> bool {
    for attempt in 0..EDITOR_CONFIRM_RETRIES {
        hide_detached_editor_window(app).await;

        let (surface_visible, detached_editor_visible) = capture_surface_state(app, surface);

        if capture_surface_presentation_ok(surface_visible, detached_editor_visible) {
            bring_editor_to_front_quiet(surface).await;
            crate::app_trace!("confirm_editor_presented: OK en intento {}", attempt + 1);
            return true;
        }

        crate::app_trace!(
            "confirm_editor_presented: intento {} surface_visible={surface_visible} detached_editor_visible={detached_editor_visible}",
            attempt + 1
        );

        bring_editor_to_front_quiet(surface).await;

        if attempt + 1 < EDITOR_CONFIRM_RETRIES {
            tokio::time::sleep(tokio::time::Duration::from_millis(EDITOR_CONFIRM_RETRY_MS)).await;
        }
    }

    let (surface_visible, detached_editor_visible) = capture_surface_state(app, surface);
    let ok = capture_surface_presentation_ok(surface_visible, detached_editor_visible);
    crate::app_trace!(
        "confirm_editor_presented: fallo final surface_visible={surface_visible} detached_editor_visible={detached_editor_visible}"
    );
    ok
}

async fn maintain_capture_surface(app: &AppHandle) {
    let mut tick = 0u32;

    loop {
        let Some(surface) = app.get_webview_window("main") else {
            break;
        };

        if !surface.is_visible().unwrap_or(false) {
            break;
        }

        hide_detached_editor_window(app).await;

        let (surface_visible, detached_editor_visible) = capture_surface_state(app, &surface);
        let needs_urgent = !surface_visible || detached_editor_visible;
        if needs_urgent {
            crate::app_trace!(
                "maintain_capture_surface: tick={tick} surface_visible={surface_visible} detached_editor_visible={detached_editor_visible}"
            );
            bring_editor_to_front_quiet(&surface).await;
        } else if tick.is_multiple_of(5) {
            bring_editor_to_front_quiet(&surface).await;
        }

        tick += 1;
        if tick >= HUB_MAINTAIN_MAX_TICKS {
            crate::app_trace!("maintain_capture_surface: tope de seguridad alcanzado");
            break;
        }

        let delay_ms = if needs_urgent {
            HUB_MAINTAIN_MS_URGENT
        } else {
            HUB_MAINTAIN_MS
        };
        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
    }

    crate::window_layout::set_main_editor_mode(false);
}

async fn editor_open_failed(app: &AppHandle, record: &SavedCapture, tried_fullscreen: bool) {
    crate::app_trace!(
        "editor window failed to open for capture {}",
        &record.id[..8.min(record.id.len())]
    );

    crate::window_layout::set_main_editor_mode(false);
    hide_detached_editor_window(app).await;

    if tried_fullscreen {
        crate::app_trace!("editor_open_failed: retrying without fullscreen");
        let retry_opened = present_editor_with_capture(app, record, false).await;
        if retry_opened {
            return;
        }
    }

    let _ = app.emit(
        "capture-error",
        AppErrorPayload::new("openEditorHint").to_emit_value(),
    );

    show_main_hub(app);
}

async fn present_editor_window(surface: &WebviewWindow, fullscreen: bool) {
    let app = surface.app_handle();

    hide_detached_editor_window(&app).await;
    crate::window_layout::reset_editor_presentation_state(surface);
    let _ = surface.set_decorations(false);

    if let Err(_error) = crate::window_layout::move_editor_to_active_monitor(surface) {
        crate::app_trace!("move_editor_to_active_monitor failed: {error}");
    }

    if fullscreen {
        crate::app_trace!("present_editor_window: entrando en modo captura (main)");
    } else if let Err(_error) = crate::window_layout::restore_windowed_editor(surface) {
        crate::app_trace!("restore_windowed_editor failed: {error}");
    }

    let _ = surface.unminimize();
    let _ = surface.show();
    let _ = surface.set_focus();

    activate_app_for_window(&app);
    hide_detached_editor_window(&app).await;

    bring_editor_to_front_quiet(surface).await;

    if fullscreen {
        let _ = surface.set_focus();
        tokio::time::sleep(tokio::time::Duration::from_millis(80)).await;
        hide_detached_editor_window(&app).await;

        if let Err(_error) = crate::window_layout::enter_editor_capture_fullscreen(surface) {
            crate::app_trace!("enter_editor_capture_fullscreen failed: {_error}");
            if let Err(_fallback) = crate::window_layout::fill_work_area(surface) {
                crate::app_trace!("fill_work_area fallback failed: {_fallback}");
            }
        }
        bring_editor_to_front_quiet(surface).await;
        hide_detached_editor_window(&app).await;
    }

    crate::app_trace!(
        "present_editor_window: superficie de captura mostrada ({})",
        surface.label()
    );
}

async fn present_editor_with_capture(
    app: &AppHandle,
    record: &SavedCapture,
    fullscreen: bool,
) -> bool {
    crate::app_trace!(
        "present_editor_with_capture: capture_id={}, fullscreen={}, session_active={}",
        &record.id[..8.min(record.id.len())],
        fullscreen,
        capture_session::is_active()
    );

    let surface = match prepare_capture_surface(app).await {
        Ok(surface) => surface,
        Err(_error) => {
            crate::app_trace!("present_editor_with_capture: prepare_capture_surface failed");
            crate::window_layout::set_main_editor_mode(false);
            return false;
        }
    };

    // Defensive timeout: if a previous presentation hung while holding the lock,
    // do not block future captures forever.
    let _guard = match tokio::time::timeout(
        tokio::time::Duration::from_secs(8),
        EDITOR_PRESENT_LOCK.lock(),
    )
    .await
    {
        Ok(guard) => guard,
        Err(_) => {
            crate::app_trace!(
                "present_editor_with_capture: timeout waiting for EDITOR_PRESENT_LOCK"
            );
            return false;
        }
    };

    // Preload the capture while the window stays hidden: the image will be ready when shown.
    deliver_capture_to_editor(&surface, record).await;
    tokio::time::sleep(tokio::time::Duration::from_millis(40)).await;

    present_editor_window(&surface, fullscreen).await;

    let _ = surface.emit("editor-presented", ());
    let _ = surface.emit("capture-complete", record);

    let presented = confirm_editor_presented(app, &surface).await;
    crate::app_trace!("present_editor_with_capture: confirm_editor_presented={presented}");

    if presented {
        notify_editor_opened(app, &record.id);
    }

    presented
}

async fn deliver_capture_to_editor(editor: &WebviewWindow, record: &SavedCapture) {
    let _ = editor.emit("capture-complete", record);
}

fn show_editor_with_capture(app: &AppHandle, record: &SavedCapture, fullscreen: bool) {
    let app_bg = app.clone();
    let record_bg = record.clone();
    tauri::async_runtime::spawn(async move {
        let _session_end = capture_session::CaptureSessionEndGuard::current(&app_bg);
        let opened = present_editor_with_capture(&app_bg, &record_bg, fullscreen).await;
        if opened {
            maintain_capture_surface(&app_bg).await;
        } else {
            editor_open_failed(&app_bg, &record_bg, fullscreen).await;
        }
    });
}

async fn open_capture_in_editor_internal(
    app: &AppHandle,
    record: &SavedCapture,
    fullscreen: bool,
) -> Result<(), String> {
    let _session = capture_session::CaptureSessionGuard::begin(app);

    let opened = present_editor_with_capture(app, record, fullscreen).await;

    if opened {
        maintain_capture_surface(app).await;
        Ok(())
    } else {
        editor_open_failed(app, record, fullscreen).await;
        Err(app_error("openEditorPermission"))
    }
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

    let image_width = image.width;
    let image_height = image.height;
    let png_bytes_for_clipboard = if auto_copy {
        Some(image.png_bytes.clone())
    } else {
        None
    };

    let app_save = app.clone();
    let (id, file_path) =
        tauri::async_runtime::spawn_blocking(move || save_temp_png(&app_save, &image))
            .await
            .map_err(|e| e.to_string())??;

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

    if let Some(png_bytes) = png_bytes_for_clipboard {
        let app_bg = app.clone();
        tauri::async_runtime::spawn(async move {
            copy_png_to_clipboard_async(app_bg, png_bytes).await;
        });
    }

    Ok(record)
}

#[tauri::command]
pub async fn list_displays(state: State<'_, AppState>) -> Result<Vec<DisplayInfo>, String> {
    state.provider.list_displays().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_windows(state: State<'_, AppState>) -> Result<Vec<WindowInfo>, String> {
    state.provider.list_windows().map_err(|e| e.to_string())
}

pub async fn capture_overlay_preview_internal(
    app: &AppHandle,
    display_id: Option<u32>,
) -> Result<OverlayPreview, String> {
    let state = app.state::<AppState>();
    let displays = state.provider.list_displays().map_err(|e| e.to_string())?;
    let display = match display_id {
        Some(id) => displays.iter().find(|d| d.id == id).ok_or_else(|| {
            AppErrorPayload::new("displayNotFound")
                .with_detail("id", id)
                .to_invoke_error()
        })?,
        None => displays
            .iter()
            .find(|d| d.is_primary)
            .or(displays.first())
            .ok_or_else(|| app_error("noDisplaysFound"))?,
    };

    let capture_display_id = display.id;
    let app_capture = app.clone();
    let (jpeg_bytes, preview_width, preview_height, source_width, source_height) =
        tauri::async_runtime::spawn_blocking(
            move || -> Result<(Vec<u8>, u32, u32, u32, u32), String> {
                let state = app_capture.state::<AppState>();
                let rgba = state
                    .provider
                    .capture_display_rgba(capture_display_id)
                    .map_err(|e| e.to_string())?;
                let source_width = rgba.width();
                let source_height = rgba.height();
                let preview = downscale_for_preview(rgba);
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
            },
        )
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
        Err("use the built-in region selector on Windows".into())
    }
}

#[tauri::command]
pub async fn capture_area_interactive(app: AppHandle) -> Result<SavedCapture, String> {
    capture_area_interactive_internal(app).await
}

pub async fn capture_screen_internal(
    app: AppHandle,
    display_id: Option<u32>,
) -> Result<SavedCapture, String> {
    let app_capture = app.clone();
    let image = tauri::async_runtime::spawn_blocking(move || {
        let state = app_capture.state::<AppState>();
        match display_id {
            Some(id) => state
                .provider
                .capture_display(id)
                .map_err(|e| e.to_string()),
            None => state
                .provider
                .capture_primary_display()
                .map_err(|e| e.to_string()),
        }
    })
    .await
    .map_err(|e| e.to_string())??;

    let state = app.state::<AppState>();
    finalize_capture(&app, &state, image).await
}

#[tauri::command]
pub async fn capture_screen(
    app: AppHandle,
    state: State<'_, AppState>,
    display_id: Option<u32>,
) -> Result<SavedCapture, String> {
    let _ = state;
    capture_screen_internal(app, display_id).await
}

#[tauri::command]
pub async fn capture_window(
    app: AppHandle,
    state: State<'_, AppState>,
    window_id: u64,
) -> Result<SavedCapture, String> {
    let gen = capture_session::begin(&app);
    if gen == 0 {
        return Err("Ya hay una captura en curso".into());
    }
    crate::capture_prep::hide_app_windows_before_capture(&app).await;

    let app_capture = app.clone();
    let image = tauri::async_runtime::spawn_blocking(move || {
        let state = app_capture.state::<AppState>();
        state
            .provider
            .capture_window(window_id)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|error| {
        capture_session::end_generation(&app, gen);
        error
    })?;

    finalize_capture(&app, &state, image).await
}

#[tauri::command]
pub async fn capture_region(
    app: AppHandle,
    state: State<'_, AppState>,
    display_id: u32,
    region: Region,
) -> Result<SavedCapture, String> {
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

    let gen = capture_session::begin(&app);
    if gen == 0 {
        return Err("Ya hay una captura en curso".into());
    }
    crate::capture_prep::hide_overlay_before_capture(&app).await;
    crate::capture_prep::hide_app_windows_before_capture(&app).await;

    let app_capture = app.clone();
    let image = match tauri::async_runtime::spawn_blocking(move || {
        let state = app_capture.state::<AppState>();
        state
            .provider
            .capture_region(display_id, region)
            .map_err(|e| e.to_string())
    })
    .await
    {
        Ok(Ok(image)) => image,
        Ok(Err(error)) => {
            capture_session::end_generation(&app, gen);
            return Err(error);
        }
        Err(error) => {
            capture_session::end_generation(&app, gen);
            return Err(error.to_string());
        }
    };

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

        let (width, height) = png_dimensions(&bytes)?;
        let capture = CaptureImage {
            width,
            height,
            png_bytes: bytes,
            rgba_bytes: Vec::new(),
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
        return Err("Capture file no longer exists on disk".into());
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

    open_capture_in_editor_internal(&app, &record, false).await
}

fn read_pending_capture(state: &State<'_, AppState>) -> Result<Option<SavedCapture>, String> {
    let json = state
        .pending_capture
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    match json {
        Some(raw) => {
            let capture: SavedCapture = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
            Ok(Some(capture))
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub fn peek_pending_capture(state: State<'_, AppState>) -> Result<Option<SavedCapture>, String> {
    read_pending_capture(&state)
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
            let capture: SavedCapture = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
            Ok(Some(capture))
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn open_pending_capture_in_editor(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let Some(record) = read_pending_capture(&state)? else {
        return Err(app_error("noPendingCapture"));
    };

    if !PathBuf::from(&record.file_path).exists() {
        return Err("Pending capture file no longer exists on disk".into());
    }

    open_capture_in_editor_internal(&app, &record, true).await
}

#[tauri::command]
pub fn clear_pending_capture(state: State<'_, AppState>) -> Result<(), String> {
    *state.pending_capture.lock().map_err(|e| e.to_string())? = None;
    Ok(())
}

#[tauri::command]
pub async fn read_capture_data_url(file_path: String) -> Result<String, String> {
    let bytes = fs::read(&file_path).map_err(|_| app_error("readImageFailed"))?;
    if bytes.is_empty() {
        return Err("Capture file is empty".into());
    }
    Ok(format!("data:image/png;base64,{}", STANDARD.encode(&bytes)))
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
    let (width, height) = png_dimensions(&bytes)?;
    let capture = CaptureImage {
        width,
        height,
        png_bytes: bytes,
        rgba_bytes: Vec::new(),
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
            .set_title("Save capture")
            .add_filter("PNG image", &["png"])
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

#[cfg(test)]
mod tests {
    use super::capture_surface_presentation_ok;

    #[test]
    fn capture_surface_presentation_ok_requires_visible_main_and_hidden_detached_editor() {
        assert!(capture_surface_presentation_ok(true, false));
        assert!(!capture_surface_presentation_ok(false, false));
        assert!(!capture_surface_presentation_ok(true, true));
        assert!(!capture_surface_presentation_ok(false, true));
    }
}
