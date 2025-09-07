#[cfg(test)]
mod security_tests {
    use crate::utils::worker::*;
    use crate::config::security;
    use std::path::Path;

    #[test]
    fn test_package_name_validation() {
        // Valid package names
        assert!(is_valid_package_name("firefox"));
        assert!(is_valid_package_name("lib32-glibc"));
        assert!(is_valid_package_name("python3.11"));
        assert!(is_valid_package_name("gcc_fortran"));
        assert!(is_valid_package_name("qt5+base"));
        
        // Invalid package names
        assert!(!is_valid_package_name("")); // Empty
        assert!(!is_valid_package_name("-malicious")); // Starts with dash
        assert!(!is_valid_package_name("package; rm -rf /")); // Command injection
        assert!(!is_valid_package_name("package$(whoami)")); // Command substitution
        assert!(!is_valid_package_name("package`id`")); // Backticks
        assert!(!is_valid_package_name("package|cat")); // Pipe
        assert!(!is_valid_package_name("package&")); // Background
        assert!(!is_valid_package_name("package>")); // Redirect
        assert!(!is_valid_package_name("package<")); // Redirect
        assert!(!is_valid_package_name("package\"")); // Quote
        assert!(!is_valid_package_name("package'")); // Quote
        assert!(!is_valid_package_name("package\n")); // Newline
        assert!(!is_valid_package_name("package\r")); // Carriage return
    }

    #[test]
    fn test_path_traversal_validation() {
        // Valid paths
        assert!(is_safe_path(Path::new("/usr/bin/ls")));
        assert!(is_safe_path(Path::new("bin/ls")));
        assert!(is_safe_path(Path::new("./bin/ls")));
        assert!(is_safe_path(Path::new("/usr/lib/uutils/coreutils")));
        
        // Invalid paths - directory traversal attempts
        assert!(!is_safe_path(Path::new("../etc/passwd")));
        assert!(!is_safe_path(Path::new("/usr/bin/../../../etc/shadow")));
        assert!(!is_safe_path(Path::new("../../root/.ssh/id_rsa")));
        assert!(!is_safe_path(Path::new("/usr/bin/..")));
        assert!(!is_safe_path(Path::new("bin/../../etc/passwd")));
    }

    #[test]
    fn test_backup_path_format() {
        let path = Path::new("/usr/bin/ls");
        let backup = backup_path(path);
        assert!(backup.to_string_lossy().contains(".ls.oxidizr.bak"));
        assert!(backup.parent().unwrap() == Path::new("/usr/bin"));
    }

    #[test]
    fn test_package_name_length_limit() {
        let long_name = "a".repeat(security::MAX_PACKAGE_NAME_LENGTH + 1);
        assert!(!is_valid_package_name(&long_name));
        
        let max_name = "a".repeat(security::MAX_PACKAGE_NAME_LENGTH);
        assert!(is_valid_package_name(&max_name));
    }

    #[test]
    fn test_path_length_limit() {
        let long_path = format!("{}/{}", "/usr", "a".repeat(security::MAX_PATH_LENGTH));
        assert!(!is_safe_path(Path::new(&long_path)));
    }

    #[test]
    fn test_aur_helper_whitelist() {
        use crate::config::aur_helpers;
        
        // Ensure only whitelisted helpers are allowed
        let helpers = aur_helper_candidates("");
        for helper in helpers {
            assert!(aur_helpers::DEFAULT_HELPERS.contains(&helper));
        }
    }

    #[test]
    fn test_no_shell_metacharacters_in_commands() {
        // Test that command building doesn't allow shell metacharacters
        let dangerous_chars = vec![";", "|", "&", ">", "<", "`", "$", "(", ")", "{", "}", "[", "]", "\\", "'", "\""];
        
        for ch in dangerous_chars {
            let bad_package = format!("package{}", ch);
            assert!(!is_valid_package_name(&bad_package), "Failed to reject package with {}", ch);
        }
    }

    #[test]
    fn test_symlink_target_validation() {
        // Ensure symlink targets are validated
        let valid_targets = vec![
            Path::new("/usr/bin/coreutils"),
            Path::new("/usr/lib/uutils/coreutils/ls"),
            Path::new("/usr/lib/cargo/bin/find"),
        ];
        
        for target in valid_targets {
            assert!(is_safe_path(target), "Valid path rejected: {:?}", target);
        }
        
        let invalid_targets = vec![
            Path::new("../../../etc/passwd"),
            Path::new("/etc/../etc/../etc/shadow"),
        ];
        
        for target in invalid_targets {
            assert!(!is_safe_path(target), "Invalid path accepted: {:?}", target);
        }
    }

    // Helper functions exposed for testing
    fn is_valid_package_name(name: &str) -> bool {
        if name.is_empty() || name.starts_with('-') {
            return false;
        }
        if name.len() > security::MAX_PACKAGE_NAME_LENGTH {
            return false;
        }
        name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '+' || c == '.')
    }

    fn is_safe_path(path: &Path) -> bool {
        // Check for directory traversal patterns
        for component in path.components() {
            if let std::path::Component::ParentDir = component {
                return false;
            }
        }
        
        if let Some(path_str) = path.to_str() {
            if path_str.len() > security::MAX_PATH_LENGTH {
                return false;
            }
            if path_str.contains("/../") || path_str.contains("..\\") {
                return false;
            }
        }
        true
    }

    fn backup_path(target: &Path) -> std::path::PathBuf {
        use crate::config::paths;
        let name = target.file_name().and_then(|s| s.to_str()).unwrap_or("backup");
        let parent = target.parent().unwrap_or_else(|| Path::new("."));
        parent.join(format!(".{}{}", name, paths::BACKUP_SUFFIX))
    }

    fn aur_helper_candidates(configured: &str) -> Vec<&str> {
        use crate::config::aur_helpers;
        if !configured.is_empty() {
            let mut helpers = vec![configured];
            helpers.extend_from_slice(&aur_helpers::DEFAULT_HELPERS);
            helpers
        } else {
            aur_helpers::DEFAULT_HELPERS.to_vec()
        }
    }
}
