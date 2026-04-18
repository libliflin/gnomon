# Domain Boundaries

Gnomon operates across several domains of knowledge. A bug that looks like "gnomon is wrong about X" might actually be "the spec says something different from what gnomon assumes" or "the CDN behavior didn't match the HTML analysis." This is the "who to ask about what" guide.

---

## Domain 1: HTML parsing and resource detection

**What it covers:** How browsers parse HTML and determine what resources to load. Which resources are render-blocking. How `<link rel>` values are interpreted. How `<script async/defer>` affects execution.

**Authoritative source:** [WHATWG HTML Living Standard](https://html.spec.whatwg.org/), especially the "Parsing HTML documents" and "Scripting" sections. MDN is a reliable secondary source.

**Where it lives in gnomon:** `src/analyze.rs`. `analyze_html` and `HtmlAnalysis`.

**Boundary confusion:** "Is `<link rel=preload as=style>` render-blocking?" The HTML spec says preload doesn't block render. Gnomon treats `preload as=style` as render-blocking because it's used to smuggle a render-blocking stylesheet past Lighthouse. This is gnomon's intentional interpretation, not the spec's. (See anti-theater section.)

---

## Domain 2: Anti-theater detection

**What it covers:** Known patterns that make performance measurement tools report better numbers than users actually experience. The list is in `PLAN.md §4` and `src/forbidden.rs`.

**Authoritative source:** PLAN.md §4 (gnomon's own list). The performance engineering community's documented tricks.

**Where it lives in gnomon:** `theater_violations` in `src/audit.rs`. Detection fields in `HtmlAnalysis`.

**Boundary confusion:** "Is this pattern always theater, or only sometimes?" Gnomon's answer is: if the pattern is present, it's theater. There are no gray cases in gnomon's model. See PLAN.md §4: "Fails by default. Override requires justification." The champion should not propose checks that have legitimate false-positive cases unless the check includes an override mechanism.

---

## Domain 3: HTTP fetch behavior and brotli recompression

**What it covers:** How browsers fetch resources, what Content-Type headers mean, how brotli compression is computed. The difference between wire bytes (what the browser downloads) and raw bytes (uncompressed content size).

**Authoritative source:** [RFC 7231](https://datatracker.ietf.org/doc/html/rfc7231) (HTTP/1.1 semantics), [RFC 7932](https://datatracker.ietf.org/doc/html/rfc7932) (brotli).

**Where it lives in gnomon:** `src/fetch.rs`. `Fetcher`, `Fetched`, `brotli_bytes` field.

**Boundary confusion:** CDNs often serve pre-compressed responses. Gnomon recompresses everything itself with the `brotli` crate rather than trusting the CDN's `Content-Encoding`. This means gnomon's `brotli_bytes` is a conservative upper bound — it represents what gnomon can achieve, not what a well-tuned CDN would achieve. The design choice is intentional: "we do not trust the CDN."

---

## Domain 4: SARIF 2.1.0

**What it covers:** The Static Analysis Results Interchange Format used by GitHub code scanning.

**Authoritative source:** [SARIF specification](https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html). GitHub's [SARIF documentation](https://docs.github.com/en/code-security/code-scanning/integrating-with-code-scanning/sarif-support-for-code-scanning).

**Where it lives in gnomon:** `src/report.rs`, `print_sarif`.

**Boundary confusion:** Gnomon violations are page-level (a URL), not line-level (a file + line number). GitHub code scanning renders them as workflow annotations, not inline PR diff comments. This is correct behavior, but confusing to users who expect inline comments. The README documents this explicitly.

---

## Domain 5: Web Vitals (LCP, CLS, INP, TTFB)

**What it covers:** The Core Web Vitals metrics that Google uses for ranking and that users care about.

**Authoritative source:** [web.dev/vitals](https://web.dev/vitals/). The Chrome team's documentation.

**Where it lives in gnomon:** Currently only in preset threshold values (`Budget.preset.vitals`). PLAN.md §7.2 describes predicted vitals (derived from static analysis); §7.3 describes measured vitals (headless Chromium). Neither is implemented in v0.0.2.

**Boundary confusion:** Gnomon's `insley` preset specifies LCP ≤ 800ms, CLS = 0.00, INP ≤ 50ms. These thresholds are gnomon's own standards, not Google's "Good" thresholds (which are more lenient). When a site fails gnomon's vitals budget but passes Google's "Good" threshold, gnomon is correct by design — gnomon is stricter than Google.

---

## Domain 6: Rust ecosystem conventions

**What it covers:** Rust 2024 edition features (let-chain syntax), Clippy conventions, error handling patterns.

**Authoritative source:** Rust Reference, Clippy documentation.

**Where it lives in gnomon:** All source files. Let-chain (`if let X && let Y`) is used throughout after cycle 1's clippy fixes.

**Boundary confusion:** The `anyhow` crate is used for error handling in `audit_url`. `thiserror` is in `Cargo.toml` but may not be used yet. New error types should use `thiserror` if they need to be matched programmatically; `anyhow` for top-level error propagation.
