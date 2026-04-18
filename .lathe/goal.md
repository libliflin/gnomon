# Goal — Cycle 20

## What

When gnomon fetches image resources, it already has each resource's `content_type`. It already fires a bytes violation naming the two largest images by size. But it says nothing about whether those images are served in a legacy format. On a real-world site (The Verge: 62 images, 51 JPEG + 11 PNG), gnomon knows every image's content type and says nothing. PLAN.md §7.1 explicitly names this check: "reject JPEG/PNG where AVIF/WebP exists."

Add a Theater violation that fires when image resources are served as JPEG, PNG, or GIF — legacy formats that AVIF or WebP would replace with better compression at equivalent visual quality.

**Expected violation:**

```
✗  theater   img_format   62 image(s) served as JPEG/PNG/GIF — serve AVIF or WebP to reduce transfer size
```

**Rules:**
- Count fetched resources classified as images (by content-type or URL classification) with `content_type` matching `image/jpeg`, `image/png`, or `image/gif`
- Only count resources that were fetched without error (exclude resources with a non-null `error` field)
- Exclude SVG (`image/svg+xml`), WebP (`image/webp`), and AVIF (`image/avif`) — these are already appropriate formats
- If the count is > 0, fire `ViolationKind::Theater` with metric `"img_format"`, `budget: 0`, `actual: count`, detail as shown above

**Detail string:**
```
"{count} image(s) served as JPEG/PNG/GIF — serve AVIF or WebP to reduce transfer size"
```

The violation lives in `audit_url` in `src/audit.rs`, assembled from the fetched resources — not in `theater_violations` (which takes only `&HtmlAnalysis`). Place it after the existing `theater_violations` call in the violations assembly block.

Add tests pinning the new check:

1. `img_format_violation_fires_for_jpeg_and_png` — fetched resources include two JPEG images and one PNG; violation fires with `actual == 3`, metric `"img_format"`, detail contains "3 image(s)".
2. `img_format_violation_does_not_fire_for_webp_avif` — fetched resources include one WebP and one AVIF image; no `img_format` violation.
3. `img_format_violation_excludes_fetch_errors` — fetched resources include one JPEG image with a non-null `error` field and one JPEG without error; violation fires with `actual == 1` (only the successfully fetched JPEG counted).
4. `img_format_violation_does_not_fire_when_no_images` — fetched resources include only JS and CSS (no images); no `img_format` violation.

No new types. No schema changes. No changes to `theater_violations`. Only the violation assembly in `audit_url` and its tests.

## Which Stakeholder

**The web performance engineer** (stakeholder 1). Last served cycle 16 — four cycles ago. Most under-served stakeholder in the current rotation.

Step 4 of their journey: "Try to act on a violation — find the source of the bloat, understand the rule."

The images bytes violation is the highest-overage violation on media-heavy sites. When it fires, gnomon names the two largest files by size. But the web performance engineer's highest-ROI action is often FORMAT conversion — switching JPEG/PNG images to AVIF/WebP can reduce image weight by 30–50% without changing which images appear on the page. Gnomon currently detects image bytes, but not image format. The engineer auditing their site has to inspect the JSON manually to understand whether format conversion is an opportunity. It's not surfaced as a violation.

## Why Now

Four cycles have passed since the web performance engineer was served (cycles 17–19 served CI integrator, budget owner, contributor).

The specific moment that failed: I ran `gnomon audit https://www.theverge.com`. The images violation fired: `18.40 MiB over — 258176_Did_Neuralink_make_the_wrong_bet__CVirginia.jpg (3.86 MiB), STKS517_AGE_VERIFICATION_B.jpg (1.92 MiB)`. That told me WHICH images were largest. But I needed to know: should I compress them harder, remove some, or convert formats? Gnomon knew the answer — it fetched 62 JPEG/PNG images. `content_type: "image/jpeg"` is in the JSON for every one. The violation said nothing about format.

I ran `--format json` and counted manually: 51 `image/jpeg`, 11 `image/png`, 0 AVIF, 0 WebP. Sixty-two images in legacy formats. That's the biggest quick-win for the site's image weight — not "which is largest" but "they're all JPEG/PNG." Gnomon had the data. The violation didn't use it.

This is the same class of gap as cycles 4, 6, 8: data in scope, violation doesn't use it. But unlike those, this is a data point gnomon hasn't surfaced in ANY output context — not in violation detail, not in the theater section, not in a count. The format information is locked in the JSON resource list and never converted to guidance.

**This is on-brand.** PLAN.md §7.1 names it: "reject JPEG/PNG where AVIF/WebP exists." Gnomon audited 62 images. Gnomon said nothing about their format. That silence is exactly what this check closes.

## Lived-Experience Note

*I became the web performance engineer. Build clean. I ran `gnomon audit https://www.theverge.com`. Sixteen violations — bytes, counts, theater, forbidden. Impressive. Most named, most actionable. Momentum building.*

*The images violation: `18.40 MiB over — 258176_Did_Neuralink_make_the_wrong_bet__CVirginia.jpg (3.86 MiB), STKS517_AGE_VERIFICATION_B.jpg (1.92 MiB)`. Named top 2 files. I can act on that — those are the images to optimize first.*

*But optimize HOW? Compress harder? Remove some? Switch formats? The violation told me nothing about whether I was already using modern formats. The images budget violation just says "you have too many image bytes."*

*I ran `--format json`. I looked through the `resources` array. Every image: `"content_type": "image/jpeg"`. All 51 of them. Plus 11 PNGs. Zero WebP. Zero AVIF. That's the actual finding — 62 images in legacy formats, none converted. That's not a "largest file" problem, that's a "missing format conversion pipeline" problem. The fix is different: not "delete images" but "run all images through a WebP/AVIF converter."*

*The momentum died at the format check. Gnomon named the bytes, named the biggest files — but didn't tell me the FORMAT was wrong across the board. I had to grep through the JSON myself. Format conversion is one of the top three image optimization actions (alongside compression and lazy loading), and gnomon has all the data to surface it. It just doesn't.*
