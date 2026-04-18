# gnomon

> A **gnomon** is the blade on a sundial — the part that casts the shadow. It
> doesn't tell you the time; it reveals it.

Performance-budget auditor for static sites. A CI gate, not a dashboard. It does
not produce a score out of 100. It produces pass or fail.

See [PLAN.md](PLAN.md) for the full design and philosophy.

## Install

```sh
cargo install gnomon
```

## Use

```sh
gnomon audit https://example.com                     # auto-loads ./gnomon.toml; falls back to insley preset
gnomon audit https://example.com --preset mcmaster   # looser, still opinionated
gnomon audit https://example.com --format json       # for CI
gnomon audit https://example.com --config path/to/gnomon.toml  # explicit config path
gnomon presets                                        # print the built-in budgets
gnomon budget-init --preset mcmaster                  # write a starter gnomon.toml
```

When `gnomon.toml` exists in the working directory, it is loaded automatically — no `--config` flag required. The common CI workflow is:

```sh
gnomon budget-init --preset mcmaster   # once: commit gnomon.toml to the repo
gnomon audit https://staging.your-site.com  # CI: picks up the committed config
```

Exit codes:

- `0` — all budgets met
- `1` — one or more violations
- `2` — configuration or network error

## v0.0.2 scope

Working today:

- URL audit with static analysis
- Two presets (`insley`, `mcmaster`)
- Byte budgets for HTML/CSS/JS/images/fonts (brotli-recomputed, CDN not trusted)
- Count budgets for requests, third-party domains, render-blocking, fonts
- Forbidden-list enforcement (GTM, Adobe DTM, Meta pixel, Segment, Intercom,
  session-replay tools, Google ads, DoubleClick, polyfill.io, etc.)
- Anti-theater detection (lazy-loaded LCP candidate, preload-as-stylesheet trick,
  missing viewport)
- Human + JSON output

Coming later:

- `--measure` with headless Chromium for real LCP/CLS/INP
- `--dir` for auditing a built directory before deploy
- `gnomon budget tighten` (the ratchet)
- Justifications with 90-day expiries
- SARIF output and GitHub Action
- Per-route overrides

## License

MIT OR Apache-2.0
 
