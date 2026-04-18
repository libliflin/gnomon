# Contributing

## Dev commands

```sh
cargo build                    # debug build
cargo test                     # run all tests
cargo clippy -- -D warnings    # must pass clean before any PR
```

All three must pass before you push. The CI gate enforces them.

## Three check types

**Anti-theater checks** — `src/analyze.rs` detects, `theater_violations()` in `src/audit.rs` fires the violation.
Pattern: add a bool/count field to `HtmlAnalysis`, wire detection in `analyze_html`, add a branch in `theater_violations`, add tests in both files.

**Forbidden-list checks** — `src/forbidden.rs`. Add a `("pattern", "reason")` tuple to `FORBIDDEN`. Pattern is a substring matched against the full resource URL, case-insensitive.

**Budget checks** — `src/audit.rs`. `bytes_check` and the count detail helpers. Budget values come from `gnomon.toml` or a preset in `src/budget.rs`.

## Contribution paths

### 1. New anti-theater rule

Copy `theater_violations_img_missing_dimensions_fires` in `src/audit.rs` as your test template.
Copy `img_missing_both_dimensions_is_counted` in `src/analyze.rs` as your detection template.

Steps: add field to `HtmlAnalysis` → detect in `analyze_html` → branch in `theater_violations` → tests in both files.

### 2. New forbidden domain

Copy `multiple_distinct_entries_each_match` in `src/forbidden.rs` as your test template.
Add a tuple to `FORBIDDEN` in `src/forbidden.rs`. Add a test that calls `ForbiddenMatcher::new().find("https://your-domain.com/script.js")` and asserts `Some(("your-domain.com", "your reason"))`.

### 3. New count violation detail

Copy `render_blocking_detail_single_name` in `src/audit.rs` as your test template.
The detail function pattern: takes `actual`, `budget`, and a contributor slice; returns a formatted string. Wire it in the violation assembly block below `theater_violations`.

## Good first issues

To find one: scan `HtmlAnalysis` fields in `src/analyze.rs` for fields that are detected and serialized to JSON but have no corresponding branch in `theater_violations` in `src/audit.rs`. Each such field is a candidate anti-theater rule waiting to be wired.

Two fields are **intentionally informational** — they are tracked in the JSON output for tooling consumers but are not theater candidates and must not get violation branches:

- **`preload_hint_count`** — counts `<link rel="preload">` and `<link rel="modulepreload">` tags (detection uses `rel.contains("preload")`). Both are positive performance optimizations. A violation saying "you used preloads" would fail any page doing the right thing.
- **`preconnect_targets`** — lists `<link rel="preconnect">` and `<link rel="dns-prefetch">` targets. Preconnecting to a CDN origin is a standard performance technique. Presence is not evidence of theater.

If the discovery algorithm points you to either of these fields, this is the explanation.
