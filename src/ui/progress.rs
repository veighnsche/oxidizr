use atty::Stream;
use indicatif::{ProgressBar, ProgressStyle, ProgressDrawTarget};
use std::sync::atomic::{AtomicBool, Ordering};

// Global toggle to quiet noisy per-item symlink INFO logs while a progress bar
// is active. Warnings and errors are never suppressed.
static QUIET_SYMLINK_INFO: AtomicBool = AtomicBool::new(false);

pub fn symlink_info_enabled() -> bool {
    !QUIET_SYMLINK_INFO.load(Ordering::Relaxed)
}

#[allow(dead_code)]
pub struct SymlinkQuietGuard(bool);

impl Drop for SymlinkQuietGuard {
    fn drop(&mut self) {
        QUIET_SYMLINK_INFO.store(false, Ordering::Relaxed);
    }
}

/// Enable quiet mode for symlink INFO logs; returns a guard that restores state on drop.
pub fn enable_symlink_quiet() -> SymlinkQuietGuard {
    QUIET_SYMLINK_INFO.store(true, Ordering::Relaxed);
    SymlinkQuietGuard(true)
}

/// Create a configured progress bar if running in a TTY and len > 0.
/// Returns None when not interactive or when len == 0, so callers can
/// gracefully fall back to plain logging.
pub fn new_bar(len: u64) -> Option<ProgressBar> {
    if len == 0 {
        return None;
    }

    // Allow forcing progress rendering even in non-TTY environments via env.
    // Recognized variables: OXI_PROGRESS=1 or OXIDIZR_PROGRESS=1 or PROGRESS=1
    let _force = matches!(
        std::env::var("OXI_PROGRESS").as_deref(),
        Ok("1") | Ok("true") | Ok("on")
    ) || matches!(
        std::env::var("OXIDIZR_PROGRESS").as_deref(),
        Ok("1") | Ok("true") | Ok("on")
    ) || matches!(
        std::env::var("PROGRESS").as_deref(),
        Ok("1") | Ok("true") | Ok("on")
    );

    let is_tty = atty::is(Stream::Stderr);

    // Reasonable redraw rate in non-TTY to be visible yet not too spammy
    let draw = if is_tty {
        ProgressDrawTarget::stderr()
    } else {
        ProgressDrawTarget::stderr_with_hz(10)
    };

    let pb = ProgressBar::with_draw_target(Some(len), draw);
    let style = ProgressStyle::with_template(
        "{spinner:.green} [{wide_bar:.cyan/blue}] {pos}/{len} {msg}"
    )
    .unwrap()
    .progress_chars("#>-");
    pb.set_style(style);
    Some(pb)
}

/// Set the current message and increment by 1 (no-op if pb is None)
pub fn set_msg_and_inc(pb: &Option<ProgressBar>, msg: impl AsRef<str>) {
    if let Some(ref bar) = pb.as_ref() {
        bar.set_message(msg.as_ref().to_string());
        bar.inc(1);
    }
}

/// Finish and clear the progress bar (no-op if None)
pub fn finish(pb: Option<ProgressBar>) {
    if let Some(bar) = pb {
        if atty::is(Stream::Stderr) {
            bar.finish_and_clear();
        } else {
            // Leave a final line so logs show completion in non-TTY capture
            bar.finish_with_message("Done");
        }
    }
}
