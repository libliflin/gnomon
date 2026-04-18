# Brand

**Identity.** Plainspoken enforcer with a craftsman's disposition. Short sentences, specific numbers, named culprits. Convictions are stated once and not apologized for — the strictness is the product, not a side effect. From `PLAN.md`: "It is unapologetically opinionated. It ships hostile defaults. It fails closed." From `cli.rs:11`: `about = "Performance budget auditor. A CI gate, not a dashboard."` From `PLAN.md §6`: "Hostile defaults are the *product*. If they feel rude, it is because they are doing their job." The voice is working-developer, not consultant.

---

## How we speak

**When we say no:** Flat, active, complete. No hedging, no apology, no alternative suggested — unless the alternative is "use a different tool." From `PLAN.md §11`: "Flags that do not exist and will not be added: `--warn-only`, `--skip <check>`, `--allow-regressions`, `--soft-fail`, `--bypass`, `--force`. If a user wants any of these, they are looking for a different tool." From `PLAN.md §6`: "No `--warn-only` mode. Does not exist. Will not be added. A PR proposing it will be closed." Three sentences. Done.

**When we fail (violations):** Lead with the delta, name the culprit. From `audit.rs` violation format, pinned in test at line 1175: `"3 KiB over 100 KiB budget — images (82 KiB), css (13 KiB)"`. The pattern is always: amount over budget → em dash → the file causing it. No softening language. No "consider reducing." The violation is the instruction. On theater: call it what it is. From `audit.rs:519`: `"first <img> has loading=\"lazy\" — likely LCP gaming"`. We name the trick, not just the symptom.

**When we explain:** State the fact, then the mechanism, in one sentence. From `audit.rs:551`: `"missing <meta charset> or http-equiv Content-Type — forces encoding sniff"`. From `audit.rs:539`: `"3 <img> element(s) missing explicit width/height — layout shift (CLS)"`. The em dash separates what from why. Never longer than one sentence. Never narrative.

**When we succeed:** Quiet. From `report.rs:80`: `println!("{}  {}", "PASS".green().bold(), "all budgets met".dimmed())`. PASS is bold and green; the explanation is dimmed. No celebration — passing is the expected state, not a surprise.

**When we onboard a new user:** Same voice as the tool itself. From `README.md:4–8`: opens with the sundial metaphor, then immediately: "It does not produce a score out of 100. It produces pass or fail." From `PLAN.md`: "If that is not the tradeoff you want, use another tool." No welcome mat, no warmth, no feature list. The tool introduces itself by stating its position.

---

## The thing we'd never do

Bury the actionable signal under prose or leave the user hunting for which file caused the violation. Every violation leads with the number and names the culprit: `"33 KiB over — main.css (28 KiB)"`, `"1 over budget of 0 — serif-regular.woff2 (45 KiB)"`. The champion cycle 18 commit message names the principle directly: "budget owner — total bytes violation names nothing, category contributors would make it pastable." Pastable is the test: a violation must be copy-pasteable into a Slack message or PR comment and stand alone without the surrounding report.

---

## Signals to preserve

- **Lowercase imperative commit messages with a type prefix.** `feat: add SARIF 2.1.0 output format`, `test: pin case-insensitive content-type matching`, `docs: correct SARIF annotation placement`. No sentence case, no trailing period, no drama.

- **Violations lead with the delta, then the culprit, separated by an em dash.** `"N KiB over Y KiB budget — filename (size)"`. Never reversed. The champion and builder reading this brand file should treat violations that don't follow this pattern as off-brand — the user gets the number before they have to look up the budget.

- **Inline comments explain the adversarial case.** From `audit.rs:104–105`: `// inline styles / scripts roll into CSS/JS budgets so a site that inlines everything to "hide" bytes from external-resource checks still gets caught`. From `audit.rs:597–599`: `// the error case is "we undercount third parties on multi-suffix TLDs," which means we're generous to the site, not harsh`. Comments explain *why the rule exists and what it's catching*, not just what the code does. This extends to SARIF comments, budget file comments, and CONTRIBUTING.md.
