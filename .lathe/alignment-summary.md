# Alignment Summary

Plain-English summary of alignment decisions for the user — not for the runtime agent.

---

## Who this serves

- **CI integrator**: a developer or DevOps engineer adding gnomon to a GitHub Actions pipeline to gate a static site's performance. Primary goal: get it running, trust the output, know the gate will hold.
- **Frontend developer on a gated project**: someone whose team already uses gnomon, hit a CI failure, and needs to understand the violation and resolve it (fix or justify). Primary goal: clarity on what to fix and how.
- **Evaluator**: a tech lead or staff engineer deciding whether to adopt gnomon for their team. Primary goal: validate that gnomon is serious and philosophically aligned with their values before committing.
- **Contributor**: a developer adding a new anti-theater rule or forbidden domain upstream. Primary goal: add a rule in under an hour with confidence.
- **Maintainer**: the author(s) building gnomon. Primary goal: coherent code, meaningful tests, a trustworthy CI gate, and lathe making real progress.

---

## Emotional signal per stakeholder

- **CI integrator**: trust + speed — "this ran in under 1 second, told me exactly what was wrong, and I know the gate will hold"
- **Frontend developer**: clarity + agency — "I know exactly what to fix and how"
- **Evaluator**: conviction + alignment — "this is as uncompromising as I am"
- **Contributor**: momentum + confidence — "I added a rule in 30 minutes and it felt right"
- **Maintainer**: coherence + progress — "the codebase is growing in the right direction"

---

## Key tensions

**Hostile defaults vs. first adoption.** The insley preset (0 JS, 0 fonts, 100 KB total) is extreme by design. But if the first audit run produces 15 confusing violations on a reasonable site, the evaluator or integrator walks away before experiencing the value. Tension: enforcement authority vs. adoption friction.

**URL mode vs. directory mode.** The tool requires a live URL. The canonical CI use case — audit the build artifact before deploy — needs `--dir`. Many CI environments don't expose staging URLs from PR builds. This is currently the most likely blocker for the CI integrator.

**Claims vs. implementation (v0.0.2 vs. PLAN.md).** PLAN.md describes a complete system: justifications with expiries, per-route overrides, ratchet, `gnomon ci`, `gnomon watch`. None of these are implemented yet. A first-time user who reads PLAN.md expecting a complete system will hit gaps. The `ConfigFile` uses `deny_unknown_fields`, so if a user writes a justification entry as described in PLAN.md §8, it will fail at parse time with an "unknown key" error — which is confusing because the docs imply it should work.

**Strictness vs. contributor confidence.** The insley preset is strict enough that a new contributor adding a rule needs to understand the philosophy to know if their rule fits. Two `HtmlAnalysis` fields (`preload_hint_count`, `preconnect_targets`) are intentionally informational and must not get violation branches — CONTRIBUTING.md explains this, but a contributor who finds them by scanning `HtmlAnalysis` may try to add violations before reading that note.

---

## What could be wrong

**Missing stakeholder?** There may be a "library consumer" stakeholder — someone importing `gnomon` as a Rust library (`use gnomon::audit_url`). `lib.rs` re-exports the public API. No documentation targets this use case and no examples exist in the repo. If library consumers show up, the playbook needs a new stakeholder entry with its own journey.

**Evaluator emotional signal may be off.** "Conviction + alignment" assumes the evaluator already believes in strict performance enforcement. A skeptical evaluator ("prove to me this is worth the friction") needs a different signal — closer to "evidence + fairness." If evaluators frequently evaluate and don't adopt, this assumption is worth revisiting.

**CI integrator may not have a staging URL.** The current tool requires `gnomon audit <url>`. In many CI setups — especially for organizations that don't maintain a persistent staging environment — there is no URL to point at. The `--dir` mode is planned (PLAN.md v0.1 roadmap) but not implemented. This may be the single largest adoption blocker for the CI integrator stakeholder.

**Repository security check.** The CI workflow uses `pull_request` (not `pull_request_target`) and has no `issue_comment` triggers — not a prompt injection risk for the lathe loop. The repo is public (github.com/libliflin/gnomon). Branch protection on `main` is a GitHub setting not visible in repo files; it should be verified in GitHub repository settings. If the default branch is not protected, lathe cycles that commit directly to main could theoretically be disrupted by an external PR.

**PLAN.md sets high expectations.** The document is polished and describes a complete, sophisticated system. A reader who skims it may believe all features are implemented. The README's "v0.0.2 scope" section lists what's working vs. coming later — this is the right approach, but it requires the reader to notice and read it. The champion should watch for cases where the evaluator's journey stalls because they tried a described feature that doesn't exist yet.
