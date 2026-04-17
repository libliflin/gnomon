# Gnomon

> A **gnomon** is the blade on a sundial — the part that casts the shadow. It doesn't tell
> you the time; it reveals it. Without the gnomon, the dial is just a painted disc.

Gnomon is a performance-budget auditor for static sites, written in Rust, designed as a
CI gate. It does not produce a score out of 100. It produces pass or fail.

It is unapologetically opinionated. It ships hostile defaults. It fails closed. It
sacrifices developer time — yours and ours — to protect the user's bandwidth, battery,
and CPU. If that is not the tradeoff you want, use another tool.

---

## 1. Thesis

The modern web has lower standards than we do. Lighthouse scores in the 80s are
celebrated. 3-second LCPs are "fine." 2MB pages with 80 requests are "industry average."
McMaster-Carr serves a fully functional industrial catalogue in under 300 KB with a
sub-500 ms TTI on a cold cache. That standard is not exotic; it is simply enforced.

Gnomon's claim is that enforcement is the whole game. Measurement without enforcement
produces dashboards nobody reads. Enforcement without speed produces tools nobody runs.
Enforcement that negotiates with you is already lost. Gnomon is therefore (a)
opinionated about what "good" means, (b) small and fast enough to run on every commit
without friction, (c) structurally designed so that lowering the bar leaves a paper
trail, and (d) willing to call the tricks people use to game other tools what they are:
theatre.

**Design principle:** sacrifice developer time to protect client experience. Every
tradeoff — in gnomon itself and in projects that adopt it — resolves toward the user's
bandwidth, battery, and CPU. There is no "easy mode." There is no "gradual adoption."
There is the standard and there is failure.

---

## 2. What gnomon is (and isn't)

**Is.** A single static Rust binary that reads a directory of built static assets (or
crawls a deployed URL), compares them against a declarative budget file checked into the
repo, and exits 0 or 1. Emits JSON, SARIF, and human output.

**Is.** The turnstile between your build and your users.

**Isn't a complement to Lighthouse in CI.** Lighthouse in CI is waste heat: 30 seconds
of Chromium boot to tell you what a static parse could have answered in 3 milliseconds,
dressed up as a score everyone will then game. Gnomon replaces Lighthouse in CI
outright. Keep Lighthouse on your laptop for once-a-quarter exploratory inspection if
you like; don't run it per-commit.

**Isn't a general web-quality tool.** No accessibility audits, no SEO scoring, no PWA
badges, no best-practices checklist. Scope discipline is how it stays fast. Other tools
are good at other things; gnomon is good at one.

**Isn't a scoring system.** There is no 0–100 number. Scores are gamed; budgets are
met. A site either meets its budgets or it does not.

**Isn't negotiable.** There is no `--warn-only` flag. There is no `--soft-fail` flag.
There is no `--except-on-tuesdays` flag. A violation is a failure. If you need to
loosen a budget, edit the budget file in a PR, with a justification, with an expiry.
That is the only path.

---

## 3. The standards

Gnomon ships two presets. `insley` is the default. `mcmaster` is the gentle version.

### 3.1 `insley` (default)

The standard for a site that treats its users' devices as sovereign hardware borrowed
for a moment, not a free compute pool to rent.

| Metric                     | `insley`            | Notes                                          |
|----------------------------|---------------------|------------------------------------------------|
| HTML (brotli)              | **8 KB**            | Single round-trip territory                    |
| CSS total                  | **14 KB**           | Inlined where possible; ≤1 external file       |
| JS total (parsed)          | **0 KB**            | Zero by default. Any byte requires justification. |
| Fonts                      | **0 files**         | System stack by default; WOFF2 subset if any   |
| Images, above fold         | **80 KB**           | AVIF or WebP; PNG/JPEG fail                    |
| Total wire bytes           | **100 KB**          |                                                |
| Requests                   | **≤ 6**             |                                                |
| Third-party domains        | **0**               | Every external host requires justification     |
| Render-blocking resources  | **0**               | Inline critical CSS or fail                    |
| TTFB (p75, cable)          | **100 ms**          |                                                |
| LCP (p75, cable)           | **800 ms**          |                                                |
| CLS                        | **0.00**            | Zero. Not "good." Zero.                        |
| INP (p75)                  | **50 ms**           |                                                |
| JS main-thread parse       | **0 ms**            | Zero JS → zero parse                           |

