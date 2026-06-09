# ai-pocket-core

Shared Rust core for **AI Pocket**, a screen-casting and capture hardware
product. This repository is the public, open-source heart of the system: it
holds the transport-agnostic domain model plus the abstractions and FFI surface
that every edge — desktop sender, mobile apps, relay server, and device
firmware — agrees on. The video codec is locked to **H.264**.

## Workspace layout

This is a Cargo workspace (`resolver = "2"`, `edition = "2024"`,
`rust-version = "1.85"`) with three crates:

| Crate | Path | Kind | Role |
| --- | --- | --- | --- |
| `ai-pocket-core` | `crates/core` | lib | Pure domain model: control protocol (`ControlMsg`), H.264 frame envelopes (`H264Frame` / `FrameKind`), session state machine (`Session`), device discovery descriptors (`DeviceDescriptor`), and the unified `CoreError`. No I/O. |
| `ai-pocket-transport` | `crates/transport` | lib | Async (Tokio) transport traits: `Discovery` (local-network) and `RelayClient` (public-network relay). Depends on `ai-pocket-core`. Ships an in-memory `stub` implementation behind a feature flag for tests. |
| `ai-pocket-ffi` | `crates/ffi` | lib + cdylib + staticlib | UniFFI facade (`PocketClient`) for iOS (Swift) and Android (Kotlin). Depends on `ai-pocket-core` and `ai-pocket-transport`. |

All three crates share `version = 0.1.0` via `[workspace.package]`.

## Build

```bash
# Build and test the whole workspace.
cargo build --workspace
cargo test --workspace

# Lint as CI does.
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
```

## Generating mobile bindings (UniFFI)

`ai-pocket-ffi` uses the UniFFI **proc-macro** approach (no `.udl`): the API is
declared with `#[uniffi::export]`, `#[derive(uniffi::Record/Enum/Object/Error)]`,
and `uniffi::setup_scaffolding!()`. Bindings are produced by the bundled
`uniffi-bindgen` binary via a helper script:

```bash
# Swift (iOS)
./scripts/gen-bindings.sh swift  ./bindings/swift

# Kotlin (Android)
./scripts/gen-bindings.sh kotlin ./bindings/kotlin
```

The script builds the `cdylib` first, then runs
`cargo run -p ai-pocket-ffi --bin uniffi-bindgen -- generate --library ...`.
Per-language module/package names live in `crates/ffi/uniffi.toml`.

## Dependency mode: `path` (dev) vs `git tag` (release)

Inside this repo, internal crates depend on each other by **path** so that
local development needs no publishing step. Each path dependency is annotated
**directly above** with the locked **git tag** form to switch to at release
time. Example, from `crates/transport/Cargo.toml`:

```toml
# 发布期改用锁定版：ai-pocket-core = { git = "https://github.com/OpenLoaf/ai-pocket-core", tag = "v0.1.0" }
ai-pocket-core = { path = "../core" }
```

- **Development**: `path = "../core"` / `path = "../transport"` — fast local
  iteration, no registry round-trip.
- **Release**: replace each path dep with the commented `git + tag` form,
  pinned to an exact tag (e.g. `v0.1.0`), so downstream consumers
  (`ai-pocket-server`, `ai-pocket-desktop`, the private umbrella) get a
  reproducible, locked version.

Downstream repositories consuming this workspace follow the same rule:
**dev = path, release = git tag (exact pin)**.

> Replace `OpenLoaf` with the real GitHub org/owner before publishing.

## License

> **TODO (action required):** the license is currently a placeholder of
> `MIT OR Apache-2.0` declared in `[workspace.package]`. Confirm the final
> license and add the corresponding `LICENSE-MIT` / `LICENSE-APACHE` (or chosen
> license) files at the repo root before the first public release.
