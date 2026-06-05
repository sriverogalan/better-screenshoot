use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Emitter};

static ACTIVE: AtomicBool = AtomicBool::new(false);
static HUB_SHOW_EPOCH: AtomicU64 = AtomicU64::new(0);
static SESSION_STARTED_AT_MS: AtomicU64 = AtomicU64::new(0);
/// Contador monotónico para asignar un token único a cada sesión real.
static GEN_COUNTER: AtomicU64 = AtomicU64::new(0);
/// Token de la sesión activa (0 = sin sesión).
static CURRENT_GEN: AtomicU64 = AtomicU64::new(0);

/// Sesiones activas más allá de este umbral se consideran obsoletas (zombie).
const STALE_SESSION_MS: u64 = 8_000;

pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub fn is_active() -> bool {
    ACTIVE.load(Ordering::SeqCst)
}

/// `true` solo si hay sesión activa y no ha superado el umbral de obsolescencia.
pub fn is_active_fresh() -> bool {
    if !is_active() {
        return false;
    }

    let started = SESSION_STARTED_AT_MS.load(Ordering::SeqCst);
    if started == 0 {
        return true;
    }

    let elapsed = now_ms().saturating_sub(started);
    if elapsed > STALE_SESSION_MS {
        crate::app_trace!(
            "capture_session: auto-clearing stale session after {elapsed}ms"
        );
        force_clear_session();
        return false;
    }

    true
}

fn force_clear_session() {
    ACTIVE.store(false, Ordering::SeqCst);
    SESSION_STARTED_AT_MS.store(0, Ordering::SeqCst);
    CURRENT_GEN.store(0, Ordering::SeqCst);
}

/// Token de la sesión activa (0 si no hay ninguna).
pub fn current_generation() -> u64 {
    CURRENT_GEN.load(Ordering::SeqCst)
}

pub fn current_hub_show_epoch() -> u64 {
    HUB_SHOW_EPOCH.load(Ordering::SeqCst)
}

/// Devuelve `false` si una captura está activa o si `epoch` quedó invalidado por un `begin` posterior.
pub fn should_show_hub(epoch: u64) -> bool {
    !is_active_fresh() && epoch == current_hub_show_epoch()
}

/// Inicia una sesión y devuelve su token. Si ya hay una activa, devuelve `0` (coalescida).
pub fn begin(app: &AppHandle) -> u64 {
    if ACTIVE.swap(true, Ordering::SeqCst) {
        crate::app_trace!("capture_session: begin skipped (already active)");
        return 0;
    }

    let gen = GEN_COUNTER.fetch_add(1, Ordering::SeqCst).wrapping_add(1);
    CURRENT_GEN.store(gen, Ordering::SeqCst);
    SESSION_STARTED_AT_MS.store(now_ms(), Ordering::SeqCst);
    HUB_SHOW_EPOCH.fetch_add(1, Ordering::SeqCst);
    crate::app_trace!("capture_session: begin (gen={gen})");
    let _ = app.emit("capture-session-active", ());
    gen
}

/// `true` si una tarea con token `gen` puede cerrar la sesión `current` vigente.
fn can_end_generation(gen: u64, current: u64) -> bool {
    gen != 0 && gen == current
}

/// Termina la sesión solo si `gen` sigue siendo la sesión activa.
/// Evita que una tarea obsoleta cierre una sesión de captura más reciente.
pub fn end_generation(app: &AppHandle, gen: u64) {
    if !can_end_generation(gen, CURRENT_GEN.load(Ordering::SeqCst)) {
        if gen != 0 {
            crate::app_trace!("capture_session: end_generation(gen={gen}) ignorado (sesion no vigente)");
        }
        return;
    }

    if !ACTIVE.swap(false, Ordering::SeqCst) {
        return;
    }

    CURRENT_GEN.store(0, Ordering::SeqCst);
    SESSION_STARTED_AT_MS.store(0, Ordering::SeqCst);
    crate::app_trace!("capture_session: end (gen={gen})");
    let _ = app.emit("capture-session-ended", ());
}

/// Inicia sesión si no hay una activa; al soltar el guard se llama a `end_generation`.
pub struct CaptureSessionGuard {
    app: AppHandle,
    gen: u64,
}

