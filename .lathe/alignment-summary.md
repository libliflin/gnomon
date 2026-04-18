# Alignment Summary

For the project owner. Plain-English record of the champion init decisions.

---

## Who this serves

- **Web performance engineer** — uses gnomon to find and fix perf regressions before they ship; needs specific, actionable violation output
- **CI/DevOps integrator** — wires gnomon into GitHub Actions; needs reliable exit codes, clean SARIF, and no surprises
- **Budget owner / tech lead** — sets and maintains `gnomon.toml`; needs the justification/expiry system to give them standing in planning conversations
- **Open-source contributor** — adds new checks and forbidden entries; needs clear contribution paths and testable patterns
- **insley-web author (you)** — uses gnomon as the enforcement layer for the insley-web thesis; needs the project to mean what it says and advance toward v1.0

---

## Emotional signal per stakeholder

| Stakeholder | Signal | One-line test |
|---|---|---|
| Web performance engineer | **Momentum** | Does each violation build toward "I want to tell someone"? |
| CI/DevOps integrator | **Confidence** | Did the tool behave exactly as described, without surprises? |
| Budget owner / tech lead | **Authority** | Does gnomon.toml give me standing in planning conversations? |
| Open-source contributor | **Clarity** | Did I know exactly where each change went and how to test it? |
| insley-web author | **Conviction** | Does gnomon feel like a tool that means what it says? |

---

## Key tensions

**Hostile defaults vs. contributor reach.** The strictness that makes gnomon trustworthy can make contribution feel high-friction. Resolved by: keeping CONTRIBUTING.md's contribution paths accurate and its test templates current.

**Static analysis completeness vs. false positives.** More theater checks = more coverage but more risk of flagging legitimate patterns. Resolved by: only check patterns that are *always* theater (no gray cases in gnomon's model), or build an override mechanism first.

**URL-only vs. `--dir` auditing.** Currently requires a live URL. The CI integrator and budget owner would both benefit from pre-deploy directory auditing. Resolved by: noting it as a v0.1 gap; journeys should walk to this wall.

**Byte-budget coverage vs. anti-theater coverage.** Two distinct check surfaces; cycles that improve one don't improve the other. Resolved by: stakeholder rotation — the web performance engineer drives anti-theater completion; the budget owner drives budget/detail quality.

---

## What could be wrong

**`ambition.md` is missing.** The champion references `.lathe/ambition.md` for the destination. It doesn't exist. The champion falls back to PLAN.md §15 roadmap as the destination. This works, but an explicit `ambition.md` would let the project owner state where the project is going in their own words, which is more durable than the builder reading PLAN.md. Consider creating it.

**`brand.md` is missing.** The champion references `.lathe/brand.md` for the project's voice. It was previously committed but has been deleted from the working tree. The champion falls back to inferring brand from PLAN.md's tone ("unapologetically opinionated," "hostile defaults," "precision, certainty, no apology"). Consider recreating it.

**The "detected but silent" gap pattern may persist.** Several cycles (8–12) closed gaps where `HtmlAnalysis` fields were populated but had no violation branch. The pattern could recur as new fields are added. The champion should check `HtmlAnalysis` fields vs. `theater_violations` branches each cycle as part of the contributor journey.

**Justification validation is not yet enforced.** PLAN.md §6 specifies that `"TODO"`, `"temporary"`, `"fix later"` justifications should be rejected at config load time. This is not implemented in v0.0.2. The budget owner journey hits this wall at step 5 — gnomon accepts invalid justifications silently. This is a real gap in the tool's authority.

**The contributor journey has no `--dir` or `gnomon budget tighten` paths.** Both are v0.1/v0.2 features. Walking to these walls is intentional — the champion should name these gaps when they appear.

**GitHub security notes.** Checked the CI configuration: no `pull_request_target` or `issue_comment` triggers in `ci.yml` or `release.yml`. The repo appears public. No branch protection rules were verified (requires GitHub API access). The champion should verify branch protection is enabled for main — without it, an agent could push directly to main.

**The snapshot doesn't report GitHub CI status.** `snapshot.sh` runs local `cargo` commands but doesn't check the GitHub Actions status for the current commit. The champion can't see whether the last push passed remote CI without checking GitHub directly.

---

## Ambition

**Destination:** Gnomon is the CI gate teams turn on and forget — passing `gnomon audit https://insley.com` as the living proof that enforcement is the whole game, culminating in v1.0 with `gnomon crawl`, `gnomon watch`, stable schemas, and insley-web as an integrated end-to-end test.

**Current gap(s):** No `--dir` mode (every audit needs a live URL); justification/expiry enforcement not implemented (budget owner authority is promised, not enforced); only ring one of three measurement rings is live (predicted vitals, `--measure`, and the ratchet are all v0.1–v0.4 targets still to land).

**What could be wrong:** The destination is read from PLAN.md §15 and §17, which are William's own planning documents — this is likely accurate but they may have evolved since they were written. The "v1.0 = insley.com integration" framing may be aspirational rather than a hard target; if insley-web has shifted scope, the capstone metric shifts with it. The velocity signal (consolidation after SARIF) may be lathe-induced rather than a real project tempo — lathe's cycle selection could shift this once ambition.md is in play.
