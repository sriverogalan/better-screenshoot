/// Silenced internal diagnostics; user-visible errors go through Tauri events (`capture-error`, etc.).
#[macro_export]
macro_rules! app_trace {
    ($($t:tt)*) => {
        ()
    };
}

#[macro_export]
macro_rules! app_warn {
    ($($t:tt)*) => {
        ()
    };
}
