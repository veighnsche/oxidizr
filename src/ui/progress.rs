use atty::Stream;
use indicatif::{ProgressBar, ProgressStyle};

/// Create a configured progress bar if running in a TTY and len > 0.
/// Returns None when not interactive or when len == 0, so callers can
/// gracefully fall back to plain logging.
pub fn new_bar(len: u64) -> Option<ProgressBar> {
    if len == 0 || !atty::is(Stream::Stderr) {
        return None;
    }
    let pb = ProgressBar::new(len);
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
        bar.finish_and_clear();
    }
}
