# Goal — Cycle 11

## What

Extract the anti-theater violation assembly from `audit_url` in `src/audit.rs` into a standalone pure function:

```rust
fn theater_violations(analysis: &HtmlAnalysis) -> Vec<Violation>
```

This function takes only `&HtmlAnalysis` and returns the violations produced by the two current anti-theater checks: `lazy_lcp_candidate` and `has_viewport_meta`. The `audit_url` body replaces the current inline block with a call to `theater_violations` and extends `violations` with the result.

Add tests for the extracted function pinning the four states a contributor needs to understand:

1. `theater_violations_lazy_lcp_fires` — `HtmlAnalysis { lazy_lcp_candidate: true, ..Default::default() }` → one `Theater/lazy_lcp` violation with the correct detail string.
2. `theater_violations_missing_viewport_fires` — `HtmlAnalysis { has_viewport_meta: false, ..Default::default() }` → one `Theater/viewport_meta` violation with the correct detail string.
3. `theater_violations_clean_page_returns_empty` — `lazy_lcp_candidate: false, has_viewport_meta: true` → empty vec.
4. `theater_violations_both_flags_emit_two_violations` — both flags set → two violations, one per check.

No functional changes to `audit_url`'s behavior. No schema changes. No new violation types. The output is identical before and after — only the internal structure changes.

The function name `theater_violations` matches the vocabulary already in use: `ViolationKind::Theater`, `PLAN.md §4` anti-theater, the "Anti-theater detection" line in the README's feature list. It names the seam correctly.

## Which Stakeholder

**The contributor** (stakeholder 4). Last served cycle 7 — four cycles unserved, the longest wait.

Step 8 of their journey: "Try to add a trivial check and verify it fires."

The contributor wants to add a new anti-theater rule — say, detecting a `<html>` element without a `lang` attribute, or a missing `<meta name="description">`. They can:

1. Add a field to `HtmlAnalysis` in `analyze.rs` → test it there (pattern exists, 30+ tests).
2. Wire detection in `analyze_html` → tested by step 1.
3. Add a branch to the theater block in `audit_url` → **no test pattern for this layer.**

Step 3 is the wall. Every test in `audit.rs` is for a helper function (`bytes_check`, `render_blocking_detail`, `url_filename`, etc.). None test the violation assembly itself. To verify a new theater violation fires, the contributor would have to call `audit_url` against a real URL — which is not testing, it's manual verification against the internet.

After this change: step 3 becomes "add a branch to `theater_violations`, write a test" — the same pattern as steps 1 and 2. The full loop is testable end-to-end with no network calls.

## Why Now

The contributor is the most under-served stakeholder (4 cycles). The test suite has grown from 1 stub to 97 tests across three files. But the violation assembly layer has zero tests. This gap is most visible for the anti-theater checks because they are:

- The simplest violation type to add (one field, one branch).
- The natural next contribution: the current checks (`lazy_lcp`, `viewport_meta`) are the start of a category that could grow (missing `lang`, missing `description`, analytics pixel in head, etc.).
- The ONLY part of `audit_url` that depends solely on `HtmlAnalysis` — extracting them requires no mocking, no fake fetched-resources, no network stub.

The specific moment: I added `has_description_meta: bool` to `HtmlAnalysis`. Wired detection in `analyze_html`. Wrote a test — green. Added the `if !analysis.has_description_meta { violations.push(...) }` branch to the theater block in `audit_url`. Then I looked for a test to copy. Every test in `audit.rs` calls a helper directly — `bytes_check`, `render_blocking_detail`. None test the violation assembly path. I could not write "given `analysis.has_description_meta == false`, the violation list must contain Theater/missing_description" without a real network call.

That's the moment clarity died. The field is detected. The violation is wired. And there's no test. Not because the project doesn't value tests — 97 of them exist — but because the architectural seam was never drawn at the right place.

**This is a structural gap, not a polish gap.** Every future contributor adding a theater check hits this same wall. Extracting `theater_violations` draws the seam once and eliminates the wall for all future contributions of that type.

## Lived-Experience Note

*I became the contributor. Cloned. Built clean. Clippy clean. `cargo test` — 97 passing. Confidence high — more tests than any comparable project at this stage.*

*Opened `analyze.rs`. Thirty tests. Every rule pinned. The pattern for adding a detection is obvious: add a field, add detection logic, add a test. I did it for a hypothetical `has_description_meta` check in under 5 minutes.*

*Then I went to `audit.rs` to wire the violation. Found the theater block — two `if` statements, clear, obvious. Added mine. Then I looked for a test.*

*Every test in `audit.rs` calls a helper. `url_filename`. `registrable_domain`. `render_blocking_detail`. `bytes_check`. `requests_count_detail`. All helpers, all pure functions, all individually tested. But the body of `audit_url` — the if-statements that check budgets and produce violations — zero tests.*

*The test I wanted to write: "given an `HtmlAnalysis` with `has_description_meta: false`, `theater_violations` must include a `Theater/missing_description` violation." I could not write that test. The logic lived inside an async function that fetches URLs.*

*The worst moment: realizing I couldn't tell if my violation was wired correctly without running `gnomon audit https://a-site-with-no-meta-description.com`. That's not testing. That's the internet as a test harness.*

*The clarity signal — "I know exactly where this goes and how to test it" — held through step 2. It broke at step 3. The extraction closes that gap at exactly the right level: `HtmlAnalysis` is pure, `theater_violations` takes only `HtmlAnalysis`, and a test needs no mock, no stub, and no network.*
