# Stakeholder Journeys

Concrete journeys the champion walks each cycle, one per stakeholder. These are the steps, the emotional signal, and the moments where friction or delight would show up. Current-state observations belong in the snapshot — this file covers durable journey structure.

---

## 1. CI Integrator

**Who:** DevOps engineer or senior developer adding gnomon to a GitHub Actions pipeline for a static site.

**Emotional signal:** Trust + speed. "This ran in under 1 second, told me exactly what was wrong, and I know the gate will hold."

**Journey steps:**

1. Find the repo (linked from insley-web, blog post, GitHub search).
2. Read the README — looking for: pre-built binary? Actions snippet? What does the exit code mean?
3. Copy the Actions snippet. Replace `https://staging.your-site.com` with their actual staging URL.
4. Push a branch. Wait for CI to run the new step.
5. Read the job output: violations or a clean pass?
6. If violations: understand which metric, what the budget is, what the actual is, which resource is responsible.
7. Consider SARIF mode: add the `--format sarif` step and `upload-sarif` action. Check if annotations appear in the PR.
8. Decide: ship this integration, tune the config first, or evaluate further.

**Where friction shows up:**
- Step 2: is the Linux x86_64 musl install command in the README correct and copy-pasteable? (This is the runner platform.)
- Step 3: is the URL they're pointing at accessible from a GitHub Actions runner? (Staging URLs behind a VPN are not. `--dir` mode would unblock this, but it doesn't exist yet.)
- Step 5: does it complete in under 10 seconds total?
- Step 6: does the violation detail name the specific file that's over budget, not just the category?
- Step 7: do SARIF violations show up as annotations in the GitHub PR? (They show as workflow-run annotations, not inline diff comments — this is correct behavior per PLAN.md §12.2, but a first-time user may expect inline comments.)

**Where delight shows up:**
- A clean, fast run on a well-optimized site.
- Violation output that reads: "css: 6 KiB over 14 KiB budget — vendor.css (20 KiB)" — specific enough to act on immediately.

---

## 2. Frontend Developer on a Gated Project

**Who:** Mid-level frontend developer whose team already uses gnomon. CI failed on their PR. They need to understand the violation and resolve it.

**Emotional signal:** Clarity + agency. "I know exactly what to fix and how."

**Journey steps:**

1. See CI failure on their PR. Open the Actions log or GitHub code scanning annotations.
2. Find the violation in the output: metric, actual value, budget, and (hopefully) contributing resource.
3. Run `gnomon audit <staging-url>` locally to reproduce the failure.
4. Read the violation detail in human output: e.g., "fonts: 1 over budget of 0 — serif-regular.woff2 (45 KiB)".
5. Decide: remove the font (fix the root cause) or write a justified exception in `gnomon.toml`.
6. If exception: look up `gnomon.toml` justification syntax — in README? in PLAN.md? in the generated file?
7. Edit `gnomon.toml`, re-run audit locally to confirm the violation is resolved.
8. Push and wait for CI to confirm.

**Where friction shows up:**
- Step 2: is the CI annotation actionable, or just a metric name?
- Step 3: can they reproduce locally? (Requires a staging URL they can hit from their laptop.)
- Step 4: does the violation detail name the specific file and its size?
- Step 6: justification syntax (`justification` + `justification_expires`) is in PLAN.md §8 and §10 — not in the README. A developer who looks in the README first won't find it.
- Step 7: in v0.0.2, per-route overrides and justification expiry enforcement are not implemented. The gnomon.toml `[bytes]`/`[count]` sections work, but the allowlist justification entries in PLAN.md §8 don't. This is an invisible gap — the `ConfigFile` uses `deny_unknown_fields`, so unknown keys fail. The developer may not be able to write a justification at all.

**Where delight shows up:**
- Violation detail that names the exact resource: immediate clarity about what to fix.
- Re-running locally and seeing PASS: confirmation before pushing.

---

## 3. Evaluator

**Who:** Tech lead or staff engineer deciding whether to adopt gnomon for their team. Technically sophisticated. Believes performance matters.

**Emotional signal:** Conviction + alignment. The moment of recognition: "this is as uncompromising as I am."

**Journey steps:**

