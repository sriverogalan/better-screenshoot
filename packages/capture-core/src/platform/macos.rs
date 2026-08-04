use std::ffi::c_void;
use std::thread;
use std::time::{Duration, Instant};

use dispatch2::DispatchQueue;
use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication};
use objc2_core_foundation::{CFBoolean, CFDictionary, CFNumber, CFNumberType, CFString};
use objc2_core_graphics::{
    CGRectMakeWithDictionaryRepresentation, CGWindowListCopyWindowInfo, CGWindowListOption,
};
use xcap::{Monitor, Window};

use crate::encode::encode_png;
use crate::error::{CaptureError, CaptureResult};
use crate::platform::xcap_helpers::region_to_u32;
use crate::provider::CaptureProvider;
use crate::types::{CaptureImage, DisplayInfo, Region, WindowInfo};

pub struct MacOSProvider;

/// macOS reports menu-bar extras (Wi-Fi, battery, Control Center, clock, ...) as regular
/// windows owned by these system processes. None of them are real application windows,
/// so they must never show up in the "capture a window" picker.
const SYSTEM_UI_OWNERS: &[&str] = &[
    "Window Server",
    "Dock",
    "Control Center",
    "SystemUIServer",
    "NotificationCenter",
    "Notification Center",
    "Spotlight",
    "loginwindow",
    "TextInputMenuAgent",
    "TextInputSwitcher",
    "ControlStrip",
];

/// Menu-bar icons are tiny (well under 60px in either dimension); real app windows are not.
/// Used together with `SYSTEM_UI_OWNERS` in case Apple ships a new agent name we don't know.
const MIN_CAPTURABLE_WINDOW_SIZE: u32 = 60;

/// How long we're willing to wait for macOS to switch Spaces and bring a window
/// onscreen after activating its owning app.
const SPACE_SWITCH_TIMEOUT: Duration = Duration::from_millis(1500);
const SPACE_SWITCH_POLL_INTERVAL: Duration = Duration::from_millis(50);

/// `kCGWindowIsOnscreen` flips to true as soon as the Space *transition starts*, well
/// before the slide animation finishes compositing the destination Space. Capturing
/// immediately produces a blank/transitional frame, so we wait out the animation once
/// the window is reported onscreen. macOS's default Spaces animation is ~350-400ms;
/// this is generous enough even with "Reduce Motion" off.
const SPACE_SWITCH_SETTLE_DELAY: Duration = Duration::from_millis(500);

/// A window as reported by `CGWindowListCopyWindowInfo`, independent of xcap's own
/// listing (which is hardcoded to the currently active Space only).
struct RawWindow {
    id: u32,
    pid: u32,
    app_name: String,
    title: String,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    on_screen: bool,
}

fn cf_dict_value(dict: &CFDictionary, key: &str) -> Option<*const c_void> {
    unsafe {
        let cf_key = CFString::from_str(key);
        let key_ref = cf_key.as_ref() as *const CFString;
        let value = dict.value(key_ref.cast());
        if value.is_null() { None } else { Some(value) }
    }
}

fn cf_dict_string(dict: &CFDictionary, key: &str) -> Option<String> {
    let value = cf_dict_value(dict, key)? as *const CFString;
    Some(unsafe { (*value).to_string() })
}

fn cf_dict_i32(dict: &CFDictionary, key: &str) -> Option<i32> {
    let value = cf_dict_value(dict, key)? as *const CFNumber;
    let mut out: i32 = 0;
    let ok = unsafe { (*value).value(CFNumberType::IntType, &mut out as *mut _ as *mut c_void) };
    ok.then_some(out)
}

fn cf_dict_bool(dict: &CFDictionary, key: &str) -> Option<bool> {
    let value = cf_dict_value(dict, key)? as *const CFBoolean;
    Some(unsafe { (*value).value() })
}

fn cf_dict_rect(dict: &CFDictionary) -> Option<(i32, i32, u32, u32)> {
    let bounds_value = cf_dict_value(dict, "kCGWindowBounds")? as *const CFDictionary;
    let mut rect = Default::default();
    let ok = unsafe { CGRectMakeWithDictionaryRepresentation(Some(&*bounds_value), &mut rect) };
    if !ok {
        return None;
    }
    Some((
        rect.origin.x as i32,
        rect.origin.y as i32,
        rect.size.width as u32,
        rect.size.height as u32,
    ))
}

/// Lists every window macOS knows about, regardless of which Space it lives on.
/// Unlike xcap's `Window::all()` (which always passes `OptionOnScreenOnly`), this can
/// see windows on other desktops so they aren't silently missing from the picker.
fn list_raw_windows() -> Vec<RawWindow> {
    unsafe {
        let Some(cf_array) =
            CGWindowListCopyWindowInfo(CGWindowListOption::ExcludeDesktopElements, 0)
        else {
            return Vec::new();
        };

        let count = cf_array.count();
        let mut result = Vec::with_capacity(count as usize);

        for i in 0..count {
            let dict_ref = cf_array.value_at_index(i) as *const CFDictionary;
            if dict_ref.is_null() {
                continue;
            }
            let dict = &*dict_ref;

            let (Some(id), Some(pid), Some((x, y, width, height))) = (
                cf_dict_i32(dict, "kCGWindowNumber"),
                cf_dict_i32(dict, "kCGWindowOwnerPID"),
                cf_dict_rect(dict),
            ) else {
                continue;
            };

            result.push(RawWindow {
                id: id as u32,
                pid: pid as u32,
                app_name: cf_dict_string(dict, "kCGWindowOwnerName").unwrap_or_default(),
                title: cf_dict_string(dict, "kCGWindowName").unwrap_or_default(),
                x,
                y,
                width,
                height,
                on_screen: cf_dict_bool(dict, "kCGWindowIsOnscreen").unwrap_or(false),
            });
        }

        result
    }
}

