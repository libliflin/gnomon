# Brand

## Identity

Precise, clinical, unapologetic. Gnomon names what it sees — "LCP gaming," "theater," "33 KiB over" — and does not soften the language to protect the reader's feelings. It earns authority by never negotiating with bad defaults, and it applies that same standard to itself (from `PLAN.md §13`: "The tool audits its own build time. That is not a joke."). The project knows exactly what it is and is not, and the boundary is stated as a fact, not a policy.

Archetype: the gnomon itself. It doesn't tell you the time — it reveals it. The shadow is what was always there.

---

## How we speak

**When we say no.**
Categorical and final, not apologetic. "There is no `--warn-only` flag. Does not exist. Will not be added. A PR proposing it will be closed." (from `PLAN.md §6`). Not "we've decided not to support this at this time." The refusal is the position; no elaboration earns a softer landing.

The pattern extends to the non-goals list (`PLAN.md §16`): "Different problem, different tool." Two sentences. Done.

**When we fail.**
One line. The number first, then what pushed it over. `"33 KiB over — main.css (28 KiB), vendor.css (5 KiB)"` (from `src/audit.rs:203–225`). Not "you may be exceeding your CSS budget." The detail is actionable by design; you can paste it into a PR comment and it stands on its own.

Fetch and config errors prefix with the tool name and give the bare cause: `gnomon: {e:#}` (from `src/main.rs:24`). No apology. No suggestion to "try again later."

**When we name a bad practice.**
We call it what it is. The lazy-loaded LCP element violation reads: `"first <img> has loading=\"lazy\" — likely LCP gaming"` (from `src/audit.rs:301`). Not "potential anti-pattern." Not "may affect Lighthouse scores." The word is "gaming." The section in PLAN.md is titled "Anti-theater" — not "potential concerns" or "performance gotchas."

**When we onboard a new user.**
The `--help` about line: `"Performance budget auditor. A CI gate, not a dashboard."` (from `src/cli.rs:11`). One sentence. Binary distinction. Then immediately: exit codes, commands, what's working, what's coming. The README opens with the sundial metaphor and moves to `cargo install gnomon` within a page. No feature-list preamble.

**When we pass.**
`PASS  all budgets met` — green, one line, understated (from `src/report.rs:80`). The success case is quieter than the failure case. This is intentional: passing is the expected state; failure is the exception that needs attention.

**When we explain an internal decision.**
Even in source comments, the voice holds. From `src/audit.rs:369–370`: `"Naive eTLD+1 for third-party comparison. Falls short on .co.uk etc., but good enough for v0.0.2 — the error case is 'we undercount third parties on multi-suffix TLDs,' which means we're generous to the site, not harsh."` — acknowledges the flaw, names the failure mode, states which direction it errs. Transparency without hedging.

---

## The thing we'd never do

We'd never graduate a violation into a warning. "There are violations and non-violations. There is no yellow state." (from `PLAN.md §16`). The temptation is real — a yellow state is "user-friendly." Gnomon rejects it because a gate that flickers is not a gate. Any output format, any future feature, any addition to the CLI that introduces a third verdict state is off-brand. PASS and FAIL are the whole vocabulary.

---

## Signals to preserve

1. **`PASS` / `FAIL` in all-caps.** Every verdict is uppercase, final, unambiguous. Not "passed," not "success." The word is a binary state, not a description.

2. **The em-dash as separator in violation details.** `"33 KiB over — main.css (28 KiB), vendor.css (5 KiB)"`. The em-dash separates the verdict from the evidence. It's the breath between "what happened" and "who did it." Preserve this rhythm in any new violation type.

3. **Categorical negation in refusals.** "Does not exist. Will not be added." Not "is not currently supported." The present tense and the future tense both close the door. When gnomon refuses something, the refusal is a design decision, not a roadmap note.
