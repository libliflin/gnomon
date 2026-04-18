# Goal — Cycle 10

## What

When a CSS file appears as both `<link rel="preload" as="style" href="X.css">` and `<link rel="stylesheet" href="X.css">` in the same page, gnomon currently counts it as two render-blocking resources and names it twice in the violation detail. It should be counted and named once.

**The bug in the output:**
```
✗  count      render_blocking    5 over — main.css, main.css, vendor.css, vendor.css, reset.css, reset.css
```

**Expected output:**
```
✗  count      render_blocking    2 over — main.css, vendor.css, reset.css
```

(Budget 1, 3 unique render-blocking URLs → actual=3, over=2.)

**Root cause:** In `src/analyze.rs`, both the `rel="stylesheet"` branch (lines 99–110) and the `rel="preload" as="style"` branch (lines 132–142) independently add the same URL to `stylesheet_urls` with `render_blocking: true` and increment `render_blocking_in_head`. When a modern site (Next.js, Nuxt, etc.) emits both tags for the same stylesheet, gnomon double-counts.

**The fix:** Deduplicate render-blocking resources so each URL is counted and named once. Where to apply the deduplication (in `analyze.rs` at parse time, or in `audit.rs` when building `totals.render_blocking` and `blocking_names`) is left to the builder. The requirement is:

- `render_blocking_in_head` (or the value used in the violation's `actual` field) reflects the number of **unique** render-blocking URLs, not the number of `<link>` tags.
- The detail string lists each unique filename once.

**Add a test** in `analyze.rs` (or `audit.rs` as appropriate) that pins this case: HTML with both `<link rel="preload" as="style" href="X.css">` and `<link rel="stylesheet" href="X.css">` must yield `render_blocking_in_head == 1`, not 2. This is the missing test that would have caught the regression.

No new types. No schema changes. No change to the violation `kind`, `metric`, `budget` fields. Only the count and the `detail` string change for pages that emit both preload and stylesheet links for the same URL.

## Which Stakeholder

**The team technical lead / budget owner** (stakeholder 3). Last served in cycle 6 — the longest wait in the current rotation (4 cycles).

Step 4 of their journey: "Read a failing violation aloud." The budget owner's authority signal is "when gnomon says fail, I can defend it." Duplicate filenames in the violation detail break that signal completely.

The budget owner pastes `render_blocking: 5 over — 8d8893031fe72ff0.css, 8d8893031fe72ff0.css, 8a89dc115690f8c4.css, 8a89dc115690f8c4.css, eb4400433120c47a.css, eb4400433120c47a.css` into a code review comment. Their teammate asks: "Why is the same file listed twice?" The budget owner has no answer. They don't know if this is a bug or a gnomon quirk. They open PLAN.md looking for an explanation. There is none. Confidence collapses.

## Why Now

The budget owner is the most under-served stakeholder (cycle 6 was last, 4 cycles ago).

The specific moment: running `gnomon audit https://www.theverge.com` with a mcmaster preset. The render_blocking violation lists 6 filenames — 3 unique names, each appearing twice. The count says `5 over` (budget 1, actual 6), but there are only 3 unique render-blocking resources. Both the count and the name list are wrong.

This is a correctness bug, not a polish gap. The `render_blocking_in_head` counter is incremented twice for each CSS file that appears as both a preload and a stylesheet. The comment in the code acknowledges "We flag both forms" — but both forms of the same URL is still one resource. The browser fetches it once. Gnomon should count it once.

**This is off-brand in the worst way.** Gnomon's brand is precision and certainty. "33 KiB over — main.css (28 KiB)" names one file once. "5 over — main.css, main.css" names the same file twice. The tool that bills itself as a CI gate should not emit a violation list that looks like a deduplication bug. The budget owner cannot defend a gate whose own output they can't explain.

A test pinning this case would have caught it — which makes it also a contributor story: the next contributor to touch `analyze.rs` has no test that would detect a regression here.

## Lived-Experience Note

*I became the budget owner. I ran `gnomon budget-init --preset mcmaster` — the file was well-commented, I could explain every field. I ran `gnomon audit https://www.theverge.com`. Sixteen violations.*

*Most I could defend in a code review immediately. `css: 19.76 KiB over — 8d8893031fe72ff0.css (32.05 KiB)` — clear. `third_party_domains: 7 over — amazon-adsystem.com, bullwhip.cloud...` — clear. `render_blocking: 5 over — 8d8893031fe72ff0.css, 8d8893031fe72ff0.css, 8a89dc115690f8c4.css, 8a89dc115690f8c4.css, eb4400433120c47a.css, eb4400433120c47a.css` — I stopped.*

*The same CSS filenames appear twice. The count says "5 over" — budget 1, actual 6 — but there are only 3 unique CSS files in that list. I stared at this for thirty seconds. Was this a gnomon bug? A real thing? I opened `analyze.rs` and traced through: `<link rel="preload" as="style">` adds the URL to `stylesheet_urls` and increments the count. Then `<link rel="stylesheet">` for the same URL also adds it and increments again. The Verge's Next.js build emits both tags for every stylesheet. Six entries, three files, each counted twice.*

*The worst moment: trying to formulate a code review comment. I couldn't paste the violation as-is — it would raise more questions than it answered. I had to manually deduplicate in my head, recount, and rewrite it: "3 render-blocking CSS files (budget: 1)." Gnomon made me do the work that gnomon should have done.*

*The authority signal died when I couldn't paste the violation unchanged. "When this says fail, I can defend it" — I couldn't. Not because the failure was wrong, but because gnomon said the same thing twice.*
