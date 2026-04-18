# Goal — Cycle 3

## What

The generated `gnomon.toml` (from `gnomon budget-init`) and the `gnomon presets` output should teach as they configure. Every byte value needs an inline comment showing the human-readable equivalent (e.g., `# 8 KiB`). Every zero-value field needs a comment explaining what "zero" enforces (e.g., `# 0 — no web fonts; system fonts only`). Every count field needs a comment explaining the unit (e.g., `# 0 — nothing may block first paint`).

The change lives entirely in `budget.rs` — specifically the `preset_toml()` function (around line 179) that generates the TOML string. The same string is used by both `budget-init` (written to disk) and `print_presets()` (printed to stdout), so fixing `preset_toml()` fixes both.

Add a header comment block. Add inline comments on every field line.

**Minimum comments:**

For the file header (generated `gnomon.toml`):
```
# gnomon.toml — performance budget (<preset> preset)
# All byte values are brotli-recompressed transfer sizes. CDN headers are not trusted.
# Edit these values to tighten or loosen your budget. Unknown keys will fail the audit.
```

For every `[bytes]` field:
- Non-zero: `# N KiB` (human-readable equivalent, e.g., `css = 14336  # 14 KiB`)
- Zero: `# 0 — <what this enforces>` (e.g., `js = 0  # 0 — zero JS; the insley standard`)

For every `[count]` field:
- Non-zero: a brief label (e.g., `requests = 6  # includes the HTML document itself`)
- Zero: `# 0 — <what this enforces>` (e.g., `render_blocking = 0  # 0 — nothing may block first paint`)

No schema change. No new logic. The numbers stay the same; the context arrives with them.

## Which Stakeholder

**The team technical lead / budget owner** (stakeholder 3). Step 2 of their journey is `gnomon budget-init --preset mcmaster`. The file lands in their repo, gets committed, and gets read by every engineer on the team. Step 3 is "try to explain it to a teammate." Today that explanation requires mental arithmetic (`10240` → "ten KiB"), domain knowledge (`render_blocking = 1` — is that a boolean flag or a count?), and a trip to PLAN.md.

## Why Now

The contributor has been served three cycles in a row. The budget owner's journey hasn't been touched.

The specific moment that failed: `render_blocking = 1`. Fifteen seconds of staring. In every other count context, "1" could be a boolean toggle (enabled/disabled) or a literal count (one resource is the limit). The field name alone doesn't answer it. This is the file that the team checks into source control as their performance contract — the ambiguity is in the source of truth.

The fix is structural: `preset_toml()` is a single function that generates all TOML output. Fix it once and every consumer of `budget-init` and `presets` gets the context. No new features, no schema changes.

**This is a brand problem, not just a UX problem.** Gnomon's violation messages are specific and actionable: "CSS 33 KiB over — main.css (28 KiB), vendor.css (5 KiB)." The config file says `css = 14336` with no context. The tool that names things precisely in its output should name things precisely in its input format. The inconsistency is off-brand.

## Lived-Experience Note

*I became the budget owner. `gnomon presets` (step 1): a wall of raw byte integers, no units, no rationale — `html = 8192`, `css = 14336`. I mentally computed: 8 KiB, 14 KiB. Fine. Then `gnomon budget-init --preset mcmaster` (step 2): file written. I opened the file. `render_blocking = 1` — I stared for fifteen seconds. Is `1` a boolean (detection is on) or a count (one resource is allowed)? The word "render_blocking" doesn't tell me. I would not be able to explain this line in a code review without documentation open. The worst moment was realizing this file lives in source control. Every engineer who opens the team's performance contract faces the same fifteen seconds of ambiguity. The violation messages are precise enough to paste into a PR comment and have them stand on their own. The config file requires external documentation to interpret. That gap is exactly what this goal closes.*
