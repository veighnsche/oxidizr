pub fn pm_lock_message(root: &std::path::Path) -> Option<String> {
    let locks = [
        "/var/lib/dpkg/lock-frontend",
        "/var/lib/dpkg/lock",
        "/var/lib/apt/lists/lock",
    ];
    for l in locks {
        let p = root.join(l.strip_prefix('/').unwrap_or(l));
        if p.exists() {
            return Some("Package manager busy (dpkg/apt lock detected); retry after current operation finishes.".to_string());
        }
    }
    None
}