### 3.2 `mcmaster`

Table stakes. If you can't meet these, you are not in the conversation.

| Metric                     | `mcmaster`          |
|----------------------------|---------------------|
| HTML (brotli)              | 10 KB               |
| CSS total                  | 20 KB               |
| JS total (parsed)          | 50 KB               |
| Fonts                      | 60 KB, ≤ 2 files    |
| Images, above fold         | 150 KB              |
| Total wire bytes           | 300 KB              |
| Requests                   | ≤ 20                |
| Third-party domains        | ≤ 2                 |
| Render-blocking resources  | ≤ 1                 |
| TTFB                       | 200 ms              |
| LCP                        | 1.5 s               |
| CLS                        | 0.05                |
| INP                        | 100 ms              |
| JS main-thread parse       | 100 ms              |

Projects can loosen these per route with justification (§9). They cannot loosen them
silently. Both presets ship the full anti-theater ruleset (§4) and the forbidden list
(§5). Preset choice controls numbers, not disposition.

---

## 4. Anti-theater

Performance tools are routinely gamed. The tricks are well-known; practitioners treat
them as craftsmanship. Gnomon treats them as failures. This list grows as new tricks
appear.

**Fails by default. Override requires justification.**

- **Lazy-loaded LCP element.** `loading="lazy"` on the largest above-fold image inflates
  Lighthouse's LCP by deferring the timing signal. It also breaks the user's experience.
  Fail.
- **Hidden-until-interaction payloads.** JavaScript gated behind `requestIdleCallback`,
  `setTimeout`, or first user interaction does not show up in LCP but still costs the
  user bytes and battery. Any script not counted is still counted by gnomon.
- **Prerender / speculation-rules for the home page.** Speculative prefetch is fine.
  Prerendering your own homepage so that Lighthouse measures a warm cache is not. Gnomon
  audits cold-cache byte cost, always.
- **Client-side-rendered shells with a splash screen.** The user sees motion; they see
  no content. LCP ticks; the site is blank. Fail.
- **Service workers that stash the critical path.** Fine for repeat visits, lie for
  first visits. Gnomon measures the first visit.
- **Font-display: swap hiding FOIT under measurement windows.** Unless paired with a
  system-stack fallback in the first paint, this is a stall disguised as a save.
- **Hydration-heavy frameworks that report TTI before hydration completes.** Gnomon's
  TTI definition includes main-thread-idle for 50 ms *after* hydration finishes, not
  before.
- **Synthetic "above the fold" that is 90 vh of hero image.** Above-fold budgets count
  against the first 800 px on mobile, period.
- **`<link rel="preload">` used to hide render-blocking resources from Lighthouse.** We
  count all resources that must arrive before first paint, regardless of their `rel`.
- **Third-party scripts deferred until interaction.** Still counted. Still part of your
  total.
- **CDN-recompressed HTML that is 4 KB wire but 2 MB inflated.** We audit inflated size
  too.
- **"The budget only applies above the fold."** No. Budgets apply to everything the user
  downloads. Below-fold content affects battery and data plans.

Anti-theater rules are not configurable off. You can justify a specific violation per
route; you cannot disable the rule.

---

## 5. Forbidden by default

A maintained blocklist of scripts and domains that gnomon fails on contact. These have
repeatedly demonstrated that their byte cost, privacy cost, or performance cost is not
worth the value they claim to deliver. Users of gnomon can add to this list, with a
justification, for their own context. They cannot silently remove entries; doing so
requires `allowlist.justification` with an expiry.

Initial list (non-exhaustive, maintained in `gnomon-data/forbidden.toml`):

