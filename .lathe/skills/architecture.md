# Architecture

## Current structure (v0.0.2)

Gnomon is a single Cargo crate at this point — the multi-crate workspace described in PLAN.md §14 is planned but not yet implemented. Everything lives in `src/`.

```
src/
  lib.rs         — public API surface; re-exports audit_url, AuditReport, Budget, Preset, Violation, ViolationKind
  main.rs        — thin CLI entry point; builds tokio runtime, dispatches to subcommands
  cli.rs         — clap definitions: Cli, Command, AuditArgs, BudgetInitArgs, OutputFormat
  audit.rs       — core audit loop: fetches root HTML, analyzes, fetches sub-resources, computes totals, assembles violations
  analyze.rs     — HTML analysis: parses HTML with scraper, extracts script/style/image/font URLs, detects anti-theater patterns
  budget.rs      — Budget and Preset types; preset definitions (insley, mcmaster); TOML config loading; preset TOML generation
  fetch.rs       — HTTP client (reqwest); Fetcher for root fetch + concurrent sub-resource fetch; brotli recompression
  forbidden.rs   — FORBIDDEN blocklist (Aho-Corasick automaton); ForbiddenMatcher
  report.rs      — human-readable output (owo-colors); JSON output (serde_json)
  violation.rs   — Violation and ViolationKind types
```

## Key data flow

```
CLI args → resolve_budget() → audit_url()
                                  ↓
                        fetch_root() → HTML body
                                  ↓
                        analyze_html() → HtmlAnalysis
                                  ↓
                        fetch_many(sub-resources) → Vec<Fetched>
                                  ↓
                        aggregate Totals
                                  ↓
                        check violations (bytes, counts, forbidden, theater)
                                  ↓
                        AuditReport → print_human / print_json
```

## Where checks live

**Anti-theater rules** — currently in `audit.rs` (the `lazy_lcp_candidate` and `viewport_meta` checks). The HTML analyzer (`analyze.rs`) sets the flags; `audit.rs` converts them to violations.

**Forbidden list** — `forbidden.rs` builds the Aho-Corasick automaton at call time. Checked against every fetched URL and every HTML-declared resource URL in `audit.rs`.

**Byte budgets and count budgets** — inline in `audit.rs` via `bytes_check` and `count_check` closures.

## What's not yet implemented (as of v0.0.2)

- `--dir` mode: audit a local directory of built assets without a server
- `gnomon ci` subcommand
- SARIF output
- Predicted vitals (LCP, CLS, JS parse time)
- `--measure` (headless browser, chromiumoxide)
- `budget tighten` (ratchet)
- `budget update` (refresh pinned forbidden list)
- `budget explain <id>`
- `gnomon diff` (compare against base ref)
- `gnomon watch`
- `gnomon crawl`
- Per-route config in gnomon.toml (the TOML parser exists but routes/justifications/expiries are not parsed)
- SARIF schema for violations
- gnomon.lock for forbidden list pinning

## Public API surface

Exported from `lib.rs`:
- `audit_url(url: &str, budget: &Budget) -> anyhow::Result<AuditReport>` — async
- `AuditReport` — full audit result with url, violations, totals, html, resources, html_analysis
- `Budget` — holds a single `Preset`; constructed via `Budget::from_preset(PresetName)` or `resolve_budget()`
- `Preset` — name + ByteBudget + CountBudget
- `Violation` — kind, metric, budget, actual, detail
- `ViolationKind` — Bytes | Count | Forbidden | Theater | FetchError

**Not exported (internal):**
- `HtmlAnalysis`, `ResourceRef`, `AssetKind` — from analyze.rs
- `Fetched`, `Fetcher` — from fetch.rs
- `ForbiddenMatcher`, `FORBIDDEN` — from forbidden.rs
- `Totals` — from audit.rs (embedded in AuditReport but not separately re-exported)

## Notable implementation choices

- **Brotli recompression:** gnomon always recomputes brotli size from the raw body. CDN content-length headers are not trusted. Quality 5 (fast, not optimal) — the goal is a consistent conservative bound.
- **Third-party detection:** naive eTLD+1 (registrable domain = last two labels). Known limitation: undercounts on multi-suffix TLDs (`.co.uk`). Error direction is generous-to-site, which is by design.
- **Forbidden list:** built as an Aho-Corasick automaton at each call to `ForbiddenMatcher::new()`. Should eventually be lazily initialized at program start.
- **HTML parsing:** uses `scraper` (CSS selector based). PLAN.md mentions `lol_html` for streaming mode — not yet adopted.
- **Render-blocking detection:** approximate. A `<script>` is render-blocking if it's in `<head>` and has no `async` or `defer`. A preload-as-style is flagged as render-blocking (anti-theater rule).
