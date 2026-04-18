# Champion Changelog — Cycle 16

## Stakeholder

The web performance engineer (stakeholder 1). Last served cycle 12 — four cycles ago, the longest wait in the current rotation.

## Journey Walked

1. Floor check: build clean, 109 tests passing, clippy clean. No floor issues.
2. `gnomon audit https://example.com --format json` — PASS, no violations.
3. Scanned `html_analysis` in JSON. Found `has_charset_meta: false`. Checked violations array — empty.
4. CONTRIBUTING.md (cycle 15) documents `has_charset_meta` as "a known gap / good first issue."
5. The gap is still open. No violation fires for missing charset meta.

## Worst Moment

Step 3. `has_charset_meta: false` is in the JSON. The violations array is empty. Gnomon detected a render-critical encoding-sniff risk and said nothing. PLAN.md §7.1 lists it. CONTRIBUTING.md calls it a known gap. The violations array and the `html_analysis` object tell different stories. A web performance engineer cannot build a dashboard on violations if violations don't reflect everything gnomon knows.

## Goal Set

Add `!analysis.has_charset_meta` branch to `theater_violations`. Fire `Theater/charset_meta` with detail `"missing <meta charset> or http-equiv Content-Type — forces encoding sniff"`. Add two tests. Update CONTRIBUTING.md's good first issues section — `has_charset_meta` is no longer a gap once this lands.

## Why This Goal, Not Another

- Web perf engineer is the most under-served stakeholder (4 cycles).
- Same class as cycle 12 (`img_missing_dimensions`): detected, tested, serialized, never fires.
- `has_charset_meta` is the last field in `HtmlAnalysis` with detection AND PLAN.md §7.1 backing AND no violation branch.
- The gap is now doubly visible: CONTRIBUTING.md says "known gap" — the JSON confirms it; the violations array denies it. Trust failure.

---

# Champion Changelog — Cycle 13

## Stakeholder

The CI integrator (stakeholder 2). Most under-served — 4 cycles since last served (cycle 9). The last 4 cycles served: WPE (12), contributor (11), budget owner (10), CI integrator (9).

## Journey Walked

1. Read README — clean and honest. Exit codes documented. The CI workflow shows `gnomon audit https://staging.your-site.com`. "SARIF output and GitHub Action" correctly listed as Coming Later.
2. Opened a new `.github/workflows/perf.yml` for a hypothetical consumer project.
3. Needed to install gnomon in a CI runner. Looked at the README install section: `cargo install gnomon`. That's the only path.
4. Searched for GitHub releases. **Nothing.** No pre-built binaries.
5. Considered `cargo-binstall gnomon`. Same result — no binaries to find.
6. Ran `gnomon audit https://example.com` — exit 0, output clean, 148ms. Verified `--format json`, exit code 1 on violations.
7. Typed `cargo install gnomon` into the workflow file, knowing it costs 3-5 minutes cold.

## Worst Moment

Step 4. No GitHub releases. No binary download option. `cargo install gnomon` is the wall — not a missing command, not a ghost feature — just a 4-minute tax on a tool that itself runs in 150ms. Three teammates in CI review ask: "Why does the perf check take 3 minutes?" The audit time and the install overhead are in different orders of magnitude. The CI integrator cannot explain this.

The confidence signal — "when this fails, it means something; when it passes, I trust it" — never got to land. The gate's setup cost was the rejection event.

## Goal Set

Add `.github/workflows/release.yml` that builds and publishes pre-built binaries (`x86_64-unknown-linux-musl`, `x86_64-apple-darwin`, `aarch64-apple-darwin`) on version tags. Update the README to document the binary download as the primary install path and include a complete GitHub Actions workflow snippet showing a consumer project integrating gnomon via the fast path.

## Why This Goal

- CI integrator is most under-served (4 cycles).
- Their remaining pain is not ghost features or wrong exit codes — the README is honest, behavior is correct. The only gap is install overhead.
- A 4-minute install is a gate teams disable. A 5-second install is invisible. The gap is a release workflow.
- This is the structural fix: the binary release is the prerequisite for `cargo-binstall`, for the future GitHub Action, for Docker images. Everything downstream of "fast install" is unblocked by this one workflow file.
- Brand check: gnomon's output is precise and fast. The install should be too.
