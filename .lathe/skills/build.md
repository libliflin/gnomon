# Build

## Dev commands

```sh
cargo build                    # debug build
cargo test                     # run all tests
cargo clippy -- -D warnings    # must pass clean before any PR
cargo check --workspace --all-targets  # fast type-check
```

All three of `build`, `test`, `clippy` must pass before pushing. CI enforces them.

## Binary

The `gnomon` binary entry point is `src/main.rs`. There is also `src/lib.rs` that re-exports the public API (`AuditReport`, `Budget`, `Preset`, `Violation`, `ViolationKind`) — gnomon is usable as a library crate.

## Release profile

```toml
[profile.release]
lto = "fat"
codegen-units = 1
strip = true
opt-level = 3
panic = "abort"
```

Produces a small, stripped binary. Target binary size: < 4 MB stripped x86_64 (per PLAN.md §13). No build scripts, no code generation, no unusual build steps.

## Distribution

Pre-built binaries on GitHub Releases for:
- Linux x86_64 musl (the CI runner target: `gnomon-x86_64-unknown-linux-musl.tar.gz`)
- macOS aarch64 (`gnomon-aarch64-apple-darwin.tar.gz`)
- macOS x86_64 (`gnomon-x86_64-apple-darwin.tar.gz`)

Also supported: `cargo binstall gnomon` and `cargo install gnomon` (from crates.io).

The `[package.metadata.binstall]` section in `Cargo.toml` configures cargo-binstall to download the right tarball from GitHub Releases.

## Release CI

`.github/workflows/release.yml` — cross-compiles and uploads tarballs to GitHub Releases on tag push. (Not read in detail; inferred from README install commands and the release tag format `v0.0.2`.)

## No unusual steps

Standard Rust project. No build scripts (`build.rs`), no code generation, no proc macros, no workspace (single crate at v0.0.2 — multi-crate workspace is planned in PLAN.md but not yet structured).

## Performance budget for gnomon itself

From PLAN.md §13 — not yet gated in CI but declared as targets:
- Cold start → first output: < 10 ms
- Static audit of one HTML page: < 1 ms
- Full crawl + static audit of 100-page site: < 100 ms
- Peak RSS (1000 pages): < 30 MB
- Binary size stripped x86_64: < 4 MB