impl CaptureSessionGuard {
    pub fn begin(app: &AppHandle) -> Self {
        let gen = if is_active() { 0 } else { begin(app) };
        Self {
            app: app.clone(),
            gen,
        }
    }
}

impl Drop for CaptureSessionGuard {
    fn drop(&mut self) {
        end_generation(&self.app, self.gen);
    }
}

/// Termina la sesión `gen` al soltar el guard (para tareas async que heredan una sesión ya iniciada).
pub struct CaptureSessionEndGuard {
    app: AppHandle,
    gen: u64,
}

impl CaptureSessionEndGuard {
    /// Captura el token de la sesión vigente para cerrarla de forma segura más tarde.
    pub fn current(app: &AppHandle) -> Self {
        Self {
            app: app.clone(),
            gen: current_generation(),
        }
    }
}

impl Drop for CaptureSessionEndGuard {
    fn drop(&mut self) {
        end_generation(&self.app, self.gen);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        can_end_generation, current_hub_show_epoch, force_clear_session, is_active_fresh, now_ms,
        should_show_hub, ACTIVE, CURRENT_GEN, HUB_SHOW_EPOCH, SESSION_STARTED_AT_MS,
        STALE_SESSION_MS,
    };
    use std::sync::atomic::Ordering;

    fn reset_state() {
        ACTIVE.store(false, Ordering::SeqCst);
        HUB_SHOW_EPOCH.store(0, Ordering::SeqCst);
        SESSION_STARTED_AT_MS.store(0, Ordering::SeqCst);
        CURRENT_GEN.store(0, Ordering::SeqCst);
    }

    #[test]
    fn should_show_hub_when_idle_and_epoch_matches() {
        reset_state();
        let epoch = current_hub_show_epoch();
        assert!(should_show_hub(epoch));
    }

    #[test]
    fn should_show_hub_rejects_stale_epoch() {
        reset_state();
        let stale = current_hub_show_epoch();
        HUB_SHOW_EPOCH.fetch_add(1, Ordering::SeqCst);
        assert!(!should_show_hub(stale));
        assert!(should_show_hub(current_hub_show_epoch()));
    }

    #[test]
    fn should_show_hub_rejects_while_active() {
        reset_state();
        let epoch = current_hub_show_epoch();
        ACTIVE.store(true, Ordering::SeqCst);
        SESSION_STARTED_AT_MS.store(now_ms(), Ordering::SeqCst);
        assert!(!should_show_hub(epoch));
    }

    #[test]
    fn is_active_fresh_clears_stale_session() {
        reset_state();
        ACTIVE.store(true, Ordering::SeqCst);
        SESSION_STARTED_AT_MS.store(0, Ordering::SeqCst);

        assert!(is_active_fresh());

        let stale_started = now_ms().saturating_sub(STALE_SESSION_MS + 1);
        SESSION_STARTED_AT_MS.store(stale_started, Ordering::SeqCst);

        assert!(!is_active_fresh());
        assert!(!ACTIVE.load(Ordering::SeqCst));
    }

    #[test]
    fn force_clear_session_resets_state() {
        reset_state();
        ACTIVE.store(true, Ordering::SeqCst);
        SESSION_STARTED_AT_MS.store(42, Ordering::SeqCst);
        CURRENT_GEN.store(7, Ordering::SeqCst);
        force_clear_session();
        assert!(!ACTIVE.load(Ordering::SeqCst));
        assert_eq!(SESSION_STARTED_AT_MS.load(Ordering::SeqCst), 0);
        assert_eq!(CURRENT_GEN.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn can_end_generation_only_for_vigent_token() {
        // El dueño de la sesión vigente sí puede cerrarla.
        assert!(can_end_generation(2, 2));
        // Un token obsoleto (de una captura anterior ya relevada) no puede.
        assert!(!can_end_generation(1, 2));
        // Una sesión coalescida (token 0) nunca cierra la sesión vigente.
        assert!(!can_end_generation(0, 2));
        // Sin sesión vigente tampoco se cierra nada.
        assert!(!can_end_generation(0, 0));
    }
}
