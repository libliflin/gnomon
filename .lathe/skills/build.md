# Build

## Prerequisites

Rust stable toolchain. `rustup toolchain install stable`.

The code uses Rust 2024 edition (`edition = "2024"` in `Cargo.toml`) and let-chain syntax (`if let X && let Y`). Requires Rust 1.87+ (stable as of 2025).

## Dev commands

```sh
cargo build                          # debug build
cargo test                           # run all tests
cargo clippy -- -D warnings          # must pass clean — CI enforces this
```

All three must pass before any PR. CI runs all three jobs in parallel.

## Release build

```sh
cargo build --release
```

Release profile (`Cargo.toml`): `lto = "fat"`, `codegen-units = 1`, `strip = true`, `opt-level = 3`, `panic = "abort"`. Binary is stripped, statically linked on Linux targets.

## CI

`.github/workflows/ci.yml` runs on `pull_request` and `push` to `main`:
- `cargo check --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

All three jobs run in parallel. Rust toolchain is pinned to `stable` via `dtolnay/rust-toolchain@stable`. Caching via `Swatinem/rust-cache@v2`.

## Not yet wired in CI

- `cargo bench` (PLAN.md §13 performance gates)
- Binary size check (target: < 4 MB stripped)
- The cold-start timing gate (`< 10 ms` first output)

## Non-obvious dependency behavior

`reqwest` is configured with `default-features = false, features = ["rustls-tls", "brotli", "gzip", "deflate"]`. This means gnomon uses rustls (not system OpenSSL) for TLS — the binary is fully self-contained with no OpenSSL dependency.

`brotli` crate is used for recompression (computing wire-equivalent byte sizes). This is separate from reqwest's brotli decompression.

`scraper` is the HTML parser. PLAN.md §14 mentions `lol_html` for streaming parse in the future — the current code uses the DOM-building scraper for simplicity.
