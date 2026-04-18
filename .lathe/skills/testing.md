# Testing

How gnomon tests. Use this to understand patterns before adding new tests.

---

## Test runner

`cargo test --workspace`. All tests are inline in the source files, in `#[cfg(test)]` modules at the bottom of each file. No separate test directory.

---

## Test locations and what they cover

**`src/analyze.rs`** — Unit tests for `analyze_html` and `classify`. Test input: raw HTML strings. Test output: `HtmlAnalysis` fields. No I/O, no network. Pattern: construct HTML string, call `analyze()` helper, assert field values.

```rust
fn analyze(html: &str) -> HtmlAnalysis {
    let base = url::Url::parse("https://example.com/").unwrap();
    analyze_html(html, &base)
}
```

This is where detection-layer tests live. ~30+ tests grouped by check (lazy_lcp, render_blocking, inline_style_bytes, etc.).

**`src/audit.rs`** — Unit tests for helper functions and violation assembly. Two layers:
1. **Helper function tests**: `url_filename`, `registrable_domain`, `render_blocking_detail`, `third_party_domains_detail`, `fonts_count_detail`, `requests_count_detail`, `bytes_check`, `count_legacy_images`, `img_format_violation`. All pure, no I/O.
2. **Theater violation tests**: `theater_violations_*` tests call `theater_violations(&analysis)` directly with a constructed `HtmlAnalysis`. No network calls. These are the test pattern for new anti-theater checks.

**`src/forbidden.rs`** — Tests for `ForbiddenMatcher`. Pattern: `ForbiddenMatcher::new().find("https://domain.com/path")` returns `Some((pattern, reason))` or `None`.

---

## Adding a new anti-theater check (the full test loop)

1. **Detection test in `analyze.rs`:** Add a test in the `#[cfg(test)]` module using the `analyze(html)` helper. Assert the new `HtmlAnalysis` field value. Template: copy `img_missing_both_dimensions_is_counted`.

2. **Violation test in `audit.rs`:** Add a test in the `theater_violations_*` group. Construct `HtmlAnalysis { new_field: value, has_viewport_meta: true, has_charset_meta: true, ..Default::default() }`. Call `theater_violations(&analysis)`. Assert violation count, metric name, kind, detail string. Template: copy `theater_violations_lazy_lcp_fires`.

This two-test pattern is the full loop. No mocking, no network, no external fixtures.

---

## What's NOT tested (intentionally)

- `audit_url` is an async function that fetches from a real URL. It has no integration tests in the test suite — by design. Testing against the live network is not how gnomon tests itself.
- The human output format (`print_human`) has no snapshot tests. The output format is stable enough that the detail strings in violation tests serve as the indirect pins.
- Performance benchmarks (`cargo bench`) are referenced in PLAN.md §13 but not yet wired.

---

## Key detail string conventions

Violation detail strings are pinned by tests. When adding a check, pin the exact detail string:

- Byte violations: `"X over Y budget — contributor1.ext (N KiB), contributor2.ext (N KiB)"`
- Count violations: `"N over budget of B — name1, name2"`
- Theater violations (bool): `"description of pattern — consequence for user"`
- Theater violations (count): `"N element(s) description — consequence for user"`
- Forbidden: `"pattern — reason"`

The detail string convention is: name the pattern, then name why it matters to the user. No hedging.