- `www.googletagmanager.com/gtm.js` — tag manager; opens the door to anything
- `assets.adobedtm.com/*` — Adobe DTM; same
- `connect.facebook.net/*` — Meta pixel
- `cdn.segment.com/analytics.js/*` — Segment; usually fires N more things
- `widget.intercom.io/*`, `static.intercom.io/*` — chat widgets (use a mailto link)
- `*.hotjar.com`, `*.fullstory.com`, `*.mouseflow.com` — session replay
- `fonts.googleapis.com` — self-host your fonts, subset them
- `code.jquery.com`, `ajax.googleapis.com/*/jquery/*` — if you still need jQuery,
  justify why
- `polyfill.io` — compromised 2024, untrusted by default
- `cdn.taboola.com`, `*.outbrain.com` — sponsored-content widgets
- Any `*.chatgpt.com` or `*.openai.com` embed that isn't explicitly justified
- `*.doubleclick.net`, `pagead2.googlesyndication.com`, `*.criteo.com` — ad networks

Seeing any of these fetched at runtime is a hard failure. The block is not a warning.

---

## 6. Fail-closed disposition

Gnomon's defaults are hostile. That is on purpose. Hostile defaults push the cost of
error onto the person trying to bypass a rule, not the user who suffers when the rule is
silently dropped.

- **Missing `gnomon.toml` fails.** No config, no audit, no build. Run `gnomon budget
  init` explicitly.
- **Unknown keys in `gnomon.toml` fail.** No silent ignores. A typo in `max_sze` does
  not quietly pass. This is rejected at load time.
- **Missing routes fail.** If `gnomon.toml` names `/checkout` and the build has no
  `/checkout`, we fail. Budgets are a contract.
- **Unmatched routes fail.** If the build has `/about` and no budget matches, we fail.
  Every route is covered or explicitly excluded with `[routes."/about"] exclude = true`
  (which itself requires justification).
- **Predicted vs measured disagreements fail toward caution.** If prediction passes but
  the measured value is within 10% of budget, the build fails and requires
  `--measure` to confirm. We never round down.
- **Relaxed budget without justification fails.** Any per-route override that is
  *looser* than `[global]` without a `justification` + `justification_expires` field
  fails at load time.
- **Expired justification fails.** No grace period. The day it expires, the build
  breaks. Renewals require a new PR.
- **Stale budget file fails.** If the budget file was not touched for 180 days, gnomon
  prints a warning on every run and, after 365 days, fails. Budgets must be tended.
- **Shorter justifications are the only way to loosen.** Max expiry is 90 days
  (`justification_expires` must be within 90 days of the current HEAD commit date).
  Longer is rejected.
- **No `--warn-only` mode.** Does not exist. Will not be added. A PR proposing it will
  be closed.
- **No `--skip-check` flag.** Individual checks cannot be disabled. Override a specific
  violation via the budget file or don't.
- **`--no-verify`-style environment escapes are not honored.** The tool has no "trust me,
  I'm in a hurry" switch.

Hostile defaults are the *product*. If they feel rude, it is because they are doing
their job.

---

## 7. What gnomon measures

Three concentric rings, cheapest first. A run stops at the cheapest ring that answers
the question.

### 7.1 Static analysis (always, fast)

Parse the built output. No browser, no network. Single-digit milliseconds per page.

- HTML byte size: raw, gzip, brotli (we recompute transfer size; we do not trust the
  CDN)
- Total JS bytes: sum of `<script src>` + inline + modulepreload chains, post-minify,
  with hidden-payload detection (§4)
- CSS bytes: external + inline, with unused-selector detection via lightningcss
- Font count, byte size, format; reject TTF/OTF where WOFF2 would serve
- Image count, byte size, format; reject JPEG/PNG where AVIF/WebP exists; reject any
  LCP candidate with `loading="lazy"`
- Missing `width`/`height` on `<img>` (CLS risk) — counted, not just flagged
- Missing `loading="lazy"` below the fold (below-fold = after the first 800 px of
  rendered HTML, not after some arbitrary byte offset)
- Render-blocking resources in `<head>` (synchronous `<script>`, non-`preload` `<link>`,
  `<link>` with `preload` and `onload=` that loads render-critical CSS — the trick in
  §4)
