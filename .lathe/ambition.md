# Ambition

## Destination

Gnomon is the CI gate that teams turn on and forget — a binary that runs on every commit, catches performance regressions before they ship, and passes `gnomon audit https://insley.com` as the living proof that "enforcement is the whole game" (PLAN.md §1). The destination is v1.0: `gnomon crawl --url`, `gnomon watch`, stable schemas, and the insley-web reference site shipping as an integrated end-to-end test (PLAN.md §15 roadmap).

The concrete measure of arrival: a team wires gnomon into CI on day one, forgets about it for six months, and it still fails the build correctly when someone sneaks in a lazy-loaded LCP candidate. The tool earns its authority by never negotiating — and the insley-web site is the test that proves it doesn't need to.

---

## The Gap

**1. No `--dir` mode — every audit still hits the network.**
PLAN.md §15 names `gnomon audit --dir` as the v0.1 milestone. Today, every audit requires a live deployed URL. The CI integrator can't run gnomon as a pre-deploy gate; the budget owner can't test locally against a built `./dist` before pushing. This is the single largest structural gap between current v0.0.2 and the destination (from README.md "Coming later: `--dir` for auditing a built directory before deploy").

**2. The ratchet and justification enforcement don't exist yet.**
PLAN.md §15 places these at v0.2. The budget file format is designed for justification/expiry enforcement (PLAN.md §6, §10), but `alignment-summary.md` line 49 confirms: "Justification validation is not yet enforced — gnomon accepts invalid justifications silently." The budget owner's authority is a promise, not a fact. `gnomon budget tighten` doesn't exist. A team adopting gnomon today cannot get the paper trail the tool is designed to produce.

**3. Only ring one of three measurement rings is live.**
PLAN.md §7 defines three concentric rings: static analysis (ring 1, live), predicted vitals (ring 2, v0.3), measured vitals via headless Chromium (ring 3, v0.4). The tool's full detection surface — predicted LCP, critical-path walker, real INP/CLS via `--measure` — is documented but not implemented. What ships today is the fast path only.

**4. Anti-theater detection is partially wired.**
Champion.md §14 ("Think in Classes, Not Instances") names the "detected but silent" gap: fields in `HtmlAnalysis` that are populated by detection logic but have no branch in `theater_violations`. The pattern has closed partially across cycles but the structural decoupling between detection and violation pipelines persists as a failure mode for new additions.

---

## What Winning Looks Like

**`--dir` mode landing** is on-ambition. It closes the pre-deploy gap, makes the CI integrator's first encounter a binary download — no staging URL required, no network dependency. A PR that adds the `--url` wrapper around existing static analysis and calls it done is off-ambition; the static analysis must work against a directory tree directly.

**Justification enforcement going live** is on-ambition. `"TODO"` and `"temporary"` rejected at config load time, expired budgets failing the build, `gnomon budget tighten` committing the ratchet — these are the features that make the budget owner's authority real. A cycle that improves the output format of existing violation strings is off-ambition; a cycle that makes `justification_expires` actually break the build is on-ambition.

**`gnomon audit https://insley.com` returning PASS** is the capstone on-ambition moment. It is both an integration test and the proof-of-thesis the insley-web author cares about. Any cycle that widens the detection surface or tightens the audit pipeline is moving toward it. Cycles that refine already-correct violation detail strings for checks that already fire are ambiguous — they don't close the gap.

---

## Velocity Signal

Recent commits (cycles ~75–88) show SARIF landing as a real structural addition, followed by a run of test-pinning and docs fixes. The pattern in commit messages — `test: pin`, `docs: fix`, `docs: extend` — suggests the project entered a consolidation mode after a feature sprint. The insley-web author stakeholder explicitly watches for this: "does the roadmap make forward progress, or is lathe polishing v0.0.2 forever?" (champion.md §5). The destination is the answer to that question.
