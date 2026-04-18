# Goal — Cycle 4

## What

When the `render_blocking` count violation fires, the detail string must name the render-blocking resource(s). Today it says `"1 over"`. It should say `"1 over — landingprod.js"` (or whichever URL is blocking render).

The data is already in scope at the violation-check site in `src/audit.rs`. After the fetch loop, `analysis.stylesheet_urls` and `analysis.script_urls` both carry a `render_blocking: bool` field on each entry. Filter those to `render_blocking == true`, extract the filename via `url_filename()` (already defined in `audit.rs`), and include them in the violation detail — following the same em-dash + comma-separated pattern that JS and image byte violations already use:

```
1 over — landingprod.js
```

or for multiple:

```
2 over — main.css, landingprod.js
```

The `count_check` helper does not need a general contributor-hints parameter. This is a targeted override specific to render-blocking: collect the blocking URL names before the `count_check` call and fold them into the detail after. Alternatively, inline the render-blocking violation directly instead of using `count_check` — either is fine; leave the how to the builder.

No new types. No schema changes. The violation `kind`, `metric`, `budget`, `actual` fields stay the same. Only `detail` changes.

## Which Stakeholder

**The web performance engineer** (stakeholder 1). First cycle serving them — they've been unserved for three cycles.

Step 4 of their journey: "Try to act on a violation — find the source of the bloat, understand the rule." When `render_blocking` fires, the current output tells them the count but not the cause. They have to open Chrome DevTools — the exact tool they came to gnomon to replace — to find which resource is blocking the render path.

## Why Now

Cycles 1 and 2 served the contributor. Cycle 3 served the budget owner. The web performance engineer hasn't been touched.

The render-blocking violation is the most actionable violation gnomon can fire — blocking render is a direct causal path to slow LCP — and the least complete. Every other violation type either names itself fully (forbidden: `googlesyndication.com — Google ad syndication`) or shows top contributors (js: `landingprod.js (112.99 KiB), iframeResizer.min.js (4.86 KiB)`). The render-blocking violation says `1 over`. That's it.

The data is there. `analysis.script_urls` already has `render_blocking: true` on the offending entry. The violation is missing one line that collects it.

**The specific moment:** Running `gnomon audit https://cnn.com` produced `render_blocking: 1 over` with no context. `--format json` showed `"detail": "1 over"` — same as the human output. But `html_analysis.script_urls[0]` in that same JSON response had `render_blocking: true` and `url: "https://cdn.optimizely.com/public/125375509/s/landingprod.js"`. Gnomon knew. The violation didn't say.

**This is off-brand.** Gnomon names things. `"render_blocking: 1 over"` sounds like a dashboard metric. `"render_blocking: 1 over — landingprod.js"` sounds like gnomon.

## Lived-Experience Note

*I became the web performance engineer. I ran `gnomon audit https://cnn.com`. The violations fired — 13 of them. Most were actionable: the forbidden list named specific domains, JS named the top two contributors, images named the two largest files. Then I hit `render_blocking: 1 over`. Momentum stopped. Which resource? I ran `--format json` looking for more detail. The JSON `violations` array had `"detail": "1 over"` — no more information than the human output. But right above it, in `html_analysis.script_urls`, was the answer: Optimizely's `landingprod.js`, `render_blocking: true`. Gnomon knew. The violation didn't say. That's the moment.*

*The emotional signal for the web performance engineer is momentum: "I want to tell someone about this." I felt it building through the audit — the forbidden list naming ad networks, the byte totals naming the biggest offenders. Then the render-blocking violation killed it. Because without knowing which file is blocking render, I can't act. I'd have to open DevTools. And if I'm opening DevTools, I don't need gnomon.*
