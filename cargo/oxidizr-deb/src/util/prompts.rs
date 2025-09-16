use std::io::{self, Write};

// Backward-compat stub
pub fn confirm_default_yes(_msg: &str) -> bool { true }

// Only prompt on TTY; in non-tty contexts (CI), do not block and proceed.
pub fn should_proceed(assume_yes: bool, _root: &std::path::Path) -> bool {
    if assume_yes { return true; }
    if atty::is(atty::Stream::Stdin) && atty::is(atty::Stream::Stdout) {
        eprintln!("This will modify the target root. Proceed? [y/N]: ");
        let _ = io::stderr().flush();
        let mut buf = String::new();
        if io::stdin().read_line(&mut buf).is_ok() {
            let s = buf.trim().to_lowercase();
            return s == "y" || s == "yes";
        }
        false
    } else {
        // Non-interactive: treat --commit as explicit consent already.
        true
    }
}
