pub fn pm_lock_message(root: &std::path::Path) -> Option<String> {
    use fs2::FileExt;
    use std::fs::OpenOptions;
    let locks = [
        "/var/lib/dpkg/lock-frontend",
        "/var/lib/dpkg/lock",
        "/var/lib/apt/lists/lock",
    ];
    for l in locks {
        let p = root.join(l.strip_prefix('/').unwrap_or(l));
        if !p.exists() {
            continue;
        }
        if let Ok(f) = OpenOptions::new().read(true).write(true).open(&p) {
            match f.try_lock_exclusive() {
                Ok(_) => {
                    let _ = f.unlock();
                    continue;
                }
                Err(_) => {
                    return Some(
                        "Package manager busy (dpkg/apt lock detected); retry after current operation finishes.".to_string(),
                    );
                }
            }
        }
    }
    None
}
