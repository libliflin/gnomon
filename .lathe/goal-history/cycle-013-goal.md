# Goal — Cycle 13

## What

Add `.github/workflows/release.yml` that builds and publishes pre-built gnomon binaries for three targets when a version tag (`v*`) is pushed:

- `x86_64-unknown-linux-musl` — portable Linux, no glibc dependency; the right target for GitHub Actions runners and Docker-based CI
- `x86_64-apple-darwin` — macOS Intel
- `aarch64-apple-darwin` — macOS Apple Silicon

Each artifact is a `.tar.gz` containing the single `gnomon` binary. The workflow uploads them to the GitHub release for the tag using `softprops/action-gh-release` (or equivalent).

Update the README `## Install` section to document three paths, in order of preference:

```sh
# Fast — binary download (~5 seconds):
curl -L https://github.com/libliflin/gnomon/releases/latest/download/gnomon-x86_64-unknown-linux-musl.tar.gz | tar xz

# With cargo-binstall:
cargo binstall gnomon

# Build from source:
cargo install gnomon
```

Also add a complete GitHub Actions workflow snippet to the README showing how a consumer project integrates gnomon into their CI pipeline, using the binary download path (not `cargo install`):

```yaml
- name: Install gnomon
  run: |
    curl -fsSL https://github.com/libliflin/gnomon/releases/latest/download/gnomon-x86_64-unknown-linux-musl.tar.gz | tar xz
    sudo mv gnomon /usr/local/bin/
- name: Audit performance budget
  run: gnomon audit https://staging.your-site.com
```

The release workflow itself must pass `cargo clippy -- -D warnings` and `cargo test` as pre-release checks before building binaries, so a broken build cannot produce a published release.

## Which Stakeholder

**The CI integrator** (stakeholder 2). Unserved for 4 cycles — the longest wait. Last served cycle 9.

## Why Now

The CI integrator's journey today ends at a dead end that the README cannot resolve: there is no fast way to install gnomon in a CI runner. `cargo install gnomon` requires the Rust toolchain plus 3–4 minutes of compilation time cold. With `Swatinem/rust-cache@v2`, warm runs drop to ~50 seconds — acceptable but not fast enough for teams with sub-minute CI step budgets or CI runners without persistent caches.

**The specific step that fails:** Step 4 of the CI integrator's journey — "Look for a precompiled binary for their CI runner OS." There are no GitHub releases. `cargo-binstall` finds nothing. The only documented path is `cargo install gnomon`, which is a 4-minute tax on every fresh CI runner.

The confidence signal for the CI integrator is "when this fails, it means something; when it passes, I trust it." A gate that costs 4 minutes is a gate teams disable. The confidence never gets a chance to build — the setup overhead is the rejection event.

**This is the structural gap.** Exit codes work. The JSON output is machine-readable. The violation details are specific and actionable. gnomon.toml auto-discovery works. The only thing standing between the CI integrator and a fully wired pipeline is the install step — and that install step is a known and unnecessary 4-minute ceiling.

A release workflow is the structural fix: it creates the binary path that every downstream CI workflow can use, including the future GitHub Action that's in "Coming Later." The binary release is the prerequisite for everything that follows — `cargo-binstall`, `libliflin/gnomon-action@v1`, Docker images. Writing the release workflow now unlocks all of those future paths.

**Brand check:** Gnomon's install story should match its output story. The output is precise and fast. A 4-minute install is not precise or fast. The binary download path is: `curl | tar xz | done`. That's gnomon's pace.

## Lived-Experience Note

*I became the CI integrator. I'd been handed a ticket: "Add gnomon to our CI pipeline — it should block PRs that regress page weight." I opened the gnomon README.*

*The README was clean. Exit codes documented. The auto-discovery workflow made sense. The "Coming Later" section was honest — no ghost features. I saw the CI example: `gnomon audit https://staging.my-site.com`. I started writing `.github/workflows/perf.yml`.*

*Step 1: I need to get the gnomon binary into my CI runner. I look at the README install section. `cargo install gnomon`. That's it. One path.*

*I know what `cargo install` costs in CI: the Rust toolchain download (handled by `dtolnay/rust-toolchain@stable`, ~30s), then gnomon plus all its dependencies compiled from scratch. Four minutes cold. With `Swatinem/rust-cache@v2` and a warm cache, maybe 50 seconds. But cache warmth is never guaranteed on first run, on new runner types, or after dependency changes.*

*I searched for GitHub releases. No releases — no binaries. I tried `cargo binstall gnomon` mentally — same result. Nothing pre-built.*

*The worst moment was typing `cargo install gnomon` into the workflow and knowing I was asking every developer on my team to wait an extra 30–240 seconds on every PR, for a tool that audits in under 200ms. That gap between the audit time and the install overhead is embarrassing.*

*The confidence signal — "when this fails, it means something" — never got a chance to land. The setup friction was the rejection event. Three developers pushed back in review: "Why does the perf check take 3 minutes?" I didn't have a good answer.*

*A binary download would cost 3 seconds. The audit itself costs 150ms. That's the tool's actual speed. The release workflow makes that speed visible from the first install.*
