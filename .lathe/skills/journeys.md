# Stakeholder Journeys

Concrete first-encounter journeys for each stakeholder. The champion walks one of these each cycle. Steps are real — run the commands, read the output, hit the walls.

---

## Stakeholder 1: Web Performance Engineer

**Emotional signal:** Momentum — "I want to tell someone about this."

### Journey steps

1. **Install.** `cargo install gnomon` or download the binary. Check it's on PATH: `gnomon --version`.
2. **First audit.** `gnomon audit https://their-site.com`. Read all output: bytes table, counts table, third-party list (if any), anti-theater/forbidden section, PASS/FAIL verdict.
3. **Read violations.** For each violation: does the detail string name the specific resource? Is the contributor (filename and size) present? Is the detail actionable — can you tell what to fix?
4. **Dig into a specific violation.** If render_blocking fires: does it name the blocking file? If js fires: does it name the top contributors? If a theater check fires: does the detail explain *why* it matters, not just that it fired?
5. **Try JSON output.** `gnomon audit https://their-site.com --format json`. Read the structure: `violations`, `totals`, `html_analysis`. Does `html_analysis` have fields that don't appear in the violations array? (The "detected but silent" gap pattern.)
6. **Try SARIF output.** `gnomon audit https://their-site.com --format sarif`. Is the output valid SARIF 2.1.0? Does it contain the violations?
7. **Try to configure a budget.** `gnomon budget-init --preset mcmaster`. Read the generated `gnomon.toml`. Is the format clear? Are the values what you'd expect?
8. **Run audit with the config.** `gnomon audit https://their-site.com` (auto-loads `gnomon.toml`). Do violations reflect the config values?
9. **Try to understand a specific preset.** `gnomon presets`. Are the budget values readable? Do the column headers make sense?
10. **Attempt a fix and re-run.** Identify one violation, describe what you'd fix, re-run. Did the violation disappear?

**Where momentum builds:** Step 4 — a violation detail that names the specific file and byte size. The engineer can act on it immediately.

**Where momentum breaks:** Step 5 — finding `html_analysis` fields with no corresponding violation. Gnomon detected it. Gnomon said nothing. This is the worst moment for this stakeholder.

**Stretch target for ambition:** Try to audit the built directory before it's deployed (`--dir` mode, not yet implemented). Try to get real LCP/CLS numbers (`--measure`, not yet implemented). Walk far enough to hit these walls.

---

## Stakeholder 2: CI/DevOps Integrator

**Emotional signal:** Confidence — "I know what this will do."

### Journey steps

1. **Open README, CI Integration section.** Read the two-step SARIF + gate pattern. Is the reason for `|| true` explained? Is the two-run structure explained?
2. **Copy the yaml snippet.** Wire into a GitHub Actions workflow. Does the binary download step work? Does the audit step work?
3. **Trigger a PR with a violation.** Add a large JS file, or remove the viewport meta tag. Does the build fail? Does the SARIF upload succeed?
4. **Check the GitHub code scanning tab.** Are the violations present? Are they labeled correctly?
5. **Check the workflow runtime.** How long did the gnomon steps add? Is it under 1–2 seconds for the binary install + audit?
6. **Check exit codes.** `gnomon audit https://their-site.com; echo $?`. 0 on pass, 1 on violations, 2 on config/network error.
7. **Try a PR that doesn't regress anything.** Does the build pass? Is there any noise output on a clean audit?
8. **Try to understand what annotations look like in GitHub.** Are violations page-level (workflow annotations) rather than inline diff comments? Is this the correct behavior for a URL audit?

**Where confidence builds:** Step 8 — the README explains clearly why gnomon produces workflow annotations (not inline diff comments) for URL audits. The integrator expected this and confirmed it.

**Where confidence breaks:** Step 3 — the build doesn't fail when it should. Or step 8 — no annotations appear, and it's unclear why.

**Stretch target:** Try to use the `libliflin/gnomon-action@v1` GitHub Action (not yet published). Walk to the wall.

---

## Stakeholder 3: Budget Owner / Tech Lead

**Emotional signal:** Authority — "The standard is here and it's enforced."

### Journey steps

