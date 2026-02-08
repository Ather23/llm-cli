# AGENTS.md - LLM-CLI

Guide for AI agents working on this Rust CLI application for interacting with LLMs.

## Project Overview

This is a Rust CLI tool (`llm-cli`) that provides an interactive chat interface with
Anthropic's Claude models. It uses the `rig` framework for LLM integration and persists
chat sessions to JSON files.

**Tech Stack:** Rust 2024 edition, Tokio async runtime, Clap for CLI parsing, Rig for
LLM integration, Serde for serialization.

---

## Build Commands

```bash
cargo build                    # Debug build
cargo build --release          # Release build with optimizations
cargo run                      # Run debug build
cargo run -- -p "prompt"       # Run with prompt argument
cargo check                    # Type checking (fast, no codegen)
```

## Testing Commands

```bash
cargo test                     # Run all tests
cargo test test_name           # Run a single test by name
cargo test test_name -- --exact # Run test with exact name match
cargo test module_name::       # Run tests in a specific module
cargo test -- --nocapture      # Run tests with output displayed
```

## Linting and Formatting

```bash
cargo fmt                      # Format code (rustfmt)
cargo fmt -- --check           # Check formatting without modifying files
cargo clippy                   # Run clippy linter
cargo clippy -- -D warnings    # Run clippy with all warnings as errors
cargo clippy --fix             # Auto-fix lint issues where possible

# Full pre-commit check
cargo fmt -- --check && cargo clippy -- -D warnings && cargo test
```

---

## Code Style Guidelines

### Import Organization

Organize imports in this order, with blank lines between groups:

```rust
// 1. External crates (alphabetical within group)
use async_stream::stream;
use clap::Parser;
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};

// 2. Standard library
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;

// 3. Local modules (crate::)
use crate::persistence::Persistence;
```

- Group multiple items from the same crate using `{}`: `use std::io::{self, BufRead, Write};`
- Use `self` when importing both the module and items from it

### Naming Conventions

| Element       | Convention          | Example                    |
|---------------|---------------------|----------------------------|
| Types/Structs | PascalCase          | `ChatMessage`, `UserType`  |
| Enums         | PascalCase          | `LlmResponse`              |
| Enum Variants | PascalCase          | `User`, `Assistant`        |
| Functions     | snake_case          | `create_session_dir`       |
| Variables     | snake_case          | `session_path`             |
| Constants     | SCREAMING_SNAKE_CASE| `LLM_ROOT_DIR`             |
| Modules       | snake_case          | `llm_core`, `persistence`  |

### Error Handling

Use `anyhow` for application-level error handling:

```rust
// Propagate errors with ?
let file = File::open(file_path)?;

// Fallback with unwrap_or_else for recoverable cases
serde_json::from_reader(reader).unwrap_or_else(|_| Vec::new())

// expect() only for truly unrecoverable errors with clear message
fs::create_dir(dir).expect("Unable to create required directory");
```

**Guidelines:**
- Prefer `?` operator over `.unwrap()` or `.expect()` in most cases
- Use `.context()` from anyhow to add meaningful error messages
- Reserve `.expect()` for initialization failures that should panic
- Use `unwrap_or_else` or `unwrap_or_default` for fallback values

### Struct and Enum Definitions

```rust
// Derive macros in this order: Parser > Serialize/Deserialize > Debug > Clone > others
#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    prompt: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct ChatMessage {
    user_type: UserType,
    message: String,
}
```

### Async Patterns

- Use `#[tokio::main]` for the async entry point
- Use `async-stream` crate with `stream!` macro for async generators
- Return `Pin<Box<dyn Stream<Item = T> + Send + 'a>>` for dynamic streams
- Chain `.await` calls where readable, use intermediate variables for clarity

### File Structure

```
src/
  main.rs              # Entry point, CLI parsing, main loop
  llm_core/            # LLM integration logic
    mod.rs             # Module exports and re-exports
    core.rs            # Core types: ChatMessage, UserType, LlmResponse, Llm trait
    llm.rs             # LlmCore implementation (Anthropic client)
  persistence/         # Data persistence
    mod.rs             # Module exports and re-exports
    core.rs            # Persistence trait definition
    local.rs           # LocalPersistence implementation (JSON file storage)
```

### Module Architecture

**llm_core module:**
- `core.rs`: Defines shared types (`ChatMessage`, `UserType`, `LlmResponse`) and the `Llm` trait
- `llm.rs`: Implements `LlmCore` struct with `generate_response()` method using Anthropic's Claude API
- `mod.rs`: Re-exports types for convenient imports: `use llm_core::{ChatMessage, Llm, LlmCore, ...}`

**persistence module:**
- `core.rs`: Defines the `Persistence` trait with `store_chat_message()` method
- `local.rs`: Implements `LocalPersistence` for JSON file-based storage
- `mod.rs`: Re-exports types: `use persistence::{Persistence, LocalPersistence}`

### Trait-Based Design

The codebase uses traits to abstract implementations:

```rust
// LLM abstraction - allows different LLM providers
pub trait Llm {
    fn generate_response<'a>(
        &'a self,
        chat_history: &'a [ChatMessage],
        prompt: &'a str,
    ) -> Pin<Box<dyn Stream<Item = LlmResponse> + Send + 'a>>;
}

// Persistence abstraction - allows different storage backends
pub trait Persistence {
    fn store_chat_message(
        &self,
        message: &ChatMessage,
    ) -> Result<(), anyhow::Error>;
}
```

Usage in main.rs:
```rust
let llm = LlmCore::new();
let persistence = LocalPersistence::new(&session_path);

// Use via trait methods
let stream = llm.generate_response(&history, &prompt);
persistence.store_chat_message(&message)?;
```

### General Style

- Keep functions focused and under ~50 lines when possible
- Use early returns for guard clauses
- Prefer iterators and combinators over explicit loops where clear
- Add comments for non-obvious logic, not for self-documenting code
- Use `&str` for function parameters, `String` for owned data in structs

---

## Environment Setup

Required environment variable for Anthropic API access:
```bash
export ANTHROPIC_API_KEY="your-api-key"
```

## Dependencies

Key crates used in this project:
- `rig-core`: LLM integration framework
- `tokio`: Async runtime
- `clap`: CLI argument parsing with derive macros
- `anyhow`: Flexible error handling
- `serde` + `serde_json`: Serialization
- `futures` + `async-stream`: Async streaming utilities
- `uuid`: Session ID generation
