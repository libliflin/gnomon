# Goal — Cycle 12

## What

When `analysis.img_missing_dimensions > 0`, fire a theater violation in `theater_violations`. Add a branch immediately after the existing `has_viewport_meta` check:

```rust
if analysis.img_missing_dimensions > 0 {
    vios.push(Violation {
        kind: ViolationKind::Theater,
        metric: "img_dimensions",
        budget: 0,
        actual: analysis.img_missing_dimensions as u64,
        detail: format!(
            "{} <img> element(s) missing explicit width/height — layout shift (CLS)",
            analysis.img_missing_dimensions
        ),
    });
}
```

Add tests for the new branch following the `theater_violations_*` pattern in `src/audit.rs`:

1. `theater_violations_img_missing_dimensions_fires` — `HtmlAnalysis { img_missing_dimensions: 3, has_viewport_meta: true, ..Default::default() }` → one Theater/img_dimensions violation, `actual == 3`, detail contains "layout shift".
2. `theater_violations_img_dimensions_zero_no_violation` — `HtmlAnalysis { img_missing_dimensions: 0, has_viewport_meta: true, ..Default::default() }` → no img_dimensions violation produced.
3. `theater_violations_img_dimensions_single_element` — `img_missing_dimensions: 1` → `actual == 1`, metric `"img_dimensions"`. Pins the singular-count case.

No new types. No schema changes. `img_missing_dimensions` is already a field on `HtmlAnalysis`, already populated by `analyze_html`, already serialized to JSON, already tested in `analyze.rs`. The only change is the branch in `theater_violations` and its tests.

## Which Stakeholder

**The web performance engineer** (stakeholder 1). Last served in cycle 8 — three cycles ago. Most under-served stakeholder in the current rotation.

Step 4 of their journey: "Try to act on a violation — find the source of the bloat, understand the rule."

Images without explicit width and height cause layout shift: the browser doesn't know the image's dimensions until it loads, so content below the image shifts as it arrives. This is CLS — a Core Web Vital that Google uses as a ranking signal. The web performance engineer who has been burned by Lighthouse scores that don't stop regressions cares specifically about CLS.

Gnomon detects this today. The detection runs, the count is computed, the data is in the JSON under `html_analysis.img_missing_dimensions`. But no violation fires. The web performance engineer auditing their site would never see this issue named.

## Why Now

Three cycles have passed since the web performance engineer was served (cycles 9–11 served CI integrator, budget owner, contributor).

The specific moment that failed: running `gnomon audit --format json` and finding `"img_missing_dimensions": 4` in `html_analysis`. Four images on the page without explicit dimensions. Zero violations for it. The field exists, the detection runs, the data is in the report — but gnomon says nothing.

This is the same class of gap that the recent cycle series has closed for other signals:
- Cycle 4: `render_blocking` had the data; violation detail didn't use it.
- Cycle 6: `third_party_domains` / `fonts` had the data; violation detail didn't use it.
- Cycle 8: inline CSS bytes had the data; violation detail didn't use it.
- Cycle 9: requests breakdown had the data; violation detail didn't use it.

This cycle's gap is starker: it is not a violation with a missing detail — it is a detected bad practice with *no violation at all*. The data goes all the way from `analyze_html` through the full pipeline into the JSON output and is never checked against anything.

**This is off-brand.** Gnomon's anti-theater category catches practices that pass Lighthouse but fail real users: lazy-loaded LCP candidates, missing viewport meta. A missing `width`/`height` on images directly causes layout shift — measurable, recordable, affecting Google ranking. Gnomon tracks it. Gnomon says nothing. That silence is off-brand for a tool that bills itself as "precision, certainty, and no apology."

The fix belongs in `theater_violations`. The function was extracted in cycle 11 for exactly this moment. The pattern is established. One branch, three tests.

## Lived-Experience Note

*I became the web performance engineer. I ran `gnomon audit https://my-site.com`. Violations fired — CSS named contributors, render_blocking named the file, forbidden named the ad network. All actionable. Momentum building.*

*I ran `--format json` to pipe the output into my dashboard tooling. I read through the JSON response. In `html_analysis` I found `"img_missing_dimensions": 4`. Four images without explicit dimensions. I know what that means — every page load, the browser doesn't know those images' sizes when it lays out the page, so content shifts as the images arrive. That's layout shift. That's CLS.*

*I looked at the `violations` array. Nothing for image dimensions. I looked at the human output. Silent. The field was there in the JSON. Gnomon had detected it. Nobody had said anything.*

*The worst moment: realizing gnomon had caught exactly the kind of thing I came here to catch — a real-world CLS cause — and then stayed silent about it. Not with a wrong answer. With silence where the answer should be.*

*The momentum signal — "I want to tell someone about this" — was building through the audit. The forbidden violation naming the ad network. The render_blocking violation naming the file. Then I found img_missing_dimensions in the JSON, checked violations, found nothing. Momentum stopped.*
