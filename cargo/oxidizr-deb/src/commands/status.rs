use std::path::Path;

pub fn exec(root: &Path) -> Result<(), String> {
    let ls = root.join("usr/bin/ls");
    let find = root.join("usr/bin/find");
    let sudo = root.join("usr/bin/sudo");
    let coreutils_active = ls.symlink_metadata().map(|m| m.file_type().is_symlink()).unwrap_or(false);
    let findutils_active = find.symlink_metadata().map(|m| m.file_type().is_symlink()).unwrap_or(false);
    let sudo_active = sudo.symlink_metadata().map(|m| m.file_type().is_symlink()).unwrap_or(false);
    println!("coreutils: {}", if coreutils_active { "active" } else { "unset" });
    println!("findutils: {}", if findutils_active { "active" } else { "unset" });
    println!("sudo: {}", if sudo_active { "active" } else { "unset" });
    Ok(())
}
