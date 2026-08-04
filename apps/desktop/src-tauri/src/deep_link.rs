use tauri::{AppHandle, Manager};

use crate::shortcuts::{handle_hotkey_action, show_main_window};
use crate::state::AppState;

pub fn handle_deep_link(app: &AppHandle, urls: Vec<String>) {
    let allow_external = {
        let state = app.state::<AppState>();
        state
            .settings
            .lock()
            .map(|s| s.allow_external_control)
            .unwrap_or(true)
    };

    if !allow_external {
        return;
    }

    for url in urls {
        if let Some(action) = parse_action(&url) {
            dispatch_action(app, &action);
        }
    }
}

fn parse_action(url: &str) -> Option<String> {
    let parsed = url.strip_prefix("betterscreenshoot://")?;
    Some(parsed.trim_matches('/').to_string())
}

fn dispatch_action(app: &AppHandle, action: &str) {
    match action {
        "capture-area" | "capture-screen" | "open-history" => {
            handle_hotkey_action(app, action);
        }
        "open-settings" => show_main_window(app, "/settings"),
        _ => {}
    }
}