- Third-party hostname count across all asset URLs, including those fetched
  speculatively or deferred
- Preload/preconnect hint count and targets (preconnect to third parties counted as
  third-party contact)
- Inline style and script budget (inlining is fine; hiding bytes is not)
- `<meta http-equiv>` misuse, missing `viewport`, missing `charset` in first 1024 bytes
- Forbidden script / domain match (§5)
- Cache headers on served assets: `immutable`, `max-age` ≥ 1 year for
  fingerprinted assets, `must-revalidate` for HTML

### 7.2 Predicted vitals (always, fast)

Derived from the static analysis. Produces conservative upper bounds. Wrong in the
generous direction by design — if the prediction says you're over, you're over.

- **Predicted LCP** = critical-path byte cost / assumed bandwidth (Slow 4G: 1.6 Mbps) +
  RTT × critical-path depth. Critical path = HTML → render-blocking CSS → LCP element.
- **Predicted CLS upper bound** = Σ (unreserved slot dimensions normalized to viewport)
- **Predicted JS parse time** = total JS bytes × 10 KB/s (Moto G4 baseline)
- **Predicted TTFB** = constant per-route, falls back to measurement if unknown

If the prediction is within 10% of the budget, the build fails and asks for
`--measure`. This is the fail-closed rule from §6 in action.

### 7.3 Measured vitals (`--measure`)

Headless Chromium via `chromiumoxide`, network-throttled (Slow 4G), CPU-throttled (4×),
cold cache, no service worker, no prerender, no speculation rules. This is the slow
path: multi-second per page. Used pre-merge on a protected branch, or when prediction
is ambiguous.

Produces real LCP / CLS / INP / TTFB, unused-bytes coverage, long-task counts, and a
hydration-complete marker for frameworks that hydrate. TTI is defined as 50 ms of main-
thread idle *after* hydration completes, not before (§4).

---

## 8. The budget file

`gnomon.toml`, checked into the repo root. Source of truth.

```toml
preset = "insley"            # required. "insley" or "mcmaster".

[global.bytes]
html      = "8KB"            # brotli
css       = "14KB"
js        = "0B"
images    = "80KB"
fonts     = "0B"
total     = "100KB"

[global.count]
requests            = 6
third_party_domains = 0
render_blocking     = 0
fonts               = 0

[global.vitals]
lcp  = "800ms"
cls  = 0.00
ttfb = "100ms"
inp  = "50ms"

[global.predict]
js_parse_budget = "0ms"
bandwidth       = "slow-4g"
cpu_throttle    = 4

# Forbidden list inherited from gnomon-data/forbidden.toml at pin.
# To allow a blocked resource locally, add an allowlist entry with justification.
[[allowlist]]
pattern               = "fonts.googleapis.com"
justification         = "stage rollout; self-hosting lands ENG-1321"
justification_expires = "2026-07-10"    # ≤ 90 days from HEAD.

# Per-route overrides. Stricter is allowed silently; looser requires justification.
[routes."/checkout"]
js                    = "90KB"
justification         = "stripe.js required for PCI; see ENG-1234"
justification_expires = "2026-07-01"
```

**Pattern matching.** Route keys accept exact paths, globs (`/blog/*`), and regex
(`~ ^/product/\d+$`). Most-specific match wins.

**Inheritance.** Per-route blocks inherit from `[global]`. You only write what differs.

**Pinning.** The forbidden list is fetched from a versioned URL at build time and
pinned by SHA-256 in `gnomon.lock`. Updates require an explicit `gnomon budget update`
that diffs the list and asks you to acknowledge new entries.

**Failure modes at load time** (§6): unknown keys fail, missing `preset` fails, unsigned
overrides fail, over-90-day expiries fail, mixing `preset = "mcmaster"` with insley-
level per-route values silently is fine (stricter is always fine; relaxing requires
justification).

---

## 9. The ratchet

