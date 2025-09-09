use std::io;
use std::path::{Path, PathBuf};
use std::sync::Once;

use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, Registry};
use tracing_subscriber::Layer;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::filter::{LevelFilter, FilterFn};
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::time::ChronoLocal;

use super::audit::AUDIT_LOG_PATH;

static INIT: Once = Once::new();

/// Initialize global tracing subscribers for human logs and audit JSONL.
///
/// - Human logs go to stderr with level controlled by VERBOSE env (0..3)
/// - Audit events (target="audit") are written as single-line JSON to AUDIT_LOG_PATH
pub fn init_logging() {
    INIT.call_once(|| {
        // Capture legacy log:: macros and route them into tracing
        let _ = LogTracer::init();

        // Map VERBOSE to a LevelFilter; fallback to INFO when unset
        let level = match std::env::var("VERBOSE").ok().and_then(|s| s.parse::<u8>().ok()) {
            Some(0) => LevelFilter::ERROR,
            Some(1) => LevelFilter::INFO,
            Some(2) => LevelFilter::DEBUG,
            Some(3) => LevelFilter::TRACE,
            _ => LevelFilter::INFO,
        };

        // Human-readable layer to stderr
        let human_layer = fmt::layer()
            .with_writer(io::stderr)
            .with_ansi(atty::is(atty::Stream::Stderr))
            .with_target(false)
            .with_level(true)
            .with_timer(ChronoLocal::rfc_3339())
            .with_filter(level);

        // JSONL audit layer to file, only for target=="audit". We provide our own timestamp field
        // inside audit_event, so we do not attach a timer here to avoid duplicate timestamps.
        let audit_layer = fmt::layer()
            .json()
            .flatten_event(true) // move fields to top-level
            .with_current_span(false)
            .with_span_list(false)
            .with_level(false)
            .with_target(false)
            .with_writer(AuditMakeWriter::new(PathBuf::from(AUDIT_LOG_PATH)))
            .with_filter(FilterFn::new(|meta| meta.target() == "audit"));

        let subscriber = Registry::default()
            .with(human_layer)
            .with(audit_layer);

        // Install the composed subscriber
        let _ = subscriber.try_init();
    });
}

/// A MakeWriter that appends to an audit log file.
/// Attempts primary path and falls back to $HOME/.oxidizr-arch-audit.log on error.
struct AuditMakeWriter {
    primary: PathBuf,
}

impl AuditMakeWriter {
    pub fn new(primary: PathBuf) -> Self { Self { primary } }

    fn open(&self) -> io::Result<std::fs::File> {
        // Try primary path, fallback to HOME if needed
        match open_append(&self.primary) {
            Ok(f) => Ok(f),
            Err(_) => {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                let fallback = Path::new(&home).join(".oxidizr-arch-audit.log");
                open_append(&fallback)
            }
        }
    }
}

fn open_append(path: &Path) -> io::Result<std::fs::File> {
    if let Some(parent) = path.parent() { std::fs::create_dir_all(parent).ok(); }
    std::fs::OpenOptions::new().create(true).append(true).open(path)
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for AuditMakeWriter {
    type Writer = AuditWriter;
    fn make_writer(&'a self) -> Self::Writer {
        let file = self.open().expect("failed to open audit log file");
        AuditWriter { file: Some(file) }
    }
}

/// Simple wrapper that implements Write over a std::fs::File
pub struct AuditWriter {
    file: Option<std::fs::File>,
}

impl io::Write for AuditWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if let Some(f) = self.file.as_mut() { f.write(buf) } else { Ok(buf.len()) }
    }
    fn flush(&mut self) -> io::Result<()> {
        if let Some(f) = self.file.as_mut() { f.flush() } else { Ok(()) }
    }
}
