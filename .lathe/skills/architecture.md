# Architecture

## Module structure (single crate, v0.0.2)

Gnomon is a **single Rust crate** at v0.0.2. PLAN.md describes a future multi-crate workspace (`gnomon-core`, `gnomon-static`, `gnomon-predict`, etc.) but the current code is a flat `src/` directory:

| File | Responsibility | I/O |
|------|---------------|-----|
| `src/main.rs` | Entry point; dispatches to subcommands | stdout, stderr |
| `src/cli.rs` | `clap`-derived CLI types: `Cli`, `Command`, `AuditArgs`, `BudgetInitArgs`, `OutputFormat` | None |
| `src/budget.rs` | `Preset`, `ByteBudget`, `CountBudget`, `Budget`; preset values; TOML config loading (`resolve_budget`, `write_preset`, `print_presets`) | Filesystem (gnomon.toml) |
| `src/fetch.rs` | HTTP fetching (`Fetcher`, `Fetched`); brotli recompression of fetched bytes | Network |
| `src/analyze.rs` | `HtmlAnalysis`; parse HTML and extract script/stylesheet/image/font URLs; detect theater signals (lazy LCP, viewport meta, img dimensions, charset meta, preloads, render-blocking) | None |
| `src/audit.rs` | `audit_url`: orchestrates fetch + analysis + violation assembly; `AuditReport`, `Totals`; all budget checks; `theater_violations()` pure function | Calls fetch + analyze |
| `src/forbidden.rs` | `ForbiddenMatcher` (aho-corasick automaton over a static `FORBIDDEN` list) | None |
| `src/report.rs` | `print_human`, `print_json`, `print_sarif` (SARIF 2.1.0) | stdout |
| `src/violation.rs` | `Violation`, `ViolationKind` (Bytes, Count, Forbidden, Theater, FetchError) | None |
| `src/lib.rs` | Re-exports public API: `AuditReport`, `audit_url`, `Budget`, `Preset`, `Violation`, `ViolationKind` | None |

## Key architectural decisions

### Brotli recompression (load-bearing)
Gnomon recomputes brotli transfer size itself rather than trusting CDN `Content-Encoding` headers. CDNs may mis-report or not compress optimally. The `brotli` crate is used directly. All byte budget comparisons are against gnomon's own brotli-compressed sizes — this is what "brotli-recomputed, CDN not trusted" means in the presets.

### Aho-Corasick for forbidden list
The forbidden list is compiled into an Aho-Corasick automaton at startup (`ForbiddenMatcher::new()`). Substring matching against all resource URLs is O(n) in total URL bytes, not O(patterns × URLs). Every fetched URL and every HTML-declared resource URL is checked against it.

### Inline byte attribution
Inline `<style>` and `<script>` bytes count toward CSS/JS budgets. They are represented as synthetic `"(inline <style>)"` and `"(inline <script>)"` contributor strings in the top-2 violation detail lists. They do NOT count as discrete requests (excluded from request-count tallies by the `starts_with("(inline ")` check in `requests_count_detail`).

### Theater detection is a pure function
`theater_violations(&HtmlAnalysis)` in `audit.rs` takes only the parsed HTML analysis — no I/O. Adding a new theater check is: (1) add a bool/count field to `HtmlAnalysis`, (2) detect in `analyze_html`, (3) add a branch in `theater_violations`, (4) write tests in both files.

Two fields are **intentionally informational** — they appear in JSON output but have no violation branch and must not get one:
- `preload_hint_count` — counts `<link rel="preload">` and `<link rel="modulepreload">` (both are positive performance optimizations)
- `preconnect_targets` — lists `<link rel="preconnect">` and `<link rel="dns-prefetch">` targets (preconnecting to a CDN origin is a standard technique)

### HTML parsing: scraper (not yet streaming)
At v0.0.2, gnomon uses the `scraper` crate (CSS selector-based DOM traversal, builds a full DOM). PLAN.md plans to move to `lol_html` (streaming) for lower memory use. Any optimization in the HTML parsing path must account for this — the current code is DOM-based.

### Third-party domain detection: naive eTLD+1
`registrable_domain()` in `audit.rs` extracts the last two domain labels. This misclassifies `.co.uk` and similar multi-suffix TLDs. The error is in the generous direction (undercounting third parties), per code comment. A proper PSL-based lookup is a future improvement.

### Async runtime
`tokio` with `rt-multi-thread` for the URL fetch path. The `audit_url` function is `async`. `rayon` is listed as a future dependency for parallel static analysis but not yet in `Cargo.toml` at v0.0.2.

## Violation assembly order in `audit_url`

1. Byte budgets (html, css, js, images, fonts, total) — with top-2 contributor detail
2. Count budgets (requests, third_party_domains, render_blocking, fonts)
3. Forbidden list (fetched resources + HTML-declared resources + preconnect targets)
4. Anti-theater (`theater_violations` pure function)
5. Image format (JPEG/PNG/GIF vs. AVIF/WebP — from fetched resources)
6. Fetch errors

## What is NOT implemented yet (v0.0.2 vs. PLAN.md)

| PLAN.md feature | Status |
|----------------|--------|
| `gnomon audit --dir` | Not implemented — URL mode only |
| `--measure` (headless Chromium) | Not implemented |
| `gnomon budget tighten` (ratchet) | Not implemented |
| Justifications with expiries in gnomon.toml | Not implemented — `ConfigFile` accepts the TOML keys but doesn't enforce expiry logic |
| Per-route overrides | Partial — bytes/count can be overridden globally, not per-route |
| `gnomon watch` | Not implemented |
| `gnomon crawl` | Not implemented |
| `gnomon ci` shorthand | Not implemented |
| `gnomon doctor` | Not implemented |
| Forbidden list pinning (`gnomon.lock`) | Not implemented |
| Multi-crate workspace | Not structured yet |
| Predicted vitals (LCP, CLS, JS parse time) | Not implemented |
| Performance benchmarks (`cargo bench`) | Not implemented |
