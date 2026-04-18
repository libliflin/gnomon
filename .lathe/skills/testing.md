# Testing

## Test runner

`cargo test` — runs all unit tests in-process. Tests live in `#[cfg(test)]` modules at the bottom of each source file. No separate test binary or integration test directory.

## Commands

```sh
cargo test                     # run all tests
cargo clippy -- -D warnings    # must pass clean; CI enforces this
cargo check --workspace --all-targets  # type-check without building
```

## What is tested (by file)

- **`src/audit.rs`**: `bytes_check` (boundary at budget, one byte over, zero budget, contributor lists, inline synthetic entries), `render_blocking_detail`, `third_party_domains_detail`, `fonts_count_detail`, `requests_count_detail` (inline exclusion, zero omission, tie-breaking), `theater_violations` (lazy LCP, missing viewport, img dimensions, charset meta — all four individually and combined), `count_legacy_images` (JPEG/PNG/GIF, WebP/AVIF, fetch errors, missing Content-Type, case-insensitivity, MIME parameters), `img_format_violation` (fields, None branches), `AuditReport` fields (pass/preset).
- **`src/budget.rs`**: `preset_toml` output format (header, KiB comments, zero-enforcement comments, count comments), `resolve_budget` (auto-discovery, explicit override, no-file fallback, malformed TOML), round-trip parse of generated TOML.
- **`src/report.rs`**: `build_sarif` (schema fields, empty violations, single violation mapping, ruleId format, duplicate kind/metric deduplication).
- **`src/analyze.rs`**: HTML analysis detection (inferred from CONTRIBUTING.md templates; fields on `HtmlAnalysis` are tested in `src/audit.rs` via `theater_violations`).
- **`src/forbidden.rs`**: `ForbiddenMatcher::find` (inferred from CONTRIBUTING.md; each forbidden entry has a test).

## Key patterns

**Boundary tests.** "Exactly at budget" and "one byte over" are both tested — pins the `actual > budget` semantics (not `>=`). Every budget check has a boundary test.

**Top-2 contributor tests.** Violation detail strings are pinned at the 2-contributor limit. Tests cover: only one contributor, exactly two, three with the smallest excluded, and the case where inline bytes rank in the top two.

**Inline synthetic entries.** `(inline <style>)` and `(inline <script>)` are tested as byte-budget contributors and explicitly excluded from request-count tallies (they're not discrete network requests).

**CWD tests in `src/budget.rs`.** Tests that change the working directory to test auto-discovery of `gnomon.toml` use a `static CWD_LOCK: Mutex<()>` to serialize CWD-touching tests and a `CwdGuard` struct (Drop restores original CWD) for panic safety. Always: acquire the lock, then construct the guard. Both must be present.

**Theater test template.** From `CONTRIBUTING.md`: copy `theater_violations_img_missing_dimensions_fires` as the violation test template; copy `img_missing_both_dimensions_is_counted` in `src/analyze.rs` as the detection template.

**Forbidden test template.** From `CONTRIBUTING.md`: copy `multiple_distinct_entries_each_match` in `src/forbidden.rs`. Call `ForbiddenMatcher::new().find("https://your-domain.com/script.js")` and assert `Some(("your-domain.com", "your reason"))`.

## CI

Three jobs in `.github/workflows/ci.yml`, each on `ubuntu-latest`:
1. `cargo check --workspace --all-targets`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace`

All three run on every `pull_request` and `push` to `main`. All must pass.

## What is NOT tested yet

- Integration tests against a live URL (all tests are pure unit tests of internal functions).
- Performance/timing benchmarks (`cargo bench` is planned in PLAN.md §13 but not present yet).
- The full `audit_url` async function end-to-end.
