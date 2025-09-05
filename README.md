# Rust Coreutils Switch

A Rust library and CLI tool for abstracting over different coreutils implementations (GNU coreutils and uutils).

## Features

- Switch between GNU coreutils and uutils implementations at runtime
- Unified interface for common core utilities
- Easy to extend with additional implementations

## Installation

```bash
cargo install --path .
```

## Usage

### As a Library

```rust
use rust_coreutils_switch::{create_core_util, CoreUtilsImpl};

// Create a GNU ls command
let ls = create_core_util("ls", CoreUtilsImpl::Gnu);
ls.execute(&["-la".to_string()]).unwrap();

// Create a uutils ls command
let ls = create_core_util("ls", CoreUtilsImpl::Uutils);
ls.execute(&["-la".to_string()]).unwrap();
```

### As a CLI Tool

```bash
# List available commands
coreutils-switch list

# Execute a command using GNU coreutils
coreutils-switch exec ls -la

# Execute a command using uutils
coreutils-switch --uutils exec ls -la
```

## Requirements

- Rust 1.60 or later
- GNU coreutils or uutils installed on your system

## License

Dual-licensed under MIT or Apache 2.0 at your option.
