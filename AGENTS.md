# Lucid Agent Guidelines

Be sure to read the [README](./README.adoc) for information on what Lucid
actually is and aims to enable.

Lucid is built from three key components:

- `api/`, the API service that everything interfaces with
- `agent/`, the host agent responsible for collecting host telemetry
- `console/`, the web UI used by administrators to interface with the system

## Coding Guidelines

### Rust

- All dependencies must be declared in the root `Cargo.toml`, not locally to
  individual crates.

- After editing any Rust code, you **must** run `cargo fmt` on the changed
  files.

- Always write unit tests for logic, write integration tests where relevant.
