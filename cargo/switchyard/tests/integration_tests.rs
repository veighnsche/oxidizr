// Integration tests for the switchyard crate
//
// This file serves as the main entry point for all integration tests,
// including those organized in subdirectories.

// Include all test submodules
mod apply;
mod audit;
mod combinatorial;
mod environment;
mod fs;
mod helpers;
mod locking;
mod oracles;
mod plan;
mod preflight;
mod requirements;
mod rollback;
mod safepath;
mod scenario;

// The tests in each submodule will be automatically discovered and run by Rust's test harness
