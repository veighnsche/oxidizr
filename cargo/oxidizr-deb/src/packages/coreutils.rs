pub fn applets() -> Vec<String> {
    [
        "ls", "cp", "mv", "rm", "cat", "echo", "touch", "mkdir", "rmdir", "chmod", "chown", "ln",
        "head", "tail", "sort", "uniq", "wc", "basename", "dirname", "date",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}
