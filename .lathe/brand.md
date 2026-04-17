# Brand

**Identity.** Blunt, load-bearing authority that refuses to negotiate and defines itself by what it won't be. Gnomon speaks in declaratives: not a dashboard, not a scorer, no `--warn-only`, no yellow state, no easy mode (from `PLAN.md:9-10,56-59,622-623`; from `Cargo.toml` description: `"A CI gate, not a dashboard."`). The name is a signal: a gnomon reveals rather than evaluates — "ancient, exact, cheap, and correct for a thousand years without maintenance" (`PLAN.md:659`). It is not trying to be liked; it is trying to be right.

---

## How we speak

**When we say no.** Precise, unhedged, no apology. "`--warn-only` — Does not exist. Will not be added. A PR proposing it will be closed." (`PLAN.md:424–426`). "If a user wants any of these, they are looking for a different tool." (`PLAN.md:431`). No "we recommend," no "consider," no softer framing. The refusal is the feature.

**When we fail.** Name what fired, state the overage, and — when the cause is a recognized pattern — name the pattern. `"first <img> has loading=\"lazy\" — likely LCP gaming"` (`audit.rs:264`). `"14.7 KiB over"` (`audit.rs:178–183`). Errors prefix with `gnomon:` and end with the raw reason: `"gnomon: root fetch of {url} returned HTTP 404"` (`audit.rs:52–53`; `main.rs:23–24`). No stack trace in the lead. No apology before the fact.

**When we explain.** One clause, reason-first, next step when one exists. `"Google Tag Manager — opens the door to anything"`. `"polyfill.io — compromised in 2024"`. `"Intercom chat widget — use a mailto link"`. `"Google Fonts — self-host and subset your fonts"` (all from `forbidden.rs:9–18`). The explanation is in the violation label itself, not a follow-up paragraph.

**When we onboard a new user.** Two sentences, no preamble. `"Performance budget auditor. A CI gate, not a dashboard."` (`Cargo.toml` description; `cli.rs:11–12`). The `--help` long description repeats the pass/fail framing a second time and doesn't soften it. New users learn what gnomon is by reading what it isn't.

**When we celebrate.** Quiet. `"PASS  all budgets met"` — PASS in green and bold; the descriptor in dim (`report.rs:80`). The tool doesn't congratulate the site. It confirms the gate held. A clean build is the expected outcome, not an occasion.

---

## The thing we'd never do

We'd never introduce a severity gradient. There are violations and non-violations; there is no warning tier, no yellow state, no "technically over but marginal" (`PLAN.md:622–623`: "There are violations and non-violations. There is no yellow state."). Every violation type — `bytes`, `count`, `forbidden`, `theater` — exits 1. Softening one violation into a warning would unravel the only thing that makes the gate trustworthy: it never negotiates. The moment the tool can be talked down to a warning, operators stop acting on it.

---

## Signals to preserve

- **`gnomon: <subject> <reason>`** — errors always carry the tool name as prefix and the raw cause at the end. Never just a code, never just a message without context (`main.rs:15,23–24`).
- **`"<domain> — <one-clause reason>"`** — the forbidden list entry rhythm: domain first, em-dash, reason in one clause, next step when one exists (`forbidden.rs:9–23`). This pattern should hold as the list grows.
- **Pass is quiet, fail is specific** — `"all budgets met"` in dim vs. type + metric + overage per violation (`report.rs:80,88–104`). These two registers should never swap: the pass verdict earns its silence by having fail be precise.
