# Alignment Summary — Gnomon Customer Champion

## Who this serves

- **Web performance engineer**: installs gnomon, runs it against a URL, reads violations, decides whether to adopt it — needs momentum from the first run
- **CI integrator**: wires gnomon into a pipeline, needs exit codes that mean something, machine-readable output, and no flake
- **Team technical lead / budget owner**: owns gnomon.toml and explains violations to teammates, needs the output to carry its own authority
- **Contributor**: adds rules or fixes to the codebase, needs clarity about where things go and how to test them

## Emotional signal per stakeholder

- **Web performance engineer**: momentum — "I want to tell someone about this"
- **CI integrator**: confidence — "When it fails, it means something; when it passes, I trust it"
- **Budget owner**: authority — "When this says fail, I can defend it to my team"
- **Contributor**: clarity — "I know exactly where this goes and how to test it"

## Key tensions

- **README vs. binary**: the README describes `gnomon ci`, SARIF output, and a GitHub Action — none of which exist in v0.0.2. This is the CI integrator's dead end. Resolve toward: update docs to match reality before adding features, unless an external consumer has already been burned.
- **Hostile defaults vs. onboarding**: gnomon is intentionally unforgiving. This is not negotiable — it's the product. The fix for an overwhelming first experience is more specificity in violation messages, not softer defaults.
- **Tests vs. velocity**: the hollow test suite is a walking contradiction for a tool that sells enforcement. This tension resolves toward tests faster than it would for most projects.

## Repository security (for autonomous operation)

The lathe agent reads CI results and PR metadata from GitHub and uses them in prompts — a prompt injection attack surface.

**Findings (as of init, 2026-04-17):**
- **CI workflows**: none exist (`.github/workflows/` is absent). No `pull_request_target` or `issue_comment` triggers to worry about.
- **Branch protection**: could not be verified automatically (gh API requires auth). **Action required**: manually verify that the `main` branch has protection rules enabled (require PR reviews, require status checks).
- **Repo visibility**: could not be verified automatically. **Action required**: if the repo is public, be especially cautious about PR title/description injection into agent prompts, since any GitHub user could open a PR.

## What could be wrong

- **Missing stakeholder: the end user whose device is protected.** The end user never touches gnomon, but they are the ultimate beneficiary and the ethical center of the tool's design. The champion currently has no journey for them. This was intentional — there's no way to "use the project as them" because they don't interact with gnomon. But the champion should hold this stakeholder in mind when deciding between two otherwise-equal goals: which one actually helps the person whose bandwidth gnomon is protecting?

- **v0.0.2 is very early.** The architecture is a single flat crate. The test suite is a stub. Several documented commands don't exist. The champion's journeys will hit walls quickly. This is appropriate — the champion should name those walls, not work around them. But the journeys in `skills/journeys.md` need to be updated as the project matures; they'll go stale by cycle 5 or 6 if the tool advances significantly.

- **No `--dir` mode.** The README and PLAN.md describe auditing a local built directory (`gnomon audit --dir ./dist`), but the current `AuditArgs` only has a `url` field. The adopting engineer or CI integrator who wants to audit a built artifact before deploy will hit this gap. The champion should notice this during the CI integrator's journey.

- **Insley preset vs. mcmaster preset selection.** The champion should periodically ask: are we optimizing for the insley-tier engineer (hardest possible standard) or the mcmaster-tier engineer (serious but achievable)? The tool serves both, but the default (`insley`) is brutal for most real-world sites. The champion should notice if the first-run experience consistently produces 8+ violations and ask whether that serves or alienates the adopting engineer — without compromising the hostile-defaults philosophy.

- **The brand.md file is absent.** Goal.md instructs the champion to apply brand as a tint, but brand.md does not exist. The champion should fall back to stakeholder emotional signal rather than applying brand tint until brand.md is created (either by a future init pass or by the champion themselves once the project's voice is clearer from evidence).
