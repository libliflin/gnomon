# Goal — Cycle 8

## What

When the CSS bytes violation fires, include inline `<style>` bytes as a named contributor in the detail string. When the JS bytes violation fires, include inline `<script>` bytes as a named contributor. Today both violations can be entirely driven by inline bytes and say nothing about it.

**Expected output change:**
- CSS, inline-only (current): `1.66 MiB over`
- CSS, inline-only (fixed): `1.66 MiB over — (inline <style>) (1.67 MiB)`
- JS, mixed — external files dominate, unchanged: `1.86 MiB over — landingprod.js (112.99 KiB), iframeResizer.min.js (4.86 KiB)`
- JS, inline-only: `450 KiB over — (inline <script>) (451 KiB)`

**Where the data lives:** `analysis.inline_style_bytes` and `analysis.inline_script_bytes` are both in scope throughout `audit_url` in `src/audit.rs`. `css_resources` and `js_resources` are built during the fetch loop and sorted descending by size before the `bytes_check` calls (around lines 172–175).

**The fix:** Add a synthetic contributor entry to `css_resources` if `analysis.inline_style_bytes > 0` — `("(inline <style>)".to_string(), analysis.inline_style_bytes)`. Add a synthetic contributor entry to `js_resources` if `analysis.inline_script_bytes > 0` — `("(inline <script>)".to_string(), analysis.inline_script_bytes)`. The synthetic label is not a valid URL, so `url_filename` falls through to its `url.to_string()` fallback at line 440 and returns the label unchanged — no changes to `url_filename` needed.

Insert the synthetic entries before the sort (so they participate in top-2 ranking by size) or after (with a manual insertion at the correct position). Either is fine; leave the exact implementation to the builder.

No changes to `bytes_check` itself. No new types. No schema changes. Only the `css_resources` and `js_resources` construction sites in `audit_url` change.

Add tests pinning the new behavior:
1. CSS violation, inline-only: `css_resources` is empty, `inline_style_bytes` is large — contributor shows `(inline <style>) (X KiB)`.
2. JS violation, inline-only: same pattern for JS.
3. Mixed case: external files plus inline bytes — confirm inline appears in its correct sorted position among the top-2 (inline bytes may or may not be large enough to appear; test both sides of that threshold).

## Which Stakeholder

**The web performance engineer** (stakeholder 1). Last served in cycle 4 — three cycles ago. Most under-served stakeholder in the current rotation.

Step 4 of their journey: "Try to act on a violation — find the source of the bloat, understand the rule."

The CSS bytes violation is the highest-overage violation on most real-world sites. When it's driven by inline styles, the engineer sees "1.66 MiB over" and has no path to the cause without digging into raw JSON. Inline styles and external stylesheets require completely different fixes — one is a `<link rel="stylesheet">` to remove; the other is framework-injected CSS to rethink. Gnomon must name which situation they're in.

## Why Now

Three cycles have passed since the web performance engineer was served. Count violations all name their contributors (render_blocking cycle 4, third_party_domains and fonts cycle 6). But bytes violations have a structural gap: inline bytes are counted toward the total but anonymous in the contributor list.

The specific moment: `gnomon audit https://cnn.com`. CSS violation: `1.66 MiB over`. No contributors named. JSON output: `html_analysis.inline_style_bytes: 1755109`. Zero external CSS files. Every byte of the CSS budget violation is inline — gnomon knows it, the violation says nothing.

The fix is the same class as cycles 4 and 6: data already in scope, violation doesn't use it, one targeted change closes the gap. The label `"(inline <style>)"` passes through `url_filename` unchanged (fallback at line 440) — no helper changes needed.

**This is off-brand.** JS names its top contributors. Images names its top contributors. CSS can be silent about 1.67 MiB of inline styles. That asymmetry is the most visible inconsistency left in the bytes violation output.

## Lived-Experience Note

*I became the web performance engineer. Build clean. I ran `gnomon audit https://cnn.com`. Thirteen violations. The output was mostly good — forbidden list named ad networks, render_blocking named landingprod.js, third_party_domains named the domains. Then: `css: 1.66 MiB over`. No contributor named. The biggest CSS overage I've seen on any site, and gnomon told me nothing about where it comes from.*

*I ran `--format json` looking for more detail. `"detail": "1.66 MiB over"` — same as human output. I scanned the JSON manually. Found `inline_style_bytes: 1755109` in `html_analysis`. All of it inline. Not a single external CSS file in the resource list.*

*The momentum died there. I'd have to open Chrome DevTools, dig through the head, figure out which JavaScript framework is injecting 1.7 MB of CSS at runtime — all work gnomon could have told me to skip by just saying `(inline <style>)`.*

*The emotional signal for the web performance engineer is momentum: "I want to tell someone about this." I felt it building through the audit — thirteen violations, most named, most specific. Then the CSS violation killed it. Because "1.66 MiB over" with no attribution is not actionable. "(inline `<style>`) (1.67 MiB)" is.*
