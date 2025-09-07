// Constants for coreutils paths and configurations
pub const COREUTILS_UNIFIED_PATH: &str = "/usr/bin/coreutils";
pub const COREUTILS_UNIFIED_CANDIDATES: [&str; 3] = [
    "/usr/lib/uutils/coreutils/coreutils",
    "/usr/lib/cargo/bin/coreutils",
    "/usr/bin/coreutils.uutils",
];
pub const COREUTILS_BINS_LIST: &str = include_str!("../../../tests/lib/rust-coreutils-bins.txt");
pub const SYSTEM_BIN_DIR: &str = "/usr/bin";
