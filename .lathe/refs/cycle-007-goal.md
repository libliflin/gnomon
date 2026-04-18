# Goal — Cycle 7

## What

Add tests to `src/forbidden.rs` that establish the testing pattern for `ForbiddenMatcher`. At minimum, pin:

1. **A known forbidden URL matches** — `ForbiddenMatcher::new().find("https://fonts.googleapis.com/css2?family=Roboto")` returns `Some(("fonts.googleapis.com", "Google Fonts — self-host and subset your fonts"))`. The full URL contains the pattern; the pattern and reason tuple are correct.

2. **A clean URL returns None** — a URL with no forbidden substrings returns `None`. Pins that the automaton does not over-fire.

3. **Matching is case-insensitive** — `FONTS.GOOGLEAPIS.COM` in a URL still fires. The automaton is built with `ascii_case_insensitive(true)`; without a test, this is an implementation claim the contributor cannot verify.

4. **Pattern is substring match, not equality** — the URL can be any length; the pattern only needs to appear anywhere in it. A contributor adding `"evil-tracker.example.com"` needs to know their entry does not require an exact URL match.

5. **At least two distinct entries each match** — e.g. GTM and DoubleClick both fire on their respective URLs. Confirms the automaton compiles all patterns correctly, not just the first one.

The test module belongs in `src/forbidden.rs` under `#[cfg(test)] mod tests { ... }`, following the pattern already established in `analyze.rs` and `audit.rs`. No changes to the FORBIDDEN list, no new public API, no functional changes — only test code.

## Which Stakeholder

**The contributor** (stakeholder 4). Not served since cycle 2 — the longest wait of any stakeholder.

Step 8 of their journey: "Try to add a trivial check and verify it fires." The most atomic contribution to gnomon is adding a new entry to the FORBIDDEN list in `src/forbidden.rs`. The file is 62 lines and immediately readable. The contribution is a two-string tuple. But there is no test in that file. The contributor cannot verify their addition fires correctly without running `gnomon audit` against a live URL that loads the forbidden domain. That is not testing — that is manual verification against the internet.

## Why Now

`analyze.rs` has 50+ tests. `audit.rs` has 18+ tests for its helper functions. `forbidden.rs` has zero.

The forbidden list is a first-class enforcement mechanism — it is what gnomon says when it rejects ad networks, session replay trackers, and compromised CDNs. For a CI gate that bills itself as "precision, certainty, and no apology," shipping an enforcement list with no verification tests is the most off-brand gap remaining.

The specific moment: I added a new entry to FORBIDDEN. Build clean. Clippy clean. Then I looked for a test to prove the automaton picked it up. Nothing. I did not know whether:
- The pattern needed to be the exact URL (it doesn't — it's substring)
- Case sensitivity mattered (it doesn't — `ascii_case_insensitive(true)` is set)
- The automaton was built correctly (it compiles at runtime; no compile-time check)

Without tests, every contributor who touches `forbidden.rs` is flying blind. With tests, the pattern is established: add an entry, add a test, done.

The fix lives entirely in `src/forbidden.rs` — no other files need to change. The blast radius is zero. The signal to the next contributor is immediate.

## Lived-Experience Note

*I became the contributor. I cloned, built clean, ran clippy — all green. `cargo test` showed 74 passing: confidence building. I opened `analyze.rs` — 50+ tests, every rule pinned, easy to copy. I opened `audit.rs` — helpers tested, detail formatters all verified.*

*Then I opened `forbidden.rs`. Sixty-two lines. Clean, tight, readable. I understood it in 30 seconds. I found the FORBIDDEN array. I found the place to add a new entry. I added `("evil-tracker.example.com", "fictional tracker — replace with real reason")`. I built. I ran clippy. Both passed.*

*Then I looked for a test.*

*Nothing.*

*I knew the ForbiddenMatcher uses Aho-Corasick with `ascii_case_insensitive(true)` — I could read it. But I did not know if my pattern would fire in practice. Does the pattern need to appear exactly? Does it match anywhere in the URL? What if I made a typo? The only way to know was to spin up a local server, serve a page that loads `evil-tracker.example.com/script.js`, and run `gnomon audit` against it.*

*That is not testing. That is asking the internet to be my test harness.*

*The clarity signal died there. I know exactly where to put the entry. I have no way to prove it works.*
