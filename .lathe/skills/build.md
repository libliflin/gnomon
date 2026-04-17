# Build

## Development build

```sh
cargo build          # debug build, fast compile
cargo build --release   # optimized binary (LTO fat, strip, O3, panic=abort)
```

Release profile is aggressive — intended for distribution:
```toml
[profile.release]
lto = "fat"
codegen-units = 1
strip = true
opt-level = 3
panic = "abort"
```

## Lint

```sh
cargo clippy -- -D warnings    # warnings are errors; must be clean before committing
```

As of v0.0.2, clippy reports ~9 diagnostics (per init-snapshot.log). These must be fixed — the project's own standard demands a clean build.

## Format

```sh
cargo fmt              # format; CI should enforce
cargo fmt -- --check   # fail if not formatted
```

## Feature flags

None currently. The `gnomon-measure` crate (headless browser) is planned as a feature-gated dependency (`features = ["measure"]`) — not yet implemented.

## Key dependencies (and why)

- `clap 4` + derive — CLI argument parsing
- `tokio` multi-thread — async runtime for URL fetching
- `reqwest 0.12` + rustls-tls — HTTP client (no OpenSSL dependency)
- `scraper 0.20` — HTML parsing via CSS selectors
- `url 2` — URL parsing and resolution
- `serde` + `serde_json` + `toml 0.8` — serialization and config parsing
- `thiserror` + `anyhow` — error types
- `owo-colors 4` — terminal color output (no unsafe, no runtime detect needed)
- `brotli 7` + `flate2` — transfer size recomputation
- `futures 0.3` — stream processing for concurrent fetches
- `humansize 2` — human-readable byte sizes in output
- `aho-corasick 1` — forbidden list pattern matching

## Binary targets

Two targets in Cargo.toml:
- `lib` at `src/lib.rs` — the public API
- `bin` "gnomon" at `src/main.rs` — the CLI

Both are built with `cargo build`.

## Planned but not present

- Benchmark suite (`cargo bench`) — PLAN.md §13 describes timing gates for cold start, per-page audit, 100-page crawl
- Precompiled binary distribution (GitHub Releases) — not yet set up
- GitHub Actions for CI — not configured
- `gnomon-action` GitHub Action — v0.2 roadmap
