# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

Lore is an open-source, centralized, content-addressed version control system by Epic Games, written in Rust. Repository state is represented as Merkle trees and an immutable revision chain, optimized for binary-first storage, deduplication, and sparse/on-demand data hydration at scale.

## Build and test

Requires stable Rust + nightly for formatting, and Python 3.13+ with `uv` for smoke tests.

```sh
cargo build                                                        # build everything
cargo +nightly fmt --all                                           # format (nightly required)
cargo clippy --all-targets -- -D warnings --no-deps                # lint (zero warnings enforced)
cargo test                                                         # all Rust tests
cargo test -p <crate-name>                                         # single crate
cargo test <test_name>                                             # single test by name
uv run pytest                                                      # smoke tests (Python)
uv run pytest scripts/test/ --lore-client-binary=release --lore-server-binary=release
```

Build profiles: `release` is the fast local dev build (debug symbols, incremental, debug-assertions on). `release-lto` is the production build.

DCO sign-off is required on every commit:
```sh
git commit -s -m "Your message"
```

## Workspace structure

| Crate | Role |
|---|---|
| `lore` | Main library; the FFI/C API surface. Builds as `staticlib`/`rlib`/`cdylib`. Uses cbindgen to generate C headers. |
| `lore-revision` | Core VCS logic: commits, branches, Merkle tree, diff, merge, stage, history, lock, file tracking. The largest and most central crate. |
| `lore-storage` | Content-addressed fragment storage: chunked files, compression (zstd/lz4/oodle), deduplication, pack files, GC, local immutable/mutable stores. |
| `lore-transport` | Network transport layer: QUIC (via quinn) and gRPC session management, TLS, auth. |
| `lore-server` | Server binary (`loreserver`): gRPC + HTTP + QUIC frontends, auth/authz, store backends, topology. |
| `lore-client` | CLI binary (`lore`): all user-facing commands. |
| `lore-base` | Platform primitives: custom allocator, types (`Hash`, `Fragment`, `Address`), tokio runtime, fs helpers, Lore logging macros. |
| `lore-proto` | Protobuf definitions (tonic/prost). |
| `lore-credential` | Credential management and keyring integration. |
| `lore-aws` | AWS S3 storage backend. |
| `lore-hashicorp` | HashiCorp Vault credential integration. |
| `lore-telemetry` | OpenTelemetry tracing integration. |
| `lore-notification` | Async notification/event dispatch. |
| `lore-error-set` / `lore-error-set-macro` | Proc macros for composing error enums. |
| `lore-macro` | General proc macros. |
| `lore-chaos-client` | Fault injection client for testing. |
| `lore-integration-tests` | Cross-crate integration tests. |

`vendor/quinn-proto` is a local patch of the upstream quinn-proto crate; it is referenced via `[patch.crates-io]` in the workspace `Cargo.toml`.

## Architecture

The client library (`lore-revision`) holds all VCS logic and talks to `lore-storage` for content and `lore-transport` for remote I/O. The `lore` crate wraps `lore-revision` into a C-callable FFI surface (with cbindgen headers), which is what language SDKs (JS, Python, C#, Go) bind against. The `lore-client` CLI calls the same library directly via Rust. `lore-server` is a separate process that clients connect to; it owns its own storage stack.

Key domain concepts live in `lore-revision`:
- **Revision** — an immutable, content-hashed snapshot; revisions chain into a tamper-evident history.
- **Branch** — a mutable named pointer to a revision.
- **Stage** — a pending changeset not yet committed.
- **Fragment** — the unit of stored data; large files are split into chunks, each stored and fetched independently.
- **LORE_CONTEXT** — a per-execution-context value threaded through all async tasks via `lore_spawn!` macros.

## Code standards

### Error handling

- Library crates use `thiserror` with typed error enums. `anyhow` is allowed only in binaries.
- Never use `unwrap()` or `expect()` in production code — a panic in `lore-server` crashes the server.
- Use the extension traits from `lore-revision/src/error.rs`:
  - `emit_map_err` / `emit()` — for unexpected failures (logs at ERROR)
  - `debug_map_err` / `debug()` — for expected/recoverable failures (logs at DEBUG)
- Errors that surface via the C FFI implement `EventError` (in `lore-revision`) and translate to `LoreError` codes. `error_code` on `LoreErrorDetail` is the canonical consumer-facing code; `LoreError` and `EventError` are legacy transition paths.
- In `lore-server` gRPC handlers, use `warn_map_err` / `warn_error_to_status` when converting internal errors to `Status` for server errors (not for expected user errors like `NotFound`).

### Logging

- **Server and tool code** (`lore-server`, `lore-chaos-client`, `lore-aws`): use the `tracing` crate (`info!`, `warn!`, `error!`, etc.).
- **Library code** (`lore-revision`, `lore-base`, etc.): use the Lore macros (`lore_info!`, `lore_warn!`, `lore_error!`). Do not use `tracing` directly in library crates.
- Control server log verbosity with `RUST_LOG`.

### Task spawning

Always use the `lore_spawn!` family of macros (defined in `lore-base/src/runtime.rs`) to spawn async or blocking tasks — never `tokio::spawn` directly. This ensures `LORE_CONTEXT` propagates across task boundaries.

### Testing

- Async Rust tests must use the `LORE_CONTEXT.scope(setup_test_execution(), async { ... }).await` pattern.
- Keep tests independent: avoid `#[serial]` by using isolated fixtures (e.g. `test_store_create()` from `lore-revision/tests/helper.rs`).
- Every new CLI command needs smoke test coverage in `scripts/test/`.
- Smoke test fixtures: `new_lore_repo`, `auto_lore_local_server`, `lore_executable_path`.

## Contributing notes

- Open a GitHub Issue before writing code for anything beyond a trivial fix.
- Changes to the wire protocol, on-disk format, public APIs, or cross-cutting features require a Lore Enhancement Proposal (LEP) in `docs/proposals/` before implementation.
- PRs require two Maintainer approvals.
- AI tool usage must be disclosed in the PR description; AI-generated PR descriptions are not accepted.
- Do not introduce GPL/LGPL/AGPL dependencies (incompatible with the MIT license).
- New files need a copyright header: `// Copyright <year> <Name>` + `// SPDX-License-Identifier: MIT`.