1. Land on the GitHub repo.
2. Read the README opening: does the framing land? "A CI gate, not a dashboard. It does not produce a score out of 100. It produces pass or fail." — yes or no.
3. Read PLAN.md thesis (§1) and fail-closed disposition (§6): is the philosophy coherent?
4. Run `gnomon audit https://a-known-fast-site.com` — expect a pass or near-pass.
5. Run `gnomon audit https://a-known-bloated-site.com` — expect specific, real violations.
6. Run `gnomon presets` — read the insley and mcmaster preset values.
7. Check the git log and recent activity — maintained?
8. Decision: adopt, evaluate further, or reject.

**Where friction shows up:**
- Step 4-5: any site that gnomon can't reach (502, auth-required, redirect loops) is a friction point. The evaluator will attribute fetch errors to gnomon being brittle, not to the site.
- Step 4: if a well-known fast site (e.g., gov.uk) fails on something that feels wrong, the evaluator loses confidence.
- Step 6: are the insley preset values defensible on inspection? 0 JS is extreme — the evaluator needs to see it as principled, not arbitrary. PLAN.md §1 defends it; the evaluator must find that text.
- Step 7: v0.0.2 is sparse. A reader of PLAN.md who sees that `--dir`, `--measure`, `gnomon budget tighten`, etc. don't exist yet may judge the tool as vaporware.

**Where delight shows up:**
- Reading PLAN.md §6 (fail-closed disposition) and recognizing it as principled, not rude.
- Running it on a bloated site and seeing a clear list of exactly what's wrong.
- The "you cannot quietly become McKinsey's website" line.

---

## 4. Contributor

**Who:** A developer adding a new anti-theater rule or forbidden domain. Has Rust experience. Has cloned the repo.

**Emotional signal:** Momentum + confidence. "I added a rule in 30 minutes and it felt right."

**Journey steps:**

1. Read `CONTRIBUTING.md` — which file? which test template?
2. Identify the right contribution path: anti-theater rule (3-file change: `analyze.rs` + `audit.rs` + tests in both), forbidden domain (1-file change: `forbidden.rs` + test).
3. Copy the named test template exactly.
4. For anti-theater: add field to `HtmlAnalysis`, detect in `analyze_html`, branch in `theater_violations`.
5. For forbidden: add tuple to `FORBIDDEN`, write `ForbiddenMatcher::new().find(...)` test.
6. Run `cargo test` — all tests pass?
7. Run `cargo clippy -- -D warnings` — clean?
8. Open PR.

**Where friction shows up:**
- Step 1: the note about intentionally informational fields (`preload_hint_count`, `preconnect_targets`) is in `CONTRIBUTING.md` under "Good first issues" — a contributor who finds these fields via `HtmlAnalysis` may try to add violations for them before reading that note.
- Step 3: the test template names in CONTRIBUTING.md are exact — if the test was renamed or moved, the instructions break.
- Step 4: the 3-file pattern for anti-theater rules is clear in CONTRIBUTING.md. The order matters: add field → detect → fire violation. Getting the order wrong causes compile errors.
- Step 6: `cargo test` is fast (all unit tests, no network). Feedback is immediate.

**Where delight shows up:**
- `cargo test` passing after adding a rule — the test suite gives immediate confirmation.
- The code pattern being consistent enough that "where to add this" is obvious after reading one existing check.

---

## 5. Maintainer

**Who:** The author(s) building and maintaining gnomon. Ongoing relationship with the codebase.

**Emotional signal:** Coherence + progress. "The codebase is growing in the right direction."

**Journey steps (typical session):**

1. `git pull && cargo test` — is everything still passing?
2. `cargo clippy -- -D warnings` — any new warnings?
3. Read the lathe snapshot — what is the improvement loop targeting this cycle?
4. Review the cycle's PR or decide on next work.
5. Build, test, ship.

**Where friction shows up:**
- If lathe cycles are targeting low-value things (e.g., adding tests for already-tested edge cases, tweaking format strings when structural gaps exist).
- If the snapshot is drowning in raw test output instead of providing a concise signal.
- If clippy warnings accumulate and become noise.
- If the test suite has gaps that let regressions through.

**Where delight shows up:**
- A lathe cycle that adds a rule a real user would care about.
- A test that would have caught a recent bug.
- A cycle that closes a documented gap (e.g., `--dir` mode, justification enforcement).
