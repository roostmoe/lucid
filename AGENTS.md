# Lucid Agent Instructions

This document provides guidelines for AI coding agents working in the Lucid codebase. Lucid is a self-hosted infrastructure management platform (think Red Hat Insights alternative) for managing Linux fleets, built with Rust, Diesel ORM, and Dropshot HTTP framework.

## Project Context

- **Type**: Multi-tenant web service/daemon with host agent component
- **Stack**: Rust 2024 edition, Dropshot (HTTP API), Diesel (PostgreSQL ORM), Tokio (async runtime)
- **Architecture**: Workspace with 13+ crates (auth, beacon, db, types, common, etc.)
- **Database**: PostgreSQL 18 with Diesel migrations
- **Auth**: mTLS for agent auth, JWT + Oso/Polar for API authz
- **Current phase**: Phase 1 (Inventory & Agent Core) - see `docs/roadmap.adoc`

## Build, Test, and Lint Commands

### Basic Commands

```bash
# Build all crates
cargo build
cargo build --release

# Run the beacon server
cargo run

# Fast compile check (no binary)
cargo check

# Format code (ALWAYS run before commits)
cargo fmt

# Check formatting without changes
cargo fmt --check

# Lint with Clippy
cargo clippy
cargo clippy --all-targets --all-features
```

### Testing

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p lucid-auth
cargo test -p lucid-beacon

# Run a single test by name
cargo test test_parse_cookies_empty_headers

# Run tests matching a pattern
cargo test parse_cookies

# Run with output shown
cargo test -- --nocapture

# Run a specific test in a specific crate
cargo test -p lucid-types test_actor_required
```

### Database Migrations

```bash
# From db/schema/ directory:
cd db/schema
diesel migration run
diesel migration revert
diesel migration redo  # revert + run

# Generate schema.rs (after migration changes)
diesel migration run
```

### Development Environment

```bash
# Start PostgreSQL container
docker compose up -d

# Stop containers
docker compose down

# Check mise toolchain
mise doctor
mise list
```

## Code Style Guidelines

### Import Organization

Use three-tier grouping with blank lines between:

```rust
// 1. Standard library
use std::collections::BTreeMap;
use std::sync::Arc;

// 2. External crates (alphabetical)
use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

// 3. Internal crates
use lucid_types::dto::{params, views};
use lucid_uuid_kinds::{OrganisationIdKind, UserIdUuid};

use crate::models::User;
```

### Formatting

- **ALWAYS run `cargo fmt` before commits**
- Use default rustfmt settings (no custom config)
- 4-space indentation
- K&R brace style (opening brace same line)
- Trailing commas in struct fields and derives
- Keep lines reasonable (~100 chars)

### Types and Error Handling

```rust
// Use thiserror for typed errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("bad authentication credentials: {source:#}")]
    BadFormat { #[source] source: anyhow::Error },
    
    #[error("unknown actor {actor:?}")]
    UnknownActor { actor: String },
}

// Use anyhow for internal error propagation
fn internal_logic() -> anyhow::Result<()> {
    // ...
}

// Standard Result return types
fn api_handler() -> Result<Response, Error> {
    // ...
}

// Use lucid_common::api::error::Error for API errors
Error::internal_error(&format!("unexpected state: {}", msg))
```

### Naming Conventions

- **Structs/Enums/Traits**: `PascalCase` (`OpContext`, `UserIdUuid`, `Actor`)
- **Functions/Methods**: `snake_case` (`load_roles`, `actor_required`)
- **Constants**: `SCREAMING_SNAKE_CASE` (`DATABASE`, `LUCID_AUTHZ_CONFIG_BASE`)
- **Type parameters**: Single letter or descriptive (`T`, `Context`, `Resource`)
- **Modules**: `snake_case` (single word preferred)

### Async Patterns

```rust
// Prefer explicit async fn
async fn fetch_user(id: UserIdUuid) -> Result<User, Error> {
    // ...
}

// Use BoxFuture for trait methods with lifetimes
use futures::future::BoxFuture;

