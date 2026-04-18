# Goal — Cycle 6

## What

When the `third_party_domains` count violation fires, the detail string must name the offending domains. Today it says `"2 over"`. It should say `"2 over — google-analytics.com, doubleclick.net, googlesyndication.com"`.

When the `fonts` count violation fires, the detail string must name the top font files. Today it says `"1 over"`. It should say `"1 over — serif-regular.woff2 (45 KiB), sans-regular.woff2 (12 KiB)"`.

**third_party_domains**: The data is already in `totals.third_party_list` (computed above the violation-check block in `audit.rs`, sorted alphabetically). List all third-party domains — the count is small by design (budgets of 0–2), so listing them all is appropriate. Follow the render_blocking pattern from cycle 4: collect the names before the violation, fold them in with the em-dash separator.

**fonts**: The data is already in `font_resources` (computed and sorted descending by size). Apply the same top-2 contributor pattern that bytes violations already use: `url_filename(url) (size)`. The em-dash + comma-separated format is already established.

The `count_check` helper does not need a general contributor-hints parameter. Both are targeted overrides at their respective violation sites — the same approach used for render_blocking in cycle 4. Alternatively, replace the `count_check` call with an inline violation block for these two. Either is fine; leave the how to the builder.

No new types. No schema changes. The violation `kind`, `metric`, `budget`, `actual` fields stay the same. Only `detail` changes.

Also add tests pinning the new detail format for each violation type. Follow the pattern in `audit.rs:tests` (the `render_blocking_detail_*` tests).

## Which Stakeholder

**The team technical lead / budget owner** (stakeholder 3). Not served since cycle 3.

Step 4 of their journey: "Read a failing violation aloud." The budget owner's authority signal is "when gnomon says fail, I can defend it." For bytes violations and render_blocking, this holds — the violation names what pushed it over. For `third_party_domains` and `fonts`, it breaks: "2 over" does not carry enough for a standalone PR comment.

## Why Now

The render_blocking fix in cycle 4 established the pattern: count violations should name their contributors where the data is available. `third_party_domains` and `fonts` were left at the pre-cycle-4 level of specificity — the inconsistency is now the most off-brand thing in the violation output.

The specific moment that fails: the budget owner is in a code review, explaining a build failure. The violation says `third_party_domains: 2 over`. The reviewer asks "which ones?" The budget owner has to go back to the gnomon report. The violation didn't carry enough. That's the moment.

The data is already computed. `totals.third_party_list` and `font_resources` are both in scope at the violation-check sites. This is exactly the same "data is there, violation doesn't use it" gap that cycle 4 fixed for render_blocking.

## Lived-Experience Note

*I became the budget owner. I ran `gnomon audit` against a site with three third-party domains (google-analytics.com, doubleclick.net, googletagservices.com) against a mcmaster budget allowing 2.*

*Reading the violations: `css: 33 KiB over — main.css (28 KiB)` — I can defend that. `render_blocking: 1 over — landingprod.js` — I can defend that. Then: `third_party_domains: 1 over` — which domain? And then: `fonts: 1 over` — which fonts?*

*I tried to paste just the violation into Slack. "third_party_domains: 1 over." My teammate asked "which one?" I didn't have the answer in the message. I had to go back to the third-parties section of the report.*

*The worst moment: noticing that render_blocking names its offender, while third_party_domains and fonts don't. Cycle 4 fixed one. The other two were left behind. Gnomon names things — except in these two cases, it doesn't.*