1. **Decide to adopt gnomon.** Pick a preset: `insley` (hostile defaults) or `mcmaster` (table stakes).
2. **Generate the budget file.** `gnomon budget-init --preset mcmaster`. Read `gnomon.toml`. Commit it to the repo.
3. **First audit on staging.** `gnomon audit https://staging.their-site.com`. Count the violations. Is the gap between current state and budget clear?
4. **Try to get a temporary exception.** Edit `gnomon.toml` to loosen one budget with a justification and expiry. Does gnomon accept it? Does the audit pass after the loosen?
5. **Try an invalid justification.** Set `justification = "TODO"`. Does gnomon reject it? (Not yet implemented in v0.0.2 — walk to the wall.)
6. **Explain the budget to a colleague.** Open `gnomon.toml` and describe why each limit exists. Is the configuration self-documenting enough?
7. **Run `gnomon presets` to compare preset values.** Are the differences between `insley` and `mcmaster` clear and explained?
8. **Try to run the ratchet.** `gnomon budget tighten` (not yet implemented — walk to the wall).
9. **Consider the paper trail.** If a PR raises a budget ceiling, what does the git diff look like? Is the justification visible in code review?

**Where authority builds:** Step 4 — gnomon accepts the temporary exception, the audit passes, and the paper trail is in git. The budget owner can explain to their team exactly when the exception expires.

**Where authority breaks:** Step 5 — gnomon v0.0.2 doesn't validate justification strings, so `"TODO"` passes silently. The tool claims not to negotiate, but it doesn't enforce the no-placeholder rule yet.

**Stretch target:** Try `gnomon budget tighten` after a perf win. Walk to the wall.

---

## Stakeholder 4: Open-Source Contributor

**Emotional signal:** Clarity — "I know exactly where this goes and how to test it."

### Journey steps

1. **Fork and clone the repo.**
2. **Build.** `cargo build`. Should succeed with no warnings.
3. **Test.** `cargo test`. Should pass clean.
4. **Clippy.** `cargo clippy -- -D warnings`. Should pass clean.
5. **Read README and PLAN.md.** Understand the thesis and the current scope.
6. **Read CONTRIBUTING.md.** Find the three contribution paths and the good-first-issues section.
7. **Pick a contribution.** Follow CONTRIBUTING.md's scan instruction: look for `HtmlAnalysis` fields in `src/analyze.rs` that have detection logic but no corresponding branch in `theater_violations` in `src/audit.rs`.
8. **Add a field to `HtmlAnalysis`.** Write a detection in `analyze_html`. Add tests in `src/analyze.rs`.
9. **Add a branch in `theater_violations`.** Copy `theater_violations_lazy_lcp_fires` as the test template from `src/audit.rs`.
10. **Run all tests.** `cargo test`. Should pass.
11. **Run clippy.** Should pass clean.
12. **Open a PR.** Does CI pass? Is the contribution pattern clear to reviewers?

**Where clarity builds:** Step 8–9 — the test pattern is obvious, the existing tests are dense enough to copy from, and the full loop (detection → violation → test) is testable without a network call.

**Where clarity breaks:** Step 7 — CONTRIBUTING.md's scan instruction returns no results (all fields are already covered or intentionally informational), but the list of planned checks in PLAN.md §4 is present. The contributor has to bridge from "the scan found nothing" to "here's the next thing to build" — CONTRIBUTING.md provides this bridge explicitly.

**Stretch target:** Try adding a check from PLAN.md §4 that requires detecting behavior in `<script>` content (speculation rules, hidden-until-interaction payloads). Walk into the complexity.

---

## Stakeholder 5: insley-web Author (Project Owner)

**Emotional signal:** Conviction — "This tool has a point of view, and it's right."

### Journey steps

1. **Run gnomon against a reference site.** Try McMaster-Carr (`mcmaster.com`), Gov.UK, or a site known to be fast. Does gnomon pass it under `mcmaster` preset? Does gnomon pass it under `insley`?
2. **Run gnomon against a known-bad site.** A site with Google Tag Manager, React hydration, Google Fonts. Does gnomon catch everything it should?
3. **Read the human output for both.** Does the output feel like the tool described in PLAN.md — "precision, certainty, no apology"? Or does it hedge?
4. **Check the anti-theater category.** For each trick in PLAN.md §4: is it detected by gnomon today? If not, is the gap acknowledged in code or docs?
5. **Check the forbidden list.** For each entry in PLAN.md §5: is it present in `src/forbidden.rs`? Are there entries in PLAN.md that haven't been added yet?
6. **Read the roadmap (PLAN.md §15).** Where is the project relative to v0.1? v0.2? What's the gap?
7. **Consider the insley-web integration.** When would it be appropriate to run `gnomon audit https://insley.com` as an integration test? What does gnomon need to support first?

**Where conviction builds:** A clean human output that names exactly what's wrong with a bad site. The tool sounds like itself.

**Where conviction breaks:** A good site fails on a false positive, or a bad site passes because a detection isn't wired. Either means gnomon doesn't yet mean what it says.

**Stretch target:** Run gnomon on a complex real-world site with multiple violations from different categories. Does the output tell the full story?