fn load_roles<'fut>(
    &'fut self,
    opctx: &'fut OpContext,
) -> BoxFuture<'fut, Result<(), Error>> {
    async move {
        // implementation
    }.boxed()
}

// For immediate returns in traits
use futures::future;

fn immediate_return(&self) -> BoxFuture<'_, Result<(), Error>> {
    future::ready(Ok(())).boxed()
}
```

### Database Models

```rust
use diesel::{Insertable, Queryable, Selectable};
use lucid_db_macros::Resource;
use lucid_db_schema::schema::users;

#[derive(Clone, Debug, Queryable, Insertable, Selectable, Resource)]
#[diesel(table_name = users)]
#[resource(uuid_kind = UserIdKind, deletable = false)]
pub struct User {
    pub id: DbTypedUuid<UserIdKind>,
    pub name: String,
    pub email: String,
    // Use DbTypedUuid<T> for typed UUID columns
    // Use #[diesel(embed)] for identity structs
}
```

### Dropshot API Endpoints

```rust
#[endpoint {
    method = POST,
    path = "/v1/users",
    tags = ["users"],
}]
async fn user_create(
    rqctx: RequestContext<ServerContext>,
    body: TypedBody<params::UserCreate>,
) -> Result<HttpResponseOk<views::User>, HttpError> {
    let apictx = rqctx.context();
    let opctx = crate::context::op_context_for_authn(&rqctx).await?;
    
    // Implementation
}
```

### Documentation

- Use `///` for doc comments on public items
- Focus on WHY, not WHAT (code should be self-documenting)
- Document invariants, safety requirements, and non-obvious behavior
- Module-level docs (`//!`) for complex modules

```rust
/// Operational context threaded through every datastore call.
/// 
/// Carries the authenticated actor, their loaded permissions, timing metadata,
/// and the kind of operation being performed. Authorization checks happen here
/// — not at the HTTP layer.
pub struct OpContext {
    // ...
}
```

### Testing

```rust
#[cfg(test)]
mod test {
    use super::*;
    use http::HeaderMap;

    #[test]
    fn test_parse_cookies_empty_headers() {
        let headers = HeaderMap::new();
        let cookies = parse_cookies(&headers).unwrap();
        assert_eq!(cookies.iter().count(), 0);
    }
    
    // Use descriptive test names with underscores
    // Test one thing per test
    // Prefer `assert_eq!` over `assert!` for better error messages
}
```

## Common Patterns

### NewType Pattern for Type Safety

The codebase extensively uses typed UUIDs via `lucid_uuid_kinds`:

```rust
use lucid_uuid_kinds::{UserIdKind, OrganisationIdKind};

// Database: DbTypedUuid<T>
pub struct User {
    pub id: DbTypedUuid<UserIdKind>,
}

// API/in-memory: just the typed UUID
pub fn fetch_user(id: UserIdUuid) -> Result<User, Error> {
    // ...
}
```

### Shared Ownership

Use `Arc<T>` for shared ownership across async boundaries:

```rust
pub struct ServerContext {
    pub authn: Arc<AuthnContext>,
    pub authz: Arc<Authz>,
}
```

### Prefer BTreeMap/BTreeSet

For deterministic ordering, prefer `BTreeMap`/`BTreeSet` over `HashMap`/`HashSet`.

### Spelling

Use **British English** in comments and documentation:
- "authorisation" not "authorization"
- "organisation" not "organization"
- "behaviour" not "behavior"

## Key Crates

- `beacon/` - Main HTTP server binary
- `auth/` - Authentication and authorization (Oso/Polar)
- `db/` - Database layer (Diesel models, schema, migrations)
- `types/` - Core domain types (Identity, OpContext, DTOs)
- `common/` - Shared utilities
- `uuid-kinds/` - Typed UUID wrappers

## Critical Notes

- **Thread `OpContext` everywhere** - it carries authn/authz context
- **Authorization at the datastore layer**, not HTTP layer
- **All DB operations must go through `OpContext`**
- **Use typed UUIDs** (`DbTypedUuid<T>`) for compile-time safety
- **Run `cargo fmt` before every commit**
- **British spelling in all text**