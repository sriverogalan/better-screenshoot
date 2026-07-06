use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Emitter};

static ACTIVE: AtomicBool = AtomicBool::new(false);
static HUB_SHOW_EPOCH: AtomicU64 = AtomicU64::new(0);
static SESSION_STARTED_AT_MS: AtomicU64 = AtomicU64::new(0);
/// Monotonic counter to assign a unique token to each real session.
static GEN_COUNTER: AtomicU64 = AtomicU64::new(0);
/// Active session token (0 = no session).
static CURRENT_GEN: AtomicU64 = AtomicU64::new(0);

/// Active sessions beyond this threshold are considered stale (zombie).
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

/// `true` only if a session is active and has not exceeded the stale threshold.
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
        crate::app_trace!("capture_session: auto-clearing stale session after {elapsed}ms");
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

/// Active session token (0 if none).
pub fn current_generation() -> u64 {
    CURRENT_GEN.load(Ordering::SeqCst)
}

pub fn current_hub_show_epoch() -> u64 {
    HUB_SHOW_EPOCH.load(Ordering::SeqCst)
}

/// Returns `false` if a capture is active or if `epoch` was invalidated by a later `begin`.
pub fn should_show_hub(epoch: u64) -> bool {
    !is_active_fresh() && epoch == current_hub_show_epoch()
}

/// Starts a session and returns its token. If one is already active, returns `0` (coalesced).
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

/// `true` if a task with token `gen` can close the current active session.
fn can_end_generation(gen: u64, current: u64) -> bool {
    gen != 0 && gen == current
}

/// Ends the session only if `gen` is still the active session.
/// Prevents a stale task from closing a more recent capture session.
pub fn end_generation(app: &AppHandle, gen: u64) {
    if !can_end_generation(gen, CURRENT_GEN.load(Ordering::SeqCst)) {
        if gen != 0 {
            crate::app_trace!(
                "capture_session: end_generation(gen={gen}) ignored (session not current)"
            );
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

/// Starts a session if none is active; dropping the guard calls `end_generation`.
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

/// Ends session `gen` when the guard is dropped (for async tasks that inherit an already-started session).
pub struct CaptureSessionEndGuard {
    app: AppHandle,
    gen: u64,
}

impl CaptureSessionEndGuard {
    /// Captures the current session token to close it safely later.
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
        // The owner of the current session can close it.
        assert!(can_end_generation(2, 2));
        // A stale token (from a superseded capture) cannot.
        assert!(!can_end_generation(1, 2));
        // A coalesced session (token 0) never closes the current session.
        assert!(!can_end_generation(0, 2));
        // With no current session, nothing is closed either.
        assert!(!can_end_generation(0, 0));
    }
}
