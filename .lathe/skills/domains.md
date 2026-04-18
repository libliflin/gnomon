# Domain Boundaries

Every non-trivial project spans multiple domains of knowledge. A bug that looks like "gnomon got the byte count wrong" might actually be "the CDN is serving a different encoding than the Content-Encoding header claims." This file maps the domains gnomon operates across: what each covers, what its authority is, and where the boundaries create confusion.

---

## 1. HTTP and Content Negotiation

**What it covers:** Fetching URLs, following redirects, reading Content-Type and Content-Encoding headers, handling transfer encoding (gzip, brotli, deflate). What does the browser actually receive?

**Authority:** RFC 9110 (HTTP semantics), RFC 7932 (Brotli), browser behavior for content negotiation.

**Boundary confusion:** A CDN may serve a resource with `Content-Encoding: br` but the body may be pre-compressed at a different level, or may not be brotli at all. Gnomon re-fetches and re-compresses bytes itself rather than trusting CDN headers. When byte totals seem off, check: is gnomon receiving the raw body or a pre-compressed body? Is it decompressing before re-compressing?

The `reqwest` client in `src/fetch.rs` handles decompression (brotli, gzip, deflate are enabled as features). The brotli compression for budget comparison is done separately. These are two different things — decompressing the wire bytes (reqwest) vs. recompressing the raw body to get the brotli size (gnomon's own computation).

---

## 2. HTML Parsing and Resource Extraction

**What it covers:** Extracting `<script>`, `<link>`, `<img>`, font references, preconnect hints from HTML. Classifying resources as render-blocking vs. not. Detecting which image is the likely LCP candidate.

**Authority:** HTML Living Standard, browser loading behavior (not parsing spec — actual browser load order).

**Boundary confusion:**
- A `<link rel="preload" as="style" onload="this.rel='stylesheet'">` is a render-blocking resource disguised as a preload hint. Gnomon treats it as blocking. A naive implementation counting `rel="preload"` as "not blocking" would miss this.
- A `<link rel="preconnect">` establishes an early connection but does not load a resource. It counts toward third-party domain exposure but not toward request count.
- The "first `<img>`" as LCP candidate is gnomon's heuristic (v0.0.2). The actual browser LCP candidate is determined by the browser's layout engine — gnomon's heuristic is conservative.
- `scraper` (the current HTML parser) builds a full DOM. This is correct but memory-intensive for large pages. It does not execute JavaScript, so dynamically-loaded resources are not detected. This is a known limitation.

---

## 3. Performance Budgets and Measurement

**What it covers:** Byte budgets (brotli-compressed transfer size), count budgets (requests, third-party domains, fonts, render-blocking), predicted vitals (LCP, CLS, TTFB, INP, JS parse time).

**Authority:** Web Vitals spec (Google), gnomon's own preset values (PLAN.md §3). The preset values are gnomon's design decisions, not industry standards.

**Boundary confusion:**
- "Byte budget" in gnomon means brotli-recompressed transfer size — not raw bytes, not gzip bytes, not CDN-reported bytes. This is the most compressed form of the resource, which is the most accurate measure of what the network actually costs.
- Lighthouse measures LCP using a headless browser with real throttling. Gnomon's predicted LCP (not yet implemented at v0.0.2) is a static computation from byte counts. They measure different things. "Gnomon says FAIL but Lighthouse says 95" does not mean gnomon is wrong.
- The `mcmaster` preset is named after McMaster-Carr (the industrial catalogue site) as a reference point. It is not affiliated with McMaster-Carr. The name is an aspiration, not an endorsement.

---

## 4. Anti-Theater Rules

**What it covers:** Detecting tricks that game performance measurement tools without improving actual user experience. The canonical examples: lazy-loading the LCP image, hiding JS payloads behind interaction, using preload hints to disguise render-blocking resources.

**Authority:** Gnomon's own ruleset (PLAN.md §4). No external spec or standard. The rules are gnomon's design decisions about what constitutes theater.

**Boundary confusion:**
- `loading="lazy"` is correct for below-fold images. On the LCP candidate (first image), it's theater. The distinction depends on which image is the LCP candidate, which gnomon approximates.
- `<link rel="preload">` is a valid performance optimization. `<link rel="preload" as="style" onload=...>` is the theater trick. The distinction is `as="style"` with an `onload` handler.
- Missing `<meta name="viewport">` is a Theater violation in gnomon (causes the browser to render at desktop width on mobile, affecting layout and LCP). It is not specifically a performance optimization trick — it's a missing meta tag that degrades mobile performance. Gnomon treats it as theater because it can be used to game mobile-specific audits.

---

## 5. Forbidden List (Third-Party Scripts)

**What it covers:** A curated list of scripts, domains, and URL patterns that gnomon fails on contact. Known trackers, tag managers, session-replay tools, ad networks, compromised CDNs.

**Authority:** Gnomon's own list (PLAN.md §5). Maintained subjectively by the project. Not a spec or standard.

**Boundary confusion:**
- Pattern matching is substring matching (Aho-Corasick), case-insensitive, against the full resource URL. `fonts.googleapis.com` matches any URL containing that string. Overly broad patterns would create false positives.
- The forbidden list does not distinguish between a site that uses GTM to fire one analytics tag and a site that fires 40 marketing pixels through GTM. Both fail. The rule is about the presence of the forbidden resource, not its configuration.
- `polyfill.io` is on the list because it was compromised in 2024 and served malicious scripts to users. This is a security-motivated ban, not purely a performance ban. The reason matters when explaining the violation to a user.

---

## 6. SARIF Output

**What it covers:** SARIF 2.1.0 format for GitHub code scanning. Enables violations to appear as annotations in the GitHub UI.

**Authority:** SARIF 2.1.0 spec (OASIS), GitHub code scanning API requirements.

**Boundary confusion:**
- Gnomon violations are page-level (a URL), not line-level (a source file + line number). SARIF's `physicalLocation.artifactLocation.uri` is set to the audited URL, not a file path. GitHub code scanning renders these as annotations on the workflow run, not inline PR diff comments.
- This is correct behavior, not a limitation. The README documents it: "Gnomon violations are page-level, not line-level, so GitHub surfaces them as annotations on the workflow run rather than inline PR diff comments." A user who expects inline diff comments will be confused — the explanation is in the README.
- The `ruleId` format is `{kind}/{metric}` (e.g., `theater/img_dimensions`, `bytes/css`). Duplicate kind/metric pairs produce one rule entry but multiple result entries. This is correct SARIF — a tool with a single rule can fire it multiple times.

---

## 7. Rust and the Crate Ecosystem

**What it covers:** The implementation language, Cargo build system, crate selection and versioning.

**Authority:** Rust reference, Cargo docs, individual crate documentation.

**Boundary confusion:**
- `scraper` uses CSS selectors for DOM traversal and builds a full DOM in memory. PLAN.md plans to replace it with `lol_html` (streaming, lower memory). Code changes in `analyze.rs` should be written with awareness that this migration is planned — avoid patterns that would be harder to port to a streaming model.
- `reqwest` with `rustls-tls` feature means no OpenSSL dependency. This is deliberate for the static binary distribution (musl builds).
- `aho-corasick` is the only non-standard choice in the hot path. It's there specifically for efficient multi-pattern substring matching in the forbidden list check. Do not replace it with a regex or a loop without measuring.
- `edition = "2024"` in `Cargo.toml` — this is Rust 2024 edition, which may have minor syntax differences from 2021. Be aware when reading code that uses `let ... else`, `if let` chains, etc.
