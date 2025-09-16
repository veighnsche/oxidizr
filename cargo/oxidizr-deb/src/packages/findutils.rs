pub fn applets() -> Vec<String> {
    ["find", "xargs"].iter().map(|s| s.to_string()).collect()
}
