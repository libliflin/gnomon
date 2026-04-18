# Domain Boundaries — Gnomon

Gnomon operates across several distinct domains. Each has its own authority. Bugs that look like they belong to one domain often trace to another — knowing the map prevents wasted effort.

---

## 1. Web standards (HTML/CSS/resource loading)

**What it covers:** What counts as "render-blocking," how brotli transfer size relates to content-type, what `loading="lazy"` does to LCP timing, how `<link rel="preload">` interacts with the render-critical path, what the first 800px viewport means for above-fold budgets.

**Authority:** WHATWG HTML spec, W3C CSS specs, MDN, web.dev. For LCP/CLS/INP definitions: web.dev/vitals and the Chrome team's explainers.

**Where confusion appears:** Gnomon's definitions of "render-blocking" and "above fold" are intentionally stricter than the browser's native behavior. This is by design (PLAN.md §4 anti-theater). A bug report like "this isn't actually render-blocking" may be correct by the browser spec but wrong for gnomon's purpose. The authority for gnomon's definitions is PLAN.md §4 and §7, not the spec alone.

---

## 2. HTTP semantics and transfer encoding

**What it covers:** Brotli vs. gzip vs. raw content, `Content-Encoding` headers, CDN-reported sizes vs. actual wire bytes, HTTP status codes, response timing.

**Authority:** RFC 7230–7235 (HTTP/1.1), RFC 7932 (Brotli), RFC 9110–9114 (HTTP/2 and HTTP/3 basics for awareness). In practice: the `reqwest` and `brotli` crate docs.

**Where confusion appears:** CDNs sometimes report compressed size in `Content-Length` that doesn't match what they actually delivered. Gnomon recompresses content itself to get a canonical brotli size. If byte totals seem wrong, suspect the fetch → brotli-recompression pipeline in `fetch.rs`, not the budget comparisons in `audit.rs`.

---

## 3. Performance measurement methodology

**What it covers:** Lighthouse methodology, Core Web Vitals measurement windows (LCP, CLS, INP), how synthetic vs. field data differ, what "Slow 4G" means as a throttle setting, what "cold cache" means for measurement reproducibility.

**Authority:** web.dev, Chrome DevTools team explainers, the Lighthouse source. For gnomon's deviations: PLAN.md §4 (anti-theater) and §7.2 (predicted vitals).

**Where confusion appears:** Gnomon's TTI definition (50 ms of main-thread idle *after* hydration completes, not before) differs from Lighthouse's. Predicted vitals are upper bounds, not actual measurements. A passing prediction + `--measure` failure is expected behavior, not a bug.

---

## 4. Rust language and compiler behavior

**What it covers:** Ownership, lifetimes, async/await (tokio), Cargo workspace layout (future), clippy lints, edition 2024 features (let chains, etc.).

**Authority:** The Rust Reference, Clippy docs, Cargo book. For async: tokio docs.

**Where confusion appears:** Edition 2024 let-chain syntax (`if let Some(x) = foo && let Some(y) = bar(x)`) is valid but may surprise contributors from older editions. Clippy's `collapsible_if` lint requires it. When in doubt, `cargo clippy` is authoritative.

---

## 5. CI and GitHub integration

**What it covers:** GitHub Actions syntax, SARIF 2.1.0 format, branch protection rules, the gnomon-action (planned).

**Authority:** GitHub Actions docs, SARIF spec, GitHub code scanning docs.

**Where confusion appears:** No CI workflows exist yet. The snapshot.sh runs `cargo build/test/clippy` locally as a substitute. When the champion says "CI is red," they mean the snapshot.sh output, not a GitHub Actions run.

---

## 6. The insley-web thesis and positioning

**What it covers:** Why gnomon exists, what "enforcing the standard" means culturally, which sites gnomon uses as benchmarks (McMaster-Carr, gov.uk, old Craigslist), what the `insley` preset name refers to.

**Authority:** PLAN.md §1, §17, §18. The README opening paragraph. The manifesto at the root of this project.

**Where confusion appears:** The insley preset and the insley-web project are related but distinct. `insley` is a preset (a set of budget numbers). `insley-web` is the broader thesis project (PLAN.md §17). Gnomon is the tooling arm; insley-web is the reference implementation. Don't conflate them when deciding whether a budget number change is a gnomon decision or an insley-web decision.
