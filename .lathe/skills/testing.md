# Testing

## Current state (v0.0.2)

The test suite is minimal. There is one test:

```rust
// src/lib.rs
#[cfg(test)]
mod tests {
    #[test]
    fn sanity() {
        assert!(true);
    }
}
```

This test exists to bootstrap the test runner and confirm `cargo test` completes. It tests nothing about gnomon's behavior.

## How to run tests

```sh
cargo test                     # all tests
cargo test --lib               # lib tests only
cargo test --lib --bins        # lib + binary tests
```

## What needs testing

The following behaviors are untested and represent the highest-value test targets:

**HTML analysis (`src/analyze.rs`):**
- `analyze_html()` — given an HTML string, produces correct `HtmlAnalysis`. Can be tested with static strings, no network needed.
- Render-blocking detection — script in head without async/defer is blocking; with async/defer is not.
- Anti-theater: first `<img loading="lazy">` sets `lazy_lcp_candidate`.
- Third-party URL extraction.
- Preload-as-style counts as render-blocking.

**Budget loading (`src/budget.rs`):**
- `resolve_budget()` with a valid TOML file — parses correctly.
- Invalid preset name in TOML — returns an error.
- Unknown keys in TOML — `#[serde(deny_unknown_fields)]` causes parse error.

**Violation logic (`src/audit.rs`):**
- A page that exceeds the HTML byte budget produces a `ViolationKind::Bytes` violation.
- A page that loads a forbidden domain produces a `ViolationKind::Forbidden` violation.
- A page that is clean produces no violations.

**Forbidden matcher (`src/forbidden.rs`):**
- `ForbiddenMatcher::find()` — known blocked URLs return a match.
- URLs not in the list return `None`.
- Case-insensitive matching.

## Test conventions to establish

- Unit tests for pure functions (analyze.rs, forbidden.rs, budget.rs) go in the same file as the function, in a `#[cfg(test)]` block.
- Integration tests that test the full `audit_url()` flow can use `mockito` or recorded HTTP responses, or can test against a locally-served static HTML file.
- Test HTML fixtures (`.html` files in `tests/fixtures/`) are the right way to test `analyze_html()` without building HTTP mocks.
- A test that exercises `audit_url()` against a real URL should be marked `#[ignore]` so it doesn't run in CI without network access.

## CI

No CI is configured as of v0.0.2. There is no `.github/workflows/` directory. The snapshot script checks for CI configuration and will report "No CI config found" each cycle until CI is added.
