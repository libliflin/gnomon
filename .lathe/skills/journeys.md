# Stakeholder Journeys — Gnomon

These are the concrete first-encounter journeys the champion walks each cycle. Each one ends with an emotional signal — the single feeling that tells you whether the moment was good, bad, or hollow.

---

## Stakeholder 1: The web performance engineer

**Who.** A developer who has been burned by Lighthouse scores that don't stop regressions. They care deeply about page weight. They've seen a 90/100 Lighthouse score on a 2MB page. They are evaluating gnomon as a replacement for Lighthouse in their CI.

**Emotional signal: momentum.** The moment after the first run, they should feel the urge to tell someone — "I found a tool that actually does what I want." Hollow is: a wall of output with no clear next action. Bad is: an error that tells them nothing.

**First 10 minutes:**
1. Find gnomon — probably via the README or the insley-web project.
2. Install: `cargo install gnomon` or download a precompiled binary.
   - *Watch for: install time, whether cargo is even installed, binary size.*
3. Run against a real URL: `gnomon audit https://their-site.com`
   - *Watch for: does it start quickly? Is there any progress indicator during the fetch?*
4. Read the output: bytes table, counts table, violations list.
   - *Watch for: are violations specific enough to act on? Do they explain why, or just that?*
5. Try to understand a violation: "CSS 47 KiB / 14 KiB (zero budget) — 33 KiB over"
   - *Watch for: which files contributed? Can they find the source of the bloat?*
6. Run with JSON output: `gnomon audit https://their-site.com --format json`
   - *Watch for: is the schema self-explanatory? Does it have what they'd need to build a dashboard?*
7. Try `gnomon presets` to understand the two presets.
   - *Watch for: are the presets legible? Do the budget numbers make sense?*
8. Look for a way to add it to CI.
   - *Watch for: `gnomon ci` command doesn't exist yet. No GitHub Action. They're on their own.*

**What to try:**
- Pick a real URL (or use example.com) and run `gnomon audit`.
- Look at the violation output as if you'd never seen the PLAN.md. Is it obvious what to fix?
- Look at the "contributor hints" in byte violations (top 2 files). Are they useful?
- Try `gnomon audit https://example.com --format json | head -40`. Is the JSON navigable?

---

## Stakeholder 2: The CI integrator

**Who.** An engineer responsible for adding gnomon to their team's pipeline. They've been asked to enforce performance budgets in CI. They are not necessarily the person who chose gnomon — they may have been handed it. They care about: exit codes that mean something, zero false positives, a fast run time, and a way to get the output into their existing tooling (GitHub Annotations, Slack notifications, etc.).

**Emotional signal: confidence.** After wiring gnomon in, they should feel "this gate is real — when it fails, it means something, and when it passes, I trust it." Hollow is: a gate that passes everything. Bad is: a gate that flakes or fails for reasons unrelated to the site's performance.

**First 10 minutes:**
1. Look for the CI integration docs.
   - *Watch for: the README mentions `gnomon ci` and a GitHub Action (`libliflin/gnomon-action@v1`), but neither exists yet. The discrepancy between the README's promises and v0.0.2's reality is the most critical gap here.*
2. Try `gnomon ci` — **this command doesn't exist.** The CLI has `audit`, `budget-init`, `presets`. No `ci` subcommand.
   - *Watch for: the error message when an unknown subcommand is used.*
3. Improvise: `gnomon audit https://their-site.com --format json; echo $?`
   - *Watch for: does exit code 1 on failure work correctly? Exit code 2 for config errors?*
4. Look for a Docker image or precompiled binary for their CI runner OS.
   - *Watch for: `cargo install gnomon` in CI is 3-5 minutes. Is there a faster path?*
5. Try to understand SARIF output (for GitHub code scanning) — **doesn't exist yet.**
6. Try `gnomon audit --help` to see all available flags.
   - *Watch for: `--format json` is there. `--sarif` is not. `--fail-on=new` is not.*

**What to try:**
- Run `gnomon audit https://example.com` and check `echo $?` for exit code.
- Verify exit code 1 on a URL that will fail (try a bloated real-world site).
- Run `gnomon audit --help` and read every flag as if you were looking for what you need.
- Note every moment where the README describes a feature that doesn't exist in the binary.

---

## Stakeholder 3: The team technical lead / budget owner

**Who.** The engineer who owns the `gnomon.toml` and is accountable for the performance standard. They explain violations to teammates. They decide when to loosen a budget and write the justification. They want to trust the tool completely — when gnomon says fail, they need to be able to defend it.

**Emotional signal: authority.** When gnomon fails a build, they should be able to say "here's why, here's the rule, here's the path to fix it" without having to read PLAN.md to their team. The violation message should carry enough weight on its own. Hollow is: a violation that says "over budget" without telling the team what pushed it over.

**First 10 minutes:**
1. Look at `gnomon presets` output to understand the numbers.
2. Run `gnomon budget-init --preset mcmaster` to generate a starter config.
   - *Watch for: does the generated file include comments explaining each budget? Does it teach or just configure?*
3. Read the generated `gnomon.toml`. Try to explain it to a hypothetical teammate.
4. Run `gnomon audit` against a real URL with the config. Read a failing violation message aloud.
   - *Watch for: does the message include enough context for a code review comment? "CSS 47 KiB over budget — main.css (38 KiB), vendor.css (9 KiB)" is good. "css: 47 KiB / 14 KiB" is not enough.*
5. Look for `gnomon budget explain <violation-id>` — **doesn't exist yet.**
6. Try to understand the justification system from `gnomon.toml` comments.
   - *Watch for: is the justification format documented in the generated file?*

**What to try:**
- Run `gnomon budget-init` and read the output file carefully.
- Run `gnomon presets` and check if the numbers are self-explanatory.
- Construct a violation message as a teammate would read it. Is it actionable?

---

## Stakeholder 4: The contributor

**Who.** A Rust developer who wants to add a check to gnomon — maybe a new anti-theater rule, a new forbidden domain, or a new violation type. They evaluate the project in the first 10 minutes and decide whether it's worth their time.

**Emotional signal: clarity.** After reading the code, they should know exactly where to put their new rule and how to test it. Hollow is: understanding the structure but having no tests to validate their change. Bad is: a codebase that fails its own quality gates (clippy errors, broken build).

**First 10 minutes:**
1. Clone the repo.
2. `cargo build` — does it build clean?
3. `cargo clippy -- -D warnings` — does it pass clean? (It does as of cycle 1 fix.)
4. `cargo test` — does it pass? (Technically yes, but there's only a sanity stub.)
5. Read `src/analyze.rs` to understand where a new HTML check would go.
   - *Watch for: is it clear how to add a new `HtmlAnalysis` field? Is the `analyze_html` function navigable?*
6. Read `src/audit.rs` to see where violations are assembled.
   - *Watch for: the violation assembly is at the bottom of `audit_url`. Is it clear how to add a new violation check?*
7. Look for a test to copy as a starting point for their new check — **none exist.**
8. Look for a `CONTRIBUTING.md` — **doesn't exist.**
9. Try to add a trivial new forbidden domain and verify it fires.

**What to try:**
- Open `src/analyze.rs` and trace through what happens to a `<script loading="lazy">` tag.
- Open `src/forbidden.rs` and try to understand how to add a new entry.
- Run `cargo test` and note the hollow green result.
- Imagine adding a new check: where would you put it? How would you test it?