`gnomon budget tighten` rewrites the budget file to match the current build minus a
**1 % safety margin**. Run it after a perf win lands. The new, lower numbers get
committed.

1 % is deliberate. 5 % would be a comfortable cushion; 1 % is barely a cushion. The
intent is that the ratchet is painful to walk back — every regression has to raise a
ceiling that was set at the exact edge of what was shipped.

`--margin` is overridable but capped at 5 %. Anything higher is rejected at CLI parse
time.

In most projects, budgets drift upward because nobody has time to argue about 3 KB of
new JS. With the ratchet, budgets drift downward and stay there. A PR that regresses
performance has to explicitly raise a ceiling someone else already lowered, leave a
note, and set an expiry.

---

## 10. Justifications with expiries

The philosophical linchpin. A budget can be loosened — bytes, count, vital — only with:

```toml
justification         = "human-readable reason, linking a ticket"
justification_expires = "YYYY-MM-DD"       # ≤ 90 days from HEAD commit date
```

**Hard rules:**

- Max expiry: 90 days. Beyond that is rejected at config load.
- No expiry: rejected.
- Empty or placeholder justification (`"TODO"`, `"temporary"`, `"fix later"`): rejected
  via a maintained list of disallowed strings.
- Expiries must be calendar dates; "30d" or "+60" is not accepted — reviewers need to
  see the exact date.
- A justification may be renewed, but the renewal PR must alter the `justification`
  string to explain *why it's still necessary*. Verbatim renewals are rejected.

Gnomon fails the build when an expiry passes. No grace. No warning period. The day it
expires, the build breaks. That is the incentive: the next engineer who would otherwise
leave your exception in place has to either fix the underlying issue or explain to a
reviewer, in fresh prose, why the exception still holds.

You cannot quietly become McKinsey's website.

---

## 11. CLI surface

```
gnomon audit [--dir DIR | --url URL]         # run all checks, exit 0/1
gnomon audit --fast                          # static + predicted only, no browser
gnomon audit --measure                       # force headless browser
gnomon audit --json | --sarif | --human      # output format
gnomon audit --diff HEAD~1                   # compare against base ref's build

gnomon budget init [--preset=insley|mcmaster|current]
gnomon budget tighten [--margin=1%]          # default 1%, cap 5%
gnomon budget check                          # validate gnomon.toml, fails on unknown keys
gnomon budget update                         # refresh pinned forbidden list
gnomon budget explain <violation-id>         # show the rule + why it matters

gnomon watch [--dir ./dist]                  # re-audit on file change, < 10 ms feedback
gnomon crawl --url https://example.com       # enumerate routes, audit each
gnomon ci                                    # short for `audit --fast --sarif`
gnomon doctor                                # self-check: binary, perms, browser presence
```

**Defaults.** `CI=true` → `--fast` + `--sarif`. TTY → `--human`. JSON output schema is
versioned; breaking changes bump the major version.

**Flags that do not exist and will not be added:**

- `--warn-only`
- `--skip <check>`
- `--allow-regressions`
- `--soft-fail`
- `--bypass`
- `--force`

If a user wants any of these, they are looking for a different tool.

---

## 12. CI integration

### 12.1 GitHub Action (primary)

```yaml
- uses: libliflin/gnomon-action@v1
  with:
    directory: ./dist
    base-ref: main
    sarif: true            # upload to GitHub code scanning, inline PR annotations
    comment: on-change     # only post a comment when budget state changes
```

The Action downloads a precompiled static binary keyed to the runner platform. No
cargo, no Node. Cold-start-to-result for a medium site: target **≤ 1 second** of the
Action's total time.

PR comment shows a table: metric, old, new, budget, delta, state. No 47-line report.
No emoji dashboards. One table, monospace, ≤ 20 rows.

### 12.2 SARIF + code scanning

Violations emit SARIF 2.1.0. GitHub's native code-scanning UI renders each violation
inline, at the offending line in the offending file. This is the moat: users see
regressions without leaving the diff view. No custom annotator code.

### 12.3 Generic CI

