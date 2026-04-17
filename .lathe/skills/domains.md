# Domain Map

Gnomon operates across several domains of knowledge, each with its own authority. When a bug or design question arises, this map tells you which domain to consult first.

---

## Domain 1: Web Performance Metrics

**What it covers:** LCP, CLS, INP, TTFB, FCP, TBT, TTI. What these metrics measure, how browsers compute them, and what site behaviors move them.

**Authoritative source:** web.dev/articles/*, Chrome DevTools documentation, the Web Vitals JS library source. The W3C Long Tasks API and Layout Instability API specs.

**Where the boundary creates confusion:** Gnomon's "predicted LCP" is a static approximation of a browser-measured metric. A question like "why does gnomon's predicted LCP differ from Lighthouse's LCP?" lives at the boundary between this domain and Domain 2 (static analysis). The answer is almost always: Lighthouse measures; gnomon predicts from wire bytes. When they disagree, the browser measurement is ground truth.

---

## Domain 2: Static HTML/CSS/JS Analysis

**What it covers:** Parsing HTML documents, extracting resource references, classifying assets, detecting render-blocking patterns, measuring inline bytes.

**Authoritative source:** The HTML Living Standard (whatwg.org/html), the CSS spec, the JavaScript spec. For "what is render-blocking," the Chromium Preload Scanner source is more authoritative than any spec — browsers implement heuristics.

**Where the boundary creates confusion:** A `<link rel="preload" as="style">` is technically not render-blocking per spec, but Gnomon flags it as theater because real-world sites use it to hide CSS from render-blocking detection. The authority here is not the spec — it's the known gaming technique. When adding new checks, ask: "Is this spec-defined behavior, or a browser-specific heuristic, or a documented gaming trick?"

---

## Domain 3: HTTP and Network Behavior

**What it covers:** Transfer encoding, content negotiation, brotli/gzip compression, CDN behavior, redirect chains, CORS, cache headers.

**Authoritative source:** RFC 9110 (HTTP Semantics), RFC 7932 (Brotli), MDN documentation.

**Where the boundary creates confusion:** Gnomon recomputes brotli size from the raw body because CDNs lie about content-length. This is Domain 3 behavior. But the *budget* against which we compare is a Domain 2 number (byte budget for a resource type). When a byte budget check behaves unexpectedly, check whether the CDN is doing something unusual in the HTTP layer before assuming the analysis code is wrong.

---

## Domain 4: Rust and the Cargo Ecosystem

**What it covers:** Rust language semantics, the borrow checker, async/await with tokio, Cargo features and workspaces, crates.io publishing.

**Authoritative source:** The Rust Reference, the Rustonomicon, the Cargo Book, the Tokio documentation.

**Where the boundary creates confusion:** Gnomon uses `scraper` (which uses `html5ever`) for HTML parsing. `html5ever` follows the HTML Living Standard parsing algorithm — not just tokenization. A question about "why does gnomon detect this element differently than I expected" may be a Rust crate behavior question (Domain 4), an HTML parsing spec question (Domain 2), or a bug. Check the crate docs before assuming a spec mismatch.

---

## Domain 5: CI/CD and GitHub Actions

**What it covers:** GitHub Actions workflow syntax, SARIF 2.1.0 schema, GitHub code scanning integration, PR annotations, precompiled binary distribution.

**Authoritative source:** docs.github.com/actions, the SARIF 2.1.0 spec (Microsoft), GitHub's code scanning documentation.

**Where the boundary creates confusion:** SARIF violations require a "ruleId" and a "location" (file + line number). For gnomon, violations are detected at the *page* level (a URL), not at a specific source line. The question of how to map a "your total JS exceeds budget" violation to a SARIF location lives at the boundary of Domain 2 and Domain 5. The answer gnomon will use: point at `gnomon.toml` as the location when the budget is the contract, and at the offending HTML file when the HTML is the source.

---

## Domain 6: The insley-web Performance Standard

**What it covers:** The `insley` and `mcmaster` presets — what the numbers mean, why they were chosen, and what the standard is trying to enforce.

**Authoritative source:** `PLAN.md` in this repo. The insley-web reference site. The McMaster-Carr case study (referenced in PLAN.md §1).

**Where the boundary creates confusion:** PLAN.md describes what the standard *is*, not what it *measures*. A question like "should we count WebP images against the image budget using raw bytes or brotli bytes?" is partly Domain 6 (what the standard intends) and partly Domain 2 (how we measure). When standard intent and measurement differ, the standard intent wins — update the measurement.

---

## Quick reference: who to ask about what

| Question | Domain |
|---|---|
| Why does this LCP value differ from Lighthouse? | 1 — Web Performance Metrics |
| Is this `<link>` render-blocking or not? | 2 — Static HTML/CSS/JS Analysis |
| Why does gnomon's byte count differ from the CDN's? | 3 — HTTP and Network |
| Why does this async pattern cause a compilation error? | 4 — Rust/Cargo |
| How do I map a violation to a SARIF location? | 5 — CI/CD and GitHub |
| Should fonts count against the total byte budget? | 6 — insley-web Standard |
