# Architecture

Key architectural decisions visible in the code. Use this to understand where changes belong and why certain structures exist.

---

## Current structure (v0.0.2)

Gnomon is a **single Rust crate** with a library (`src/lib.rs`) and a thin CLI binary (`src/main.rs`). PLAN.md §14 sketches a multi-crate workspace; the current code is a monolith that approximates those roles within a single crate.

```
src/
  main.rs          — CLI entry point, thin: parses args, calls lib functions, exits
  cli.rs           — clap-derived CLI definitions
  lib.rs           — re-exports: AuditReport, audit_url, Budget, Preset, Violation, ViolationKind
  analyze.rs       — HtmlAnalysis: parses HTML, extracts resources, detects anti-theater signals
  audit.rs         — audit_url: orchestrates fetch → analyze → check → report; violation assembly
  budget.rs        — Budget, Preset: hardcoded insley/mcmaster presets; future: gnomon.toml parsing
  fetch.rs         — Fetcher, Fetched: HTTP fetching with brotli recompression
  forbidden.rs     — ForbiddenMatcher: Aho-Corasick automaton over the forbidden list
  report.rs        — print_human: colored terminal output; print_json, print_sarif: structured output
  violation.rs     — Violation, ViolationKind: the output type from any check
```

---

## The detection pipeline

The key seam: **detection** (does the pattern exist?) is separate from **violation assembly** (should we complain?).

```
analyze_html(html, base) → HtmlAnalysis   ← detect: fields, counts, booleans
                                ↓
theater_violations(&analysis) → Vec<Violation>   ← assemble: pure function, no I/O
audit_url(...)    calls both, plus bytes_check, forbidden matching
```

`HtmlAnalysis` is the bridge. Adding a new check:
1. Add a field to `HtmlAnalysis` (with `Default` via `#[derive(Default)]`)
2. Populate it in `analyze_html` with detection logic
3. Branch in `theater_violations` (for anti-theater) or the count/bytes block in `audit_url` (for budget checks)
4. Tests: `analyze.rs` for detection, `audit.rs` for violation assembly

The "detected but silent" gap pattern: a field is populated by step 2 but step 3 is missing. This has been the source of multiple cycles' improvements (img_missing_dimensions, etc.).

---

## `theater_violations` — the extracted pure function

`theater_violations(&HtmlAnalysis) -> Vec<Violation>` in `src/audit.rs`. Extracted in cycle 11 to make theater checks testable without a network call.

This is the right place for anti-theater checks (pattern present in HTML → categorical violation). Budget checks (actual > budget) live in the `audit_url` body. The distinction:
- Anti-theater: "this pattern is present" → violation (no budget threshold)
- Budget: "this metric exceeds a threshold" → violation (with budget value and actual)

---

## `HtmlAnalysis` serialization

`HtmlAnalysis` is `#[derive(Serialize)]`. It appears in the JSON output as `html_analysis`. Fields added here are automatically visible in `--format json` output. This is load-bearing for the CI integrator who pipes JSON to tooling.

---

## The brotli recompression model

Gnomon fetches resources and recompresses them with brotli to compute `brotli_bytes`. It does not trust the CDN's `Content-Encoding` headers. This is how it enforces byte budgets consistently regardless of CDN configuration. `fetch.rs` handles this.

---

## The forbidden list

`src/forbidden.rs` contains a `FORBIDDEN` static array of `(&str, &str)` tuples (pattern, reason). At runtime, `ForbiddenMatcher::new()` compiles these into an Aho-Corasick automaton. Pattern matching is substring + case-insensitive against full resource URLs.

PLAN.md envisions a pinned external `gnomon-data/forbidden.toml` with SHA-256 locking. Currently: in-tree static array. Adding new entries: append to `FORBIDDEN`, add a test calling `ForbiddenMatcher::new().find(url)`.

---

## Violation types

Four `ViolationKind` variants in `src/violation.rs`:
- `Bytes` — actual > budget in bytes
- `Count` — actual > budget in count
- `Theater` — anti-theater pattern detected
- `Forbidden` — URL matches forbidden list

`FetchError` is a fifth case handled inline in `audit_url` (not a `ViolationKind`).

---

## The SARIF output

`src/report.rs` produces SARIF 2.1.0. Violations are page-level (against a URL, not a line in a source file), so they appear as workflow annotations in GitHub code scanning, not inline diff comments. This is documented as correct behavior in the README.

---

## What's not yet implemented

From PLAN.md, currently absent from the code:
- `--dir` mode (audit built directory without fetching)
- `gnomon budget tighten` (the ratchet)
- Justifications with expiries in `gnomon.toml` (validation at config load time)
- `--measure` (headless Chromium via chromiumoxide)
- Predicted vitals (predicted LCP, CLS, JS parse time)
- `gnomon crawl` (URL enumeration)
- `gnomon watch` (file-change re-audit)
- `libliflin/gnomon-action@v1` (GitHub Action)
- Multi-crate workspace (currently a monolith)
- `gnomon.lock` for pinning the forbidden list
