/// Diagnóstico interno silenciado; los errores visibles van por eventos Tauri (`capture-error`, etc.).
#[macro_export]
macro_rules! app_trace {
    ($($t:tt)*) => { () };
}

#[macro_export]
macro_rules! app_warn {
    ($($t:tt)*) => { () };
}