```sh
gnomon ci                     # exit 0/1, SARIF to stdout, silent on pass
gnomon audit --fail-on=any    # fail on any violation (default)
gnomon audit --fail-on=new    # fail only on newly introduced violations
```

### 12.4 Pre-commit

```sh
gnomon audit --fast --diff HEAD
```

`--fast` completes in under 50 ms for most repos — fast enough that nobody disables it.
This is the load-bearing claim. If it runs slower than typecheck, people turn it off,
and the tool fails.

---

## 13. Performance budget for gnomon itself

Eat the dogfood. The binary has its own budgets, tested in CI:

| Operation                                    | Target        |
|----------------------------------------------|---------------|
| Cold start → first output (no-op audit)      | **< 10 ms**   |
| Static audit of one HTML page                | **< 1 ms**    |
| Full crawl + static audit of 100-page site   | **< 100 ms**  |
| Full crawl + static audit of 1000-page site  | **< 1 s**     |
| Peak RSS, 1000-page site                     | < 30 MB       |
| Binary size (stripped, release, x86_64)      | < 4 MB        |
| Measured audit (Chromium), per page, p95     | < 3 s         |
| Binary start-up syscalls                     | < 200         |
| Allocations per HTML page audit              | < 500         |

Checked by `cargo bench` and by an end-to-end timing gate in CI. Any regression > 5 %
fails the build. Any `unsafe` block outside `gnomon-measure` requires a justification
comment pointing at the microbenchmark that motivated it.

The tool audits its own build time. That is not a joke.

**How we get there (non-exhaustive):**

- No dynamic dispatch in hot paths. Generics and monomorphization.
- One allocation per page for the HTML blob; reuse buffers across pages via
  thread-local pools in the rayon worker.
- `lol_html` in streaming mode: never materialize the full DOM.
- Precomputed ASCII-range lookup tables for attribute-name matching.
- No regex in hot paths; hand-rolled `memchr`-based scanners.
- No format-string allocation in the violation emitter; writes go into a thread-local
  `BumpAlloc`.
- The forbidden list is compiled to an Aho-Corasick automaton at startup, reused.
- CI timing gate uses a median of 9 runs, discards top and bottom two.

---

## 14. Architecture sketch

```
crates/
  gnomon-core       // types: Budget, Violation, RouteAudit, Report
  gnomon-static     // HTML/CSS/JS/font/image analysis, no I/O
  gnomon-predict    // predicted-vitals calculators
  gnomon-antitheater // theatre-detection rules (§4)
  gnomon-measure    // headless browser driver (feature-gated)
  gnomon-crawl      // URL enumeration, robots-aware
  gnomon-report     // human, JSON, SARIF emitters
  gnomon-cli        // clap; thin; shells out to the above
  gnomon-data       // shipped data: presets, forbidden list, bench corpora
```

**Key dependencies (selected for speed, not ergonomics):**

- `lol_html` — streaming HTML rewriter, faster than scraper for read-only work
- `lightningcss` — CSS parse + unused-selector analysis, Parcel's engine
- `oxc_parser` — fastest Rust JS parser; used for bundle-size accounting
- `imagesize` — header-only image dimension probe, no decode
- `brotli` / `flate2` — recompute transfer size ourselves
- `woff2` — font parsing and subsetting inspection
- `aho-corasick` — forbidden-list automaton
- `rayon` — data-parallel file walks
- `tokio` + `hyper` — only for `--url` crawl mode (feature-gated)
- `chromiumoxide` — headless browser (feature = "measure", off by default)
- `serde` + `toml` — config
- `clap` derive — CLI
- `miette` — error rendering with source spans
- `serde_sarif` — SARIF output

**Parallelism model.** `rayon` for static-analysis fan-out across routes. One page per
thread, all checks on a page run sequentially on that thread — cache locality matters
more than per-check parallelism at this scale. Network crawl is `tokio` with a bounded
semaphore.

**Allocator.** System allocator by default. Optional `mimalloc` feature for deployments
that care about every microsecond.

---

## 15. Roadmap

### v0.1 — the core loop (target: 4 weeks)

