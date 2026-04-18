# gnomon

> A **gnomon** is the blade on a sundial — the part that casts the shadow. It
> doesn't tell you the time; it reveals it.

Performance-budget auditor for static sites. A CI gate, not a dashboard. It does
not produce a score out of 100. It produces pass or fail.

See [PLAN.md](PLAN.md) for the full design and philosophy.

## Install

**Linux:**
```sh
curl -fsSL https://github.com/libliflin/gnomon/releases/latest/download/gnomon-x86_64-unknown-linux-musl.tar.gz | tar xz
sudo mv gnomon /usr/local/bin/
```

**macOS (Apple Silicon):**
```sh
curl -fsSL https://github.com/libliflin/gnomon/releases/latest/download/gnomon-aarch64-apple-darwin.tar.gz | tar xz
sudo mv gnomon /usr/local/bin/
```

**macOS (Intel):**
```sh
curl -fsSL https://github.com/libliflin/gnomon/releases/latest/download/gnomon-x86_64-apple-darwin.tar.gz | tar xz
sudo mv gnomon /usr/local/bin/
```

**With cargo-binstall:**
```sh
cargo binstall gnomon
```

**Build from source:**
```sh
cargo install gnomon
```

## CI Integration

Add gnomon to a GitHub Actions workflow using the pre-built binary:

```yaml
- name: Install gnomon
  run: |
    curl -fsSL https://github.com/libliflin/gnomon/releases/latest/download/gnomon-x86_64-unknown-linux-musl.tar.gz | tar xz
    sudo mv gnomon /usr/local/bin/
- name: Audit performance budget
  run: gnomon audit https://staging.your-site.com
```

For violations surfaced in GitHub code scanning (requires SARIF upload):

```yaml
- name: Install gnomon
  run: |
    curl -fsSL https://github.com/libliflin/gnomon/releases/latest/download/gnomon-x86_64-unknown-linux-musl.tar.gz | tar xz
    sudo mv gnomon /usr/local/bin/
- name: Audit performance budget (SARIF)
  run: gnomon audit https://staging.your-site.com --format sarif > results.sarif; true
- name: Upload SARIF to GitHub code scanning
  uses: github/codeql-action/upload-sarif@v3
  with:
    sarif_file: results.sarif
```

Exit code 1 (violations) must not abort the workflow before the upload step — the `; true` suffix ensures the SARIF file is always uploaded. Gnomon violations are page-level, not line-level, so GitHub surfaces them as annotations on the workflow run rather than inline PR diff comments — the correct behavior for a tool that audits a URL, not a source file.

## Use

```sh
gnomon audit https://example.com                     # auto-loads ./gnomon.toml; falls back to insley preset
gnomon audit https://example.com --preset mcmaster   # looser, still opinionated
gnomon audit https://example.com --format json       # machine-readable
gnomon audit https://example.com --format sarif      # SARIF 2.1.0 for GitHub code scanning
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
- Human, JSON, and SARIF 2.1.0 output (`--format sarif`)

Coming later:

- `--measure` with headless Chromium for real LCP/CLS/INP
- `--dir` for auditing a built directory before deploy
- `gnomon budget tighten` (the ratchet)
- Justifications with 90-day expiries
- GitHub Action (`libliflin/gnomon-action@v1`)
- Per-route overrides

## License

MIT OR Apache-2.0
 
