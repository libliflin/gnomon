# Goal — Cycle 14

## What

Include the budget value in every bytes and count violation detail string so the violation is self-contained when pasted into a PR comment, Slack message, or code review note.

**Current output (violation list only, after the bytes/counts tables):**
```
✗  bytes      html                      489.47 KiB over
✗  bytes      css                       1.72 MiB over — (inline <style>) (1.74 MiB)
✗  bytes      total                     6.62 MiB over
✗  count      requests                  52 over — 68 img, 2 js (72 total)
✗  count      render_blocking           2 over — main.css, vendor.css
```

**Expected output:**
```
✗  bytes      html                      489.47 KiB over 10 KiB budget
✗  bytes      css                       1.72 MiB over 20 KiB budget — (inline <style>) (1.74 MiB)
✗  bytes      total                     6.62 MiB over 300 KiB budget
✗  count      requests                  52 over budget of 20 — 68 img, 2 js (72 total)
✗  count      render_blocking           2 over budget of 1 — main.css, vendor.css
```

**Where the change lives:** All five detail helpers in `src/audit.rs`, plus their call sites:

1. **`bytes_check`** (line ~349): `budget` is already a parameter. Append `" {budget_str} budget"` after the overage amount before any em-dash contributors. `humansize::format_size(budget, humansize::BINARY)` produces the formatted size.

2. **`render_blocking_detail`** (line ~388): add `budget: u32` parameter. Append `" budget of {budget}"` after the overage, before the em-dash. Call site: line ~259, `bgt` is in scope.

3. **`third_party_domains_detail`** (line ~398): add `budget: u32` parameter. Same pattern. Call site: line ~239, `bgt` is in scope.

4. **`fonts_count_detail`** (line ~409): add `budget: u32` parameter. Same pattern. Call site: line ~272, `bgt` is in scope.

5. **`requests_count_detail`** (line ~433): add `budget: u32` parameter. Same pattern. Call site: line ~219, `bgt` is in scope.

No changes outside `src/audit.rs`. The `Violation` struct, `budget.rs`, `report.rs`, and `violation.rs` are unchanged — the `detail` string already carries everything that appears in both human and JSON output.

Update all existing tests that pin the current `"N over — ..."` or `"N over"` format strings (approximately 15 tests across `render_blocking_detail_*`, `third_party_domains_detail_*`, `fonts_count_detail_*`, and `requests_count_detail_*` test groups). The new format must include the budget value.

The exact phrasing (e.g., `"over 14 KiB budget"` vs `"/ 14 KiB (over)"`) is left to the builder. The requirement: given only the violation line, a reader knows both the overage AND the budget without needing the bytes/counts table above it.

## Which Stakeholder

**The team technical lead / budget owner** (stakeholder 3). Last served in cycle 10 — four cycles ago, the longest wait.

Step 4 of their journey: "Run `gnomon audit` with the config and read a failing violation aloud." The authority signal is: "When gnomon says fail, I can defend it."

Defending a violation means being able to paste it into a code review comment and have it stand without additional explanation. "CSS is 1.72 MiB over budget" requires context. "CSS is 1.72 MiB over the 20 KiB budget — mostly inline styles" doesn't.

## Why Now

Four cycles have passed since the budget owner was served. The violation output has improved substantially: every violation names its contributors (bytes contributors from cycle 8, render_blocking names its files from cycle 4, third_party_domains from cycle 6, fonts from cycle 6, requests breakdown from cycle 9). But none of the bytes or count violations include the budget value. The budget number is only visible in the bytes/counts tables printed above — it disappears when a violation is pasted without that context.

The forbidden and theater violations are fully self-contained today:
- `googlesyndication.com — Google ad syndication` — names the pattern AND the reason
- `2 <img> element(s) missing explicit width/height — layout shift (CLS)` — names the issue AND the consequence

The bytes and count violations state magnitude but not the standard being enforced:
- `489.47 KiB over` — over what? (budget: 10 KiB)
- `52 over — 68 img, 2 js (72 total)` — over what? (budget: 20)
- `6.62 MiB over` — over what? (budget: 300 KiB)

This asymmetry means the budget owner can paste a forbidden violation and have it stand; they cannot paste a bytes violation and have it stand. The budget is the entire reason for the violation's existence. Omitting it from the detail string is the single most off-brand thing in the current output.

The specific moment: `gnomon audit https://cnn.com` with a mcmaster budget. Looking at `html: 489.47 KiB over` — I would write this into a Slack message but I'd have to manually add "(budget: 10 KiB)" so my teammate could understand the severity. Every other violation I'm explaining is already doing that work. This one isn't.

The change is localized to one file (`src/audit.rs`), touches only detail string formatting, and updates existing tests. No new features, no new types, no schema changes.

## Lived-Experience Note

*I became the budget owner. Build clean, 104 tests, clippy clean. I ran `gnomon budget-init --preset mcmaster` — the file was well-commented, I could explain every field (cycle 3 fixed this). I ran `gnomon audit https://cnn.com` with the config in the working directory — auto-discovery picked it up (cycle 5 fixed this). Twelve violations.*

*I opened Slack to post the violations. The forbidden list I copied verbatim — self-explanatory. The theater violation: `2 <img> element(s) missing explicit width/height — layout shift (CLS)` — self-explanatory. The render_blocking violation: `2 over — main.css, vendor.css` — I paused. Over what? Budget of 1. I added that manually before sending.*

*Then I hit the bytes violations. `489.47 KiB over`. The HTML is that far over budget. My teammate will ask: "over what budget?" I have to say "the HTML budget is 10 KiB." That's context I pulled from the gnomon.toml, not from the violation. `1.72 MiB over 20 KiB budget` would have carried it.*

*The worst moment: `6.62 MiB over`. The total budget violation. No contributors, no budget number. Six and a half megabytes over — what exactly? A budget of 300 KiB. Gnomon knows this. The violation says nothing. For a tool that insists on precision, printing the magnitude of a failure without printing the standard being violated is the single biggest gap left in the output.*

*The authority signal — "when gnomon says fail, I can defend it" — holds for the forbidden and theater violations. It breaks for every bytes and count violation the moment they're pasted without the surrounding table.*
