# Build — Gnomon

## Standard commands

```sh
cargo build                    # debug build
cargo build --release          # release build (LTO, stripped, opt-level=3)
cargo test                     # run all tests
cargo clippy --all-targets     # lint (treat as blocking)
cargo clippy -- -D warnings    # lint with warnings-as-errors (the standard)
```

## Release profile

`Cargo.toml` configures an aggressive release profile:
- `lto = "fat"` — link-time optimization across all crates
- `codegen-units = 1` — maximum optimization, slower compile
- `strip = true` — strips debug symbols from the binary
- `opt-level = 3`
- `panic = "abort"` — no unwinding

This means `cargo build --release` is slow (tens of seconds). Use debug builds during development.

## No CI yet

There are no `.github/workflows/` files. CI is not configured. This means:
- No automated build checks on PRs
- No automated test runs
- No clippy enforcement
- No binary size checks

The snapshot.sh checks `cargo build`, `cargo test`, and `cargo clippy` locally, which substitutes for CI during lathe cycles.

## Feature flags

`gnomon-measure` (headless Chromium) will be feature-gated but is not implemented yet. No feature flags currently affect the build.

## Optional allocator

A `mimalloc` feature is mentioned in PLAN.md for deployments that care about microsecond-level allocator performance, but it is not in `Cargo.toml` yet.

## Binary output

The built binary is at `target/release/gnomon`. It is a single static binary with no runtime dependencies (Rust + musl for Linux targets).
