# Alignment Summary

For the human reviewing this init. Plain English.

---

## Who this serves

- **Web developer evaluating gnomon** — wants to know if their site meets a real performance bar; runs `gnomon audit --url` and reads the output
- **CI/CD pipeline operator** — wants a perf gate that exits 0/1 reliably and integrates with GitHub Actions; currently has no prebuilt path
- **Contributor** — wants to submit a PR; hits clippy warnings and a near-empty test suite on first checkout
- **insley-web author (William)** — uses gnomon to enforce the standard on the reference site; currently blocked from `--dir` mode and the ratchet
- **Library consumer** — uses `audit_url()` in their own Rust code; public API surface is small but usable

---

## Emotional signal per stakeholder

- Web developer evaluating: **trust** — does the output feel like a knowledgeable peer reviewed the site?
- CI/CD operator: **confidence** — the gate fires reliably and stays quiet when it should
- Contributor: **momentum** — from "I want to fix this" to "I have a working, tested PR" without hitting a wall
- insley-web author: **conviction** — using gnomon makes the standard feel real and enforced
- Library consumer: **predictability** — you don't have to think about gnomon

---

## Key tensions

- **Hostile defaults vs first-encounter survival:** Insley preset will fail nearly every real site on first run. This is by design, but makes the evaluating developer's first experience a wall of violations with no hierarchy. Resolution signal: if you're inhabiting the evaluating developer and the violations wall is unreadable, output clarity is the goal.
- **Static analysis completeness vs audit speed:** More checks extend audit time. The 50 ms fast-mode gate from PLAN.md §12.4 is a hard constraint. Resolution signal: if the champion's journey hits noticeable latency for a small site, speed wins.
- **CLI evolution vs library API stability:** Adding totals and analysis fields changes the `AuditReport` type. Safe pre-1.0 while consumers are internal. Resolution signal: check the version number and known consumers before refactoring public types.
- **`--url` vs `--dir` priority:** Most natural CI usage wants `--dir`; only `--url` exists. Resolution signal: if CI adoption or insley-web author workflow is blocked, `--dir` wins.

---

## Repository security for autonomous operation

- **Default branch protection:** Not verified. The GitHub API would need to be checked. Assumption: not protected (new repo, no CI).
- **GitHub Actions workflows:** None exist. No `.github/workflows/` directory. No `pull_request_target` or `issue_comment` triggers — no prompt injection surface via CI.
- **Repo visibility:** Public (inferred from `repository = "https://github.com/libliflin/gnomon"` in `Cargo.toml`). Lathe's goal commits will be publicly visible.
- **Recommendation:** Add branch protection and CI before running lathe in branch mode with PRs at scale.

---

## What could be wrong

- **Missing stakeholder:** If there are downstream teams or companies already using gnomon that weren't visible in the repo (no issues, no dependents on crates.io at v0.0.2), they may have been missed. The library consumer stakeholder is inferred from the `lib.rs` public surface — it may be premature.
- **Insley-web author conflated with William's role:** This stakeholder is assumed to be William. If there are co-authors of the reference site, their needs may differ.
- **Emotional signal calibration:** "Conviction" for the insley-web author may be wrong if William's actual frustration isn't about conviction but about tooling speed or correctness. Walk the journey to verify.
- **Brand.md is absent:** The champion cannot apply brand tint until `brand.md` is written from observed evidence. The project is pre-brand — the voice is visible in `PLAN.md` ("There is no easy mode. There is no gradual adoption.") but hasn't been distilled into a brand document.
- **Clippy state:** Init snapshot logged ~9 clippy diagnostics. These are real and the builder should address them. If the champion finds clippy has been fixed by cycle 2, the floor is clean.
- **No CI:** Every cycle's snapshot will report "No CI config found" until CI is added. The champion should treat this as a sustained signal for the CI/CD operator stakeholder.