- `gnomon audit --dir` with the full static-check set (§7.1)
- `gnomon.toml` parser with full fail-closed load-time validation (§6)
- Anti-theater rules (§4), first pass: lazy LCP, render-blocking-via-preload,
  hidden-payload detection
- Forbidden list (§5), pinned in `gnomon.lock`
- Both presets (`insley`, `mcmaster`)
- Human + JSON output
- Precompiled binaries for Linux x86_64/arm64, macOS x86_64/arm64, Windows x86_64
- gnomon's own perf budget (§13) gated in CI from day one

### v0.2 — the ratchet and the trail

- `budget tighten` with 1 % default margin
- Justifications with expiries, 90-day cap, renewal-requires-new-prose
- SARIF output
- GitHub Action v1
- Diff mode against a base ref

### v0.3 — prediction

- Predicted LCP / CLS / JS parse time
- Critical-path walker
- `--diff` shows predicted-vitals deltas, not just byte deltas
- Fail-closed 10 % prediction margin (§7.2)

### v0.4 — measurement

- `--measure` via chromiumoxide (feature-gated)
- Real LCP, CLS, INP, TTFB, coverage
- Auto-escalation from predicted to measured
- Hydration-complete marker for TTI

### v1.0 — crawl + watch

- `gnomon crawl --url` with robots + sitemap support
- `gnomon watch` for dev loop, < 10 ms feedback
- Stable JSON + SARIF schemas, semver guarantees
- Shipped with the `insley-web` reference site as an integration test
- Public perf leaderboard of opted-in projects, ranked by violations per kLOC

---

## 16. Non-goals

- **Scores.** No 0–100 numbers, now or ever. The budget is the score.
- **Accessibility.** Different problem, different tool.
- **SEO.** Different problem, different tool.
- **SSR-vs-CSR religion.** Gnomon audits output, not process. Next.js and a static HTML
  file both pass or fail on the same byte counts.
- **Framework plugins.** We do not ship Webpack / Vite / Turbopack / Next / Astro
  integrations. Framework authors can call gnomon; we will not chase them.
- **Hosted SaaS.** Gnomon is a binary. If someone wants a dashboard, they can pipe
  `--json` into one.
- **Auto-fix.** Gnomon tells you what's over. It will not rewrite your code. Taste
  lives in the implementation; enforcement lives in gnomon.
- **"Easy mode" / gradual adoption.** There is no `--relaxed`, no `--starter`, no
  `--migration-friendly`. Projects that cannot meet `mcmaster` do not use gnomon. The
  tool earns its authority by never negotiating.
- **Warnings.** There are violations and non-violations. There is no yellow state.
- **Retries.** A failed build is a failed build. No `gnomon audit --retry-on-flake`.
  Measurement flake is addressed by running a deterministic measured path, not by
  papering over noise.
- **Plugin API.** Rules are in-tree. Users can propose rules via PR. No per-project
  JavaScript extensions, no WASM rule packs, no "bring your own check." Opinionation is
  the product.

---

## 17. Positioning inside insley-web

insley-web is the thesis: the modern web has decayed, and a small group of craftsmen
can rebuild a better one with old-fashioned discipline. The thesis needs three things
to move from essay to movement:

1. **A reference implementation** — the insley-web site itself.
2. **A canonical benchmark** — McMaster-Carr, Gov.UK, the old Craigslist.
3. **Tooling that makes the standard enforceable** — this is gnomon.

Without gnomon, "respect your users' bandwidth" is a vibe. With gnomon, it is a CI
gate. The tool's job is to drag the vibe into the build pipeline so it survives contact
with deadlines, new hires, quarterly planning, and the pull toward mediocrity that
every codebase experiences in the absence of an external force.

Gnomon is that external force.

---

## 18. Naming

A gnomon is the shadow-casting blade of a sundial. It measures by revealing what was
always there — the sun's position — in a form a human can act on. The sundial's face
would be inert without it.

The tool is pronounced with a silent *g*: **NO-mon**. Ancient, exact, cheap, and
correct for a thousand years without maintenance. That is the brief.
