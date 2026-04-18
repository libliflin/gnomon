# Goal — Cycle 19

## What

`preload_hint_count` in `HtmlAnalysis` is detected (line 118 of `src/analyze.rs`), serialized to JSON, and has zero tests. No test verifies that `<link rel="preload">` tags increment the count, that the count stays at zero for non-preload links, or that different `as=` variants all trigger it. A contributor who modifies preload detection logic has no test that would catch a regression.

Add tests for `preload_hint_count` in `src/analyze.rs`:

1. `preload_hint_count_increments_for_each_preload_variant` — HTML with three preload tags (`as="style"`, `as="font"`, `as="image"`) must produce `preload_hint_count == 3`. Pins that all `<link rel="preload">` variants count regardless of `as=` type.
2. `non_preload_link_does_not_increment_preload_hint_count` — HTML with a plain `<link rel="stylesheet">` and a `<link rel="icon">` must produce `preload_hint_count == 0`. Pins the boundary: only `rel="preload"` triggers the count.
3. `single_preload_hint_is_counted` — HTML with exactly one `<link rel="preload" as="script">` must produce `preload_hint_count == 1`. Minimal positive case.

Additionally, update `CONTRIBUTING.md`'s good-first-issues section. The current guidance says: "scan `HtmlAnalysis` fields in `src/analyze.rs` for fields that are detected and serialized to JSON but have no corresponding branch in `theater_violations`. Each such field is a candidate anti-theater rule waiting to be wired."

This algorithm now produces a false positive: `preload_hint_count` and `preconnect_targets` match the pattern but are intentionally informational — preload hints are a positive performance optimization, not an anti-pattern, and preconnect targets are already policed by the forbidden list. A contributor following the guide finds these fields and spends time deciding whether to add a theater violation for them. The guide said "each such field is a candidate" — these aren't.

Update the good-first-issues section to:
- State there are currently no known gaps (all theater candidates are wired)
- Name `preload_hint_count` and `preconnect_targets` explicitly as intentionally informational fields (not gaps) so the discovery algorithm's false positives are named, not left as archaeology

No functional changes. No new violation types. No schema changes. Only new tests in `analyze.rs` and a documentation update in `CONTRIBUTING.md`.

## Which Stakeholder

**The contributor** (stakeholder 4). Last served cycle 15 — four cycles ago. Most under-served stakeholder in the current rotation.

Step 7 of their journey: "Look for a test to copy as a starting point." And the CONTRIBUTING.md good-first-issue path.

The contributor's clarity signal is "I know exactly where this goes and how to test it." CONTRIBUTING.md was written in cycle 15 to deliver that clarity. It did — for `has_charset_meta`. After cycle 16 wired `has_charset_meta`, the general discovery algorithm still works but now points to `preload_hint_count`. Which is not a theater candidate. The guide said "candidate." The contributor finds it and immediately realizes: a violation saying "you have preload hints" would fail any page doing correct performance optimization. That's wrong. They stop and wonder if they're misunderstanding the guide or if this is genuinely a gap.

## Why Now

Contributor is the most under-served stakeholder (4 cycles). The two specific gaps compound each other:

**Gap 1: zero test coverage for `preload_hint_count`.**
The field is counted, serialized, and tracked across audit runs. A contributor who modifies the preload detection logic in `analyze_html` — say, to change how `rel="modulepreload"` is handled, or to add a new `as=` variant — has no test that catches a regression in the count. Contrast with every other `HtmlAnalysis` field: `render_blocking_in_head` has 9 direct tests in `analyze.rs`, `img_missing_dimensions` has 3, `lazy_lcp_candidate` has 2. `preload_hint_count` has 0. The existing preload tests (`preload_as_style_is_render_blocking`, `preload_as_font_is_not_render_blocking`, etc.) all assert on `render_blocking_in_head` — none assert that `preload_hint_count` was incremented.

**Gap 2: CONTRIBUTING.md discovery algorithm has a known false positive.**
The guide says "each such field is a candidate anti-theater rule." This was true when written — `has_charset_meta` was the only field matching the pattern. After cycle 16, the pattern finds `preload_hint_count` and `preconnect_targets`, neither of which should be a theater violation. The clarity signal dies at the same point as the `has_charset_meta` ambiguity in cycle 15 — "is this a known gap or intentional?" — except this time the guide's answer ("it's a candidate") is wrong.

The fix is structural in the CONTRIBUTING.md sense: name the exception once, prevent 20 minutes of archaeology for every future contributor who follows the path. The test fix is structural in the analyze.rs sense: every detected-and-tracked field should have at least one direct assertion on its counting logic.

## Lived-Experience Note

*I became the contributor. Build clean. Clippy clean. 119 tests passing — high confidence.*

*Opened `analyze.rs`. 37 tests. Every detection logic pinned: `render_blocking_in_head` has 9 tests, `img_missing_dimensions` has 3, `has_charset_meta` has 2. I skimmed the test names and felt clear about every field.*

*Opened `audit.rs`. `theater_violations()` comment: "New theater checks belong here." Clear. Opened CONTRIBUTING.md. Three paths, named tests to copy. Good first issues: "scan HtmlAnalysis fields with no theater_violations branch — each such field is a candidate."*

*I scanned `HtmlAnalysis`. Four theater violations wired: `lazy_lcp_candidate`, `has_viewport_meta`, `img_missing_dimensions`, `has_charset_meta`. Two fields without theater branches: `preload_hint_count` and `preconnect_targets`.*

*I went to `theater_violations` to add the branch for `preload_hint_count`. I wrote:*

```rust
if analysis.preload_hint_count > 0 {
    // Wait.
}
```

*Preloads are a performance optimization. `<link rel="preload" as="font">` tells the browser to prefetch a font early. That's a GOOD signal. A violation saying "you have preload hints" would fail any page doing the right thing. The guide said "each such field is a candidate anti-theater rule" — this one isn't.*

*I deleted the branch. Then I went looking for a test to prove the field even works correctly. There are zero assertions on `preload_hint_count` in all of `analyze.rs`. The tests for preload links (`preload_as_style_is_render_blocking`, `preload_as_font_is_not_render_blocking`) all check `render_blocking_in_head` — none check `preload_hint_count`. The field is counted and serialized to JSON, but I have no way to verify the count is right without adding a test myself.*

*Two failures: a guide that sent me to the wrong place, and a field with no test coverage. Either one would have been fixable in 5 minutes with the right signal. Without the signal, I spent 20 minutes deciding whether the guide was wrong or I was misunderstanding the design. The clarity signal died at "is this a gap or intentional?" — the exact question CONTRIBUTING.md was written to answer, but now it points me in the wrong direction.*