fn is_capturable_raw_window(window: &RawWindow, own_pid: u32) -> bool {
    if window.pid == own_pid {
        return false;
    }
    if window.app_name.is_empty() {
        return false;
    }
    if SYSTEM_UI_OWNERS.contains(&window.app_name.as_str()) {
        return false;
    }
    window.width >= MIN_CAPTURABLE_WINDOW_SIZE && window.height >= MIN_CAPTURABLE_WINDOW_SIZE
}

/// Brings the owning app (and whichever Space its window lives on) to the front.
/// Returns false if no running app has that pid.
/// `NSRunningApplication` activation is an AppKit call: it must run on the main thread
/// to reliably trigger the Space switch. `capture_window` runs on a background thread
/// (via `spawn_blocking`), so we hop onto the main queue and block until it's done.
fn activate_app(pid: u32) -> bool {
    let mut activated = false;
    DispatchQueue::main().exec_sync(|| {
        activated = NSRunningApplication::runningApplicationWithProcessIdentifier(pid as _)
            .map(|app| app.activateWithOptions(NSApplicationActivationOptions::empty()))
            .unwrap_or(false);
    });
    activated
}

fn wait_until_onscreen(window_id: u64) -> bool {
    let deadline = Instant::now() + SPACE_SWITCH_TIMEOUT;
    loop {
        let onscreen = list_raw_windows()
            .into_iter()
            .any(|w| w.id as u64 == window_id && w.on_screen);
        if onscreen {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        thread::sleep(SPACE_SWITCH_POLL_INTERVAL);
    }
}

impl MacOSProvider {
    pub fn new() -> Self {
        Self
    }

    fn find_monitor(id: u32) -> CaptureResult<Monitor> {
        let monitors = Monitor::all().map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;
        monitors
            .into_iter()
            .find(|m| m.id().unwrap_or(0) as u32 == id)
            .ok_or(CaptureError::DisplayNotFound(id))
    }

    fn find_window(id: u64) -> CaptureResult<Window> {
        let windows = Window::all().map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;
        windows
            .into_iter()
            .find(|w| w.id().unwrap_or(0) as u64 == id)
            .ok_or(CaptureError::WindowNotFound(id))
    }
}

impl CaptureProvider for MacOSProvider {
    fn list_displays(&self) -> CaptureResult<Vec<DisplayInfo>> {
        let monitors = Monitor::all().map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;
        if monitors.is_empty() {
            return Err(CaptureError::PermissionDenied(
                "macOS returned no displays. Enable Screen Recording for Better Screenshoot in System Settings → Privacy & Security.".into(),
            ));
        }

        Ok(monitors
            .into_iter()
            .map(|monitor| DisplayInfo {
                id: monitor.id().unwrap_or(0) as u32,
                name: monitor.friendly_name().unwrap_or_else(|_| "Display".into()),
                width: monitor.width().unwrap_or(0),
                height: monitor.height().unwrap_or(0),
                scale_factor: monitor.scale_factor().unwrap_or(1.0) as f64,
                is_primary: monitor.is_primary().unwrap_or(false),
                x: monitor.x().unwrap_or(0),
                y: monitor.y().unwrap_or(0),
            })
            .collect())
    }

    fn list_windows(&self) -> CaptureResult<Vec<WindowInfo>> {
        let own_pid = std::process::id();
        Ok(list_raw_windows()
            .into_iter()
            .filter(|w| is_capturable_raw_window(w, own_pid))
            .map(|window| WindowInfo {
                id: window.id as u64,
                title: window.title,
                app_name: window.app_name,
                width: window.width,
                height: window.height,
                x: window.x,
                y: window.y,
                on_current_space: window.on_screen,
            })
            .collect())
    }

    fn capture_display_rgba(&self, display_id: u32) -> CaptureResult<image::RgbaImage> {
        let monitor = Self::find_monitor(display_id)?;
        monitor
            .capture_image()
            .map_err(|e| CaptureError::CaptureFailed(e.to_string()))
    }

    fn capture_display(&self, display_id: u32) -> CaptureResult<CaptureImage> {
        let image = self.capture_display_rgba(display_id)?;
        encode_png(image)
    }

    fn capture_window(&self, window_id: u64) -> CaptureResult<CaptureImage> {
        // Fast path: the window is already on the active Space, xcap can see it directly.
        if let Ok(window) = Self::find_window(window_id) {
            let image = window
                .capture_image()
                .map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;
            return encode_png(image);
        }

        // Not visible on the current Space: find its owner via the raw (all-spaces)
        // listing, activate that app so macOS switches to its Space, then retry.
        let owner_pid = list_raw_windows()
            .into_iter()
            .find(|w| w.id as u64 == window_id)
            .map(|w| w.pid)
            .ok_or(CaptureError::WindowNotFound(window_id))?;

        if !activate_app(owner_pid) || !wait_until_onscreen(window_id) {
            return Err(CaptureError::WindowActivationFailed(window_id));
        }
        thread::sleep(SPACE_SWITCH_SETTLE_DELAY);

        let window = Self::find_window(window_id)?;
        let image = window
            .capture_image()
            .map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;
        encode_png(image)
    }

    fn capture_region(&self, display_id: u32, region: Region) -> CaptureResult<CaptureImage> {
        if !region.validate() {
            return Err(CaptureError::InvalidRegion {
                message: "width and height must be greater than zero".into(),
            });
        }
        let monitor = Self::find_monitor(display_id)?;
        let (x, y, width, height) = region_to_u32(region)?;
        let image = monitor
            .capture_region(x, y, width, height)
            .map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;
        encode_png(image)
    }
}
