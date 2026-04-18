# Architecture — Gnomon

## Current state (v0.0.2)

Gnomon is a single crate with a flat `src/` structure:

```
src/
  main.rs        — thin entrypoint; delegates to cli.rs
  cli.rs         — clap CLI definitions (Cli, Command, AuditArgs, BudgetInitArgs, OutputFormat)
  lib.rs         — pub re-exports (AuditReport, audit_url, Budget, Preset, Violation, ViolationKind)
  audit.rs       — orchestrates a full URL audit: fetch root, analyze HTML, fetch resources, check violations
  analyze.rs     — HTML parsing via `scraper`; produces HtmlAnalysis (byte counts, resource refs, render-blocking, anti-theater signals)
  budget.rs      — preset definitions (Insley, McMaster); Budget + Preset types; TOML-configurable (partial)
  fetch.rs       — HTTP fetch via reqwest; Fetcher, Fetched, brotli recompression
  forbidden.rs   — Aho-Corasick automaton over the hardcoded forbidden list; ForbiddenMatcher
  report.rs      — human-readable and JSON output for AuditReport
  violation.rs   — Violation and ViolationKind types
```

## Planned architecture (PLAN.md §14)

The design calls for a multi-crate workspace:
```
crates/
  gnomon-core       — shared types
  gnomon-static     — HTML/CSS/JS/font/image analysis, no I/O
  gnomon-predict    — predicted vitals
  gnomon-antitheater — theater detection rules
  gnomon-measure    — headless browser (feature-gated)
  gnomon-crawl      — URL enumeration
  gnomon-report     — output formatters
  gnomon-cli        — thin CLI shell
  gnomon-data       — presets, forbidden list, bench corpora
```

This split has not happened yet. All code lives in the single `gnomon` crate. The lib.rs exists so audit logic can be tested without going through the CLI.

## Key design decisions

**Fail-closed everywhere.** Missing `gnomon.toml` fails. Unknown keys fail. Stale budgets fail. The tool has no `--warn-only` mode and never will.

**Brotli recompression.** Gnomon recomputes transfer size itself — it does not trust CDN-reported `Content-Encoding` headers. This catches the "CDN brotlis it to 4KB but inflated is 2MB" trick.

**Inline bytes count.** Inline `<style>` and `<script>` content is counted toward CSS/JS budgets respectively. You can't hide bytes by inlining them.

**Aho-Corasick for forbidden list.** The forbidden domain/script list is compiled to an automaton at startup and reused across all URL checks.

**`--dir` mode is not yet implemented.** README and PLAN.md describe auditing a local built directory, but the current CLI only supports `--url`. The `AuditArgs` struct has no `dir` field.

**No `--measure` yet.** The headless Chromium path (predicted vitals → real LCP/CLS/INP/TTFB) is planned for v0.3–v0.4 but not implemented.

**No SARIF output yet.** Planned for v0.2. Current `OutputFormat` enum has only `Human` and `Json`.

**Tests are nearly absent.** Only a `lib.rs:sanity` stub (`assert!(true)`) exists. There are no integration tests, no snapshot tests, no per-module unit tests.

## Performance constraints (PLAN.md §13)

Gnomon has its own performance budget enforced in CI:
- Cold start → first output: < 10 ms
- Static audit of one page: < 1 ms
- Full crawl of 1000-page site: < 1 s
- Peak RSS: < 30 MB
- Binary size: < 4 MB

These are not yet wired to CI (no CI workflows exist), but they inform implementation decisions.
