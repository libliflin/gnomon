# Testing — Gnomon

## Current state

Tests are nearly absent. The only test is a sanity stub in `src/lib.rs`:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn sanity() {
        assert!(true);
    }
}
```

`cargo test` reports this as passing. The green result is misleading — it proves nothing about the tool's behavior.

## Test runner

Standard `cargo test`. No external test harness. Runs: unit tests (in-source `#[cfg(test)]` blocks) + doc tests (none currently) + integration tests (none in `tests/` directory).

## What's missing

- **Unit tests for `analyze.rs`**: the HTML analysis is the most complex logic (render-blocking detection, lazy LCP detection, inline byte counting). None of it is tested.
- **Unit tests for `forbidden.rs`**: the Aho-Corasick automaton matching against the forbidden list has no tests.
- **Unit tests for `budget.rs`**: preset loading and budget comparison have no tests.
- **Unit tests for `violation.rs`**: violation kind labeling has no tests.
- **Integration tests**: no tests that run `audit_url` end-to-end against a known HTML fixture.
- **Snapshot tests**: no tests that check human output or JSON output format.
- **Performance benchmarks**: `cargo bench` is referenced in PLAN.md §13 but no benchmarks exist in `benches/`.

## Adding tests

Place unit tests in the same file as the code under `#[cfg(test)] mod tests { ... }`.

Place integration tests in `tests/` at the crate root. They can call `use gnomon::*` via the lib crate.

For HTML fixture-based tests (the most valuable thing to add): create `tests/fixtures/` with small HTML snippets that exercise specific rules, then parse and assert on `HtmlAnalysis` or `AuditReport`.

## Running tests with output

```sh
cargo test -- --nocapture    # show println! during tests
cargo test <name>            # run a specific test by name
```
