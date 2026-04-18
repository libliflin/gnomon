# Brand

## Identity

Precise, working-developer voice — leads with the specific thing, ends with the consequence or the next step, never hedges in between. Named after a part that *reveals* rather than scores, and that logic runs through every surface: the output tells you what the site is actually doing, not how it performs on a rubric (from `README.md` lines 3–8: "A gnomon is the blade on a sundial — the part that casts the shadow. It doesn't tell you the time; it reveals it" → "It does not produce a score out of 100. It produces pass or fail"). Confident about what it is; blunt about what it isn't.

---

## How we speak

**When we say no** (forbidden list, design limits): name the thing, then name the consequence or the alternative — nothing in between. `"Google Tag Manager — opens the door to anything"`, `"Intercom chat widget — use a mailto link"`, `"jQuery CDN — justify why in 2026"` (from `src/forbidden.rs` lines 9–32). We don't moralize; we state. When the refusal implies an alternative we name it. When the problem is historical fact we cite it: `"polyfill.io — compromised in 2024"`.

**When we fail** (error messages): tool name prefix, then what went wrong, then the offending value. `"gnomon: --url 'foo' is not a valid URL: ..."`, `"gnomon: root fetch of {url_str} returned HTTP {status}"` (from `src/audit.rs` lines 48, 54; `src/main.rs` lines 15, 24). Never apologetic. Surfaces the thing that failed so the user can act without re-running with verbose flags.

**When we detect a violation** (violation detail strings): observation — consequence, em-dash as the connector. Count or element first, never category first. `"first <img> has loading=\"lazy\" — lazy-loaded LCP candidate delays first render"`, `"N image(s) served as JPEG/PNG/GIF — serve AVIF or WebP to reduce transfer size"`, `"3 KiB over 100 KiB budget — images (82 KiB), css (13 KiB)"` (from `src/audit.rs` lines 520, 572, and the `bytes_check` function). The specific thing is always legible before the explanation arrives. No violation says "performance issue detected."

**When we explain** (README inline explanations, code comments): we give the reason, not just the rule. `"Running gnomon twice is the correct pattern when you want both annotations and enforcement"` (from `README.md` line 71). `"Naive eTLD+1 for third-party comparison. Falls short on .co.uk etc., but good enough for v0.0.2 — the error case is 'we undercount third parties on multi-suffix TLDs,' which means we're generous to the site, not harsh"` (from `src/audit.rs` lines 596–600). We name which direction our limitations lean so the reader can trust our judgment.

**When we pass** (verdict): flat, no exclamation. `"PASS  all budgets met"` (from `src/report.rs` line 80). The pass speaks for itself. We do not celebrate loudly; we confirm quietly. The work of celebration belongs to the engineer who fixed the thing.

**When we define ourselves**: we use the contrast. `"A CI gate, not a dashboard"` (from `Cargo.toml` description; `cli.rs` line 11). We repeat binary statements when they need to land: `"It does not produce a score. It produces pass or fail."` appears in the README, the CLI `about`, and the CLI `long_about` (from `README.md` lines 6–8, `cli.rs` lines 11–15). Repetition here is rhetorical, not redundant — the message is the thing we're correcting in how people think about performance tooling.

---

## The thing we'd never do

Produce a violation that buries the actionable detail under a category label. We never say "js performance issue" or "image format problem" when we can say `"first <img> has loading=\"lazy\" — lazy-loaded LCP candidate delays first render"`. The metric name (`lazy_lcp`, `img_format`, `viewport_meta`) is a machine key; the detail string is what the engineer reads. The detail string always leads with the specific element, the count, or the resource filename — never with the category (from `src/audit.rs` throughout `theater_violations` and `bytes_check`). If a violation detail string could belong to any site, it hasn't been written yet.

---

## Signals to preserve

**Em-dash as the observation–consequence connector in violation strings.** Not a colon, not a parenthetical, not a period and new sentence. The em-dash creates a single readable unit: what the site is doing, then what it costs. Breaking this rhythm weakens every violation string that follows it (from `src/audit.rs` lines 520, 529, 539, 550, 572, and all detail helper functions).

**Lowercase snake_case metric names.** `lazy_lcp`, `img_format`, `viewport_meta`, `charset_meta` — not "Lazy LCP Candidate" or "Missing Viewport Tag." The machine-facing names stay technical and flat; the human-facing detail strings carry the explanation. Never let marketing language into a metric name (from `src/audit.rs` throughout).

**Commit messages: `type: lowercase imperative phrase`, no period, no emoji.** `fix: remove hedge from lazy_lcp violation detail string`, `docs: explain why SARIF locations use URL not file path`, `feat: add SARIF 2.1.0 output format` (from git log). The commit that removes a hedge from a violation detail string is itself hedgeless. The discipline is consistent.

**The "not a X" definition pattern.** When introducing what gnomon is, we name what it's not before we explain what it is, and only when that contrast does real work. "A CI gate, not a dashboard" earns its existence because the alternative — a score out of 100 — is what the reader already expects and what we're replacing in their mental model (from `README.md` line 7, `Cargo.toml` description, `cli.rs` line 11).
