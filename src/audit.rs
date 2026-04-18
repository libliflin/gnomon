use std::collections::HashSet;
use std::time::Instant;

use serde::Serialize;
use url::Url;

use crate::analyze::{AssetKind, HtmlAnalysis, analyze_html, classify};
use crate::budget::Budget;
use crate::fetch::{Fetched, Fetcher};
use crate::forbidden::ForbiddenMatcher;
use crate::violation::{Violation, ViolationKind};

#[derive(Debug, Serialize)]
pub struct AuditReport {
    pub url: String,
    pub gnomon_version: &'static str,
    pub elapsed_ms: u128,
    pub html: Fetched,
    pub html_analysis: HtmlAnalysis,
    pub resources: Vec<Fetched>,
    pub totals: Totals,
    pub violations: Vec<Violation>,
}

#[derive(Debug, Default, Serialize)]
pub struct Totals {
    pub requests: u32,
    pub third_party_domains: u32,
    pub third_party_list: Vec<String>,

    pub html_brotli: u64,
    pub css_bytes: u64,
    pub js_bytes: u64,
    pub image_bytes: u64,
    pub font_bytes: u64,
    pub total_bytes: u64,

    pub fonts_count: u32,
    pub render_blocking: u32,
}

pub async fn audit_url(url_str: &str, budget: &Budget) -> anyhow::Result<AuditReport> {
    let started = Instant::now();

    let url = Url::parse(url_str)
        .map_err(|e| anyhow::anyhow!("--url {url_str:?} is not a valid URL: {e}"))?;
    let fetcher = Fetcher::new()?;

    let root = fetcher.fetch_root(&url).await?;

    if root.status >= 400 {
        anyhow::bail!("root fetch of {url_str} returned HTTP {}", root.status);
    }

    let html_text = root
        .body
        .as_deref()
        .map(|b| String::from_utf8_lossy(b).into_owned())
        .unwrap_or_default();

    let analysis = analyze_html(&html_text, &url);

    // Every sub-resource referenced in the HTML, deduplicated.
    let mut to_fetch: Vec<Url> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    let push_resource = |u: &str, to_fetch: &mut Vec<Url>, seen: &mut HashSet<String>| {
        if seen.insert(u.to_string())
            && let Ok(parsed) = Url::parse(u)
        {
            to_fetch.push(parsed);
        }
    };

    for r in &analysis.script_urls {
        push_resource(&r.url, &mut to_fetch, &mut seen);
    }
    for r in &analysis.stylesheet_urls {
        push_resource(&r.url, &mut to_fetch, &mut seen);
    }
    for r in &analysis.image_urls {
        push_resource(&r.url, &mut to_fetch, &mut seen);
    }
    for r in &analysis.font_urls {
        push_resource(&r.url, &mut to_fetch, &mut seen);
    }
    for r in &analysis.other_urls {
        push_resource(&r.url, &mut to_fetch, &mut seen);
    }

    let fetched = fetcher.fetch_many(to_fetch, false).await;

    // Aggregate totals.
    let mut totals = Totals::default();
    let root_host_root = registrable_domain(url.host_str().unwrap_or(""));

    // Count root HTML.
    totals.requests = 1 + fetched.len() as u32;
    totals.html_brotli = root.brotli_bytes;
    // inline styles / scripts roll into CSS/JS budgets so a site that inlines
    // everything to "hide" bytes from external-resource checks still gets caught
    totals.css_bytes += analysis.inline_style_bytes;
    totals.js_bytes += analysis.inline_script_bytes;

    // Brotli-approximate external resources so the byte totals are apples-to-
    // apples with the HTML number. For binary images/fonts this is close to a
    // no-op (brotli barely compresses them), which is what we want.
    let mut third_party_hosts: HashSet<String> = HashSet::new();

    // Build quick lookup: is a URL a known stylesheet/script/image/font by what
    // the HTML said, so we can classify even when content-type is missing?
    let is_css: HashSet<&str> = analysis.stylesheet_urls.iter().map(|r| r.url.as_str()).collect();
    let is_js: HashSet<&str> = analysis.script_urls.iter().map(|r| r.url.as_str()).collect();
    let is_img: HashSet<&str> = analysis.image_urls.iter().map(|r| r.url.as_str()).collect();
    let is_font: HashSet<&str> = analysis.font_urls.iter().map(|r| r.url.as_str()).collect();

    // Per-category resource lists for violation contributor hints.
    let mut css_resources: Vec<(String, u64)> = Vec::new();
    let mut js_resources: Vec<(String, u64)> = Vec::new();
    let mut image_resources: Vec<(String, u64)> = Vec::new();
    let mut font_resources: Vec<(String, u64)> = Vec::new();

    for f in &fetched {
        let size = f.brotli_bytes;

        // Host for third-party tally.
        if let Ok(u) = Url::parse(&f.url)
            && let Some(host) = u.host_str()
        {
            let reg = registrable_domain(host);
            if !reg.is_empty() && reg != root_host_root {
                third_party_hosts.insert(reg);
            }
        }

        let kind = if is_css.contains(f.url.as_str()) {
            AssetKind::Css
        } else if is_js.contains(f.url.as_str()) {
            AssetKind::Js
        } else if is_img.contains(f.url.as_str()) {
            AssetKind::Image
        } else if is_font.contains(f.url.as_str()) {
            AssetKind::Font
        } else {
            classify(&f.url, f.content_type.as_deref())
        };

        match kind {
            AssetKind::Css => {
                totals.css_bytes += size;
                css_resources.push((f.url.clone(), size));
            }
            AssetKind::Js => {
                totals.js_bytes += size;
                js_resources.push((f.url.clone(), size));
            }
            AssetKind::Image => {
                totals.image_bytes += size;
                image_resources.push((f.url.clone(), size));
            }
            AssetKind::Font => {
                totals.font_bytes += size;
                totals.fonts_count += 1;
                font_resources.push((f.url.clone(), size));
            }
            AssetKind::Html | AssetKind::Other => {
                // Do not attribute to a typed bucket, but still counts toward total.
            }
        }
    }

    // Inline bytes count toward the budget totals (lines 102–103). Surface them
    // as named contributors so the violation detail says "(inline <style>)" rather
    // than staying silent when external files are absent or smaller.
    if analysis.inline_style_bytes > 0 {
        css_resources.push(("(inline <style>)".to_string(), analysis.inline_style_bytes));
    }
    if analysis.inline_script_bytes > 0 {
        js_resources.push(("(inline <script>)".to_string(), analysis.inline_script_bytes));
    }

    // Sort each category descending by size so top contributors are first.
    css_resources.sort_unstable_by_key(|b| std::cmp::Reverse(b.1));
    js_resources.sort_unstable_by_key(|b| std::cmp::Reverse(b.1));
    image_resources.sort_unstable_by_key(|b| std::cmp::Reverse(b.1));
    font_resources.sort_unstable_by_key(|b| std::cmp::Reverse(b.1));

    totals.total_bytes =
        totals.html_brotli + totals.css_bytes + totals.js_bytes + totals.image_bytes + totals.font_bytes;
    totals.third_party_domains = third_party_hosts.len() as u32;
    totals.third_party_list = {
        let mut v: Vec<String> = third_party_hosts.into_iter().collect();
        v.sort();
        v
    };
    totals.render_blocking = analysis.render_blocking_in_head;

    // -------- Violations --------
    let mut violations = Vec::new();

    // Byte budgets.
    bytes_check(&mut violations, "html", totals.html_brotli, budget.preset.bytes.html, &[]);
    bytes_check(&mut violations, "css", totals.css_bytes, budget.preset.bytes.css, &css_resources);
    bytes_check(&mut violations, "js", totals.js_bytes, budget.preset.bytes.js, &js_resources);
    bytes_check(&mut violations, "images", totals.image_bytes, budget.preset.bytes.images, &image_resources);
    bytes_check(&mut violations, "fonts", totals.font_bytes, budget.preset.bytes.fonts, &font_resources);
    bytes_check(&mut violations, "total", totals.total_bytes, budget.preset.bytes.total, &[]);

    // Count budgets.
    {
        let actual = totals.requests;
        let bgt = budget.preset.count.requests;
        if actual > bgt {
            violations.push(Violation {
                kind: ViolationKind::Count,
                metric: "requests",
                budget: bgt as u64,
                actual: actual as u64,
                detail: requests_count_detail(
                    actual - bgt,
                    bgt,
                    actual,
                    &css_resources,
                    &js_resources,
                    &image_resources,
                    &font_resources,
                ),
            });
        }
    }
    {
        let actual = totals.third_party_domains;
        let bgt = budget.preset.count.third_party_domains;
        if actual > bgt {
            violations.push(Violation {
                kind: ViolationKind::Count,
                metric: "third_party_domains",
                budget: bgt as u64,
                actual: actual as u64,
                detail: third_party_domains_detail(actual - bgt, bgt, &totals.third_party_list),
            });
        }
    }
    {
        let actual = totals.render_blocking;
        let bgt = budget.preset.count.render_blocking;
        if actual > bgt {
            let blocking_names: Vec<String> = analysis
                .stylesheet_urls
                .iter()
                .chain(analysis.script_urls.iter())
                .filter(|r| r.render_blocking)
                .map(|r| url_filename(&r.url))
                .collect();
            violations.push(Violation {
                kind: ViolationKind::Count,
                metric: "render_blocking",
                budget: bgt as u64,
                actual: actual as u64,
                detail: render_blocking_detail(actual - bgt, bgt, &blocking_names),
            });
        }
    }
    {
        let actual = totals.fonts_count;
        let bgt = budget.preset.count.fonts;
        if actual > bgt {
            violations.push(Violation {
                kind: ViolationKind::Count,
                metric: "fonts",
                budget: bgt as u64,
                actual: actual as u64,
                detail: fonts_count_detail(actual - bgt, bgt, &font_resources),
            });
        }
    }

    // Forbidden list.
    let forbidden = ForbiddenMatcher::new();
    let mut flagged_urls: HashSet<String> = HashSet::new();
    let check = |vios: &mut Vec<Violation>, u: &str, flagged: &mut HashSet<String>| {
        if let Some((pat, reason)) = forbidden.find(u)
            && flagged.insert(pat.to_string())
        {
            vios.push(Violation {
                kind: ViolationKind::Forbidden,
                metric: "forbidden",
                budget: 0,
                actual: 1,
                detail: format!("{pat} — {reason}"),
            });
        }
    };
    for f in &fetched {
        check(&mut violations, &f.url, &mut flagged_urls);
    }
    // Also check HTML-declared resources that we couldn't fetch but referenced.
    for r in analysis
        .script_urls
        .iter()
        .chain(analysis.stylesheet_urls.iter())
        .chain(analysis.image_urls.iter())
        .chain(analysis.font_urls.iter())
        .chain(analysis.other_urls.iter())
    {
        check(&mut violations, &r.url, &mut flagged_urls);
    }
    for host in &analysis.preconnect_targets {
        check(&mut violations, host, &mut flagged_urls);
    }

    // Anti-theater.
    violations.extend(theater_violations(&analysis));

    // Fetch errors.
    let mut errored_urls: Vec<String> = Vec::new();
    for f in &fetched {
        if f.error.is_some() || (f.status >= 400 && !f.url.is_empty()) {
            errored_urls.push(f.url.clone());
        }
    }
    if !errored_urls.is_empty() {
        violations.push(Violation {
            kind: ViolationKind::FetchError,
            metric: "fetch_error",
            budget: 0,
            actual: errored_urls.len() as u64,
            detail: format!(
                "{} resource(s) failed to fetch (first: {})",
                errored_urls.len(),
                errored_urls[0]
            ),
        });
    }

    Ok(AuditReport {
        url: url_str.to_string(),
        gnomon_version: env!("CARGO_PKG_VERSION"),
        elapsed_ms: started.elapsed().as_millis(),
        html: root,
        html_analysis: analysis,
        resources: fetched,
        totals,
        violations,
    })
}

/// Appends a `ViolationKind::Bytes` entry to `vios` when `actual > budget`.
/// `top` is already sorted descending by size; up to 2 contributors are named.
fn bytes_check(
    vios: &mut Vec<Violation>,
    metric: &'static str,
    actual: u64,
    budget: u64,
    top: &[(String, u64)],
) {
    if actual > budget {
        let mut detail = format!(
            "{} over {} budget",
            humansize::format_size(actual.saturating_sub(budget), humansize::BINARY),
            humansize::format_size(budget, humansize::BINARY)
        );
        let contributors: Vec<String> = top
            .iter()
            .take(2)
            .map(|(url, bytes)| {
                format!(
                    "{} ({})",
                    url_filename(url),
                    humansize::format_size(*bytes, humansize::BINARY)
                )
            })
            .collect();
        if !contributors.is_empty() {
            detail.push_str(" — ");
            detail.push_str(&contributors.join(", "));
        }
        vios.push(Violation {
            kind: ViolationKind::Bytes,
            metric,
            budget,
            actual,
            detail,
        });
    }
}

/// Formats the detail string for a render_blocking count violation.
/// `over` is `actual - budget`; `names` are the blocking resource filenames.
fn render_blocking_detail(over: u32, budget: u32, names: &[String]) -> String {
    let mut s = format!("{over} over budget of {budget}");
    if !names.is_empty() {
        s.push_str(" — ");
        s.push_str(&names.join(", "));
    }
    s
}

/// Formats the detail string for a third_party_domains count violation.
fn third_party_domains_detail(over: u32, budget: u32, domains: &[String]) -> String {
    let mut s = format!("{over} over budget of {budget}");
    if !domains.is_empty() {
        s.push_str(" — ");
        s.push_str(&domains.join(", "));
    }
    s
}

/// Formats the detail string for a fonts count violation.
/// Shows top-2 contributors with their brotli-compressed sizes.
fn fonts_count_detail(over: u32, budget: u32, top: &[(String, u64)]) -> String {
    let mut s = format!("{over} over budget of {budget}");
    let contributors: Vec<String> = top
        .iter()
        .take(2)
        .map(|(url, bytes)| {
            format!(
                "{} ({})",
                url_filename(url),
                humansize::format_size(*bytes, humansize::BINARY)
            )
        })
        .collect();
    if !contributors.is_empty() {
        s.push_str(" — ");
        s.push_str(&contributors.join(", "));
    }
    s
}

/// Formats the detail string for a requests count violation.
/// Shows a per-type breakdown sorted by count descending, omitting zeros.
/// Inline synthetic entries (identified by the `"(inline "` prefix) are excluded
/// from type counts — they contribute to totals but are not discrete requests.
fn requests_count_detail(
    over: u32,
    budget: u32,
    total: u32,
    css: &[(String, u64)],
    js: &[(String, u64)],
    images: &[(String, u64)],
    fonts: &[(String, u64)],
) -> String {
    let css_count = css.iter().filter(|(u, _)| !u.starts_with("(inline ")).count() as u32;
    let js_count = js.iter().filter(|(u, _)| !u.starts_with("(inline ")).count() as u32;
    let img_count = images.len() as u32;
    let font_count = fonts.len() as u32;

    let mut parts: Vec<(&str, u32)> =
        vec![("js", js_count), ("css", css_count), ("img", img_count), ("font", font_count)];
    parts.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));

    let parts: Vec<String> = parts
        .iter()
        .filter(|&&(_, n)| n > 0)
        .map(|&(label, n)| format!("{n} {label}"))
        .collect();

    let mut s = format!("{over} over budget of {budget}");
    if !parts.is_empty() {
        s.push_str(" — ");
        s.push_str(&parts.join(", "));
    }
    s.push_str(&format!(" ({total} total)"));
    s
}

/// Returns the last non-empty path segment of a URL, falling back to the host.
/// Used to produce short contributor labels in byte-budget violation details.
fn url_filename(url: &str) -> String {
    if let Ok(u) = Url::parse(url) {
        let seg = u
            .path_segments()
            .and_then(|mut segs| segs.rfind(|s| !s.is_empty()))
            .unwrap_or("");
        if !seg.is_empty() {
            return seg.to_string();
        }
        if let Some(host) = u.host_str() {
            return host.to_string();
        }
    }
    url.to_string()
}

/// Assembles anti-theater violations from the HTML analysis.
/// Pure function — takes only `&HtmlAnalysis`, makes no network calls.
/// New theater checks belong here: add a branch, add a test, done.
fn theater_violations(analysis: &HtmlAnalysis) -> Vec<Violation> {
    let mut vios = Vec::new();
    if analysis.lazy_lcp_candidate {
        vios.push(Violation {
            kind: ViolationKind::Theater,
            metric: "lazy_lcp",
            budget: 0,
            actual: 1,
            detail: "first <img> has loading=\"lazy\" — likely LCP gaming".into(),
        });
    }
    if !analysis.has_viewport_meta {
        vios.push(Violation {
            kind: ViolationKind::Theater,
            metric: "viewport_meta",
            budget: 0,
            actual: 1,
            detail: "missing <meta name=\"viewport\">".into(),
        });
    }
    if analysis.img_missing_dimensions > 0 {
        vios.push(Violation {
            kind: ViolationKind::Theater,
            metric: "img_dimensions",
            budget: 0,
            actual: analysis.img_missing_dimensions as u64,
            detail: format!(
                "{} <img> element(s) missing explicit width/height — layout shift (CLS)",
                analysis.img_missing_dimensions
            ),
        });
    }
    if !analysis.has_charset_meta {
        vios.push(Violation {
            kind: ViolationKind::Theater,
            metric: "charset_meta",
            budget: 0,
            actual: 1,
            detail: "missing <meta charset> or http-equiv Content-Type — forces encoding sniff"
                .into(),
        });
    }
    vios
}

/// Naive eTLD+1 for third-party comparison. Falls short on `.co.uk` etc., but
/// good enough for v0.0.2 — the error case is "we undercount third parties on
/// multi-suffix TLDs," which means we're generous to the site, not harsh.
fn registrable_domain(host: &str) -> String {
    if host.is_empty() {
        return String::new();
    }
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() <= 2 {
        return host.to_ascii_lowercase();
    }
    let n = parts.len();
    format!("{}.{}", parts[n - 2], parts[n - 1]).to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- url_filename ----

    #[test]
    fn url_filename_returns_last_path_segment() {
        assert_eq!(
            url_filename("https://cdn.example.com/js/landingprod.js"),
            "landingprod.js"
        );
    }

    #[test]
    fn url_filename_strips_query_string_from_segment() {
        // ?v=123 must not leak into the filename label.
        assert_eq!(
            url_filename("https://cdn.example.com/app.js?v=abc"),
            "app.js"
        );
    }

    #[test]
    fn url_filename_falls_back_to_host_on_trailing_slash() {
        assert_eq!(
            url_filename("https://cdn.example.com/"),
            "cdn.example.com"
        );
    }

    #[test]
    fn url_filename_falls_back_to_host_on_no_path() {
        assert_eq!(url_filename("https://cdn.example.com"), "cdn.example.com");
    }

    #[test]
    fn url_filename_returns_bare_filename() {
        assert_eq!(
            url_filename("https://cdn.example.com/main.css"),
            "main.css"
        );
    }

    // ---- registrable_domain ----

    #[test]
    fn registrable_domain_extracts_last_two_labels() {
        assert_eq!(registrable_domain("cdn.example.com"), "example.com");
    }

    #[test]
    fn registrable_domain_returns_host_when_two_labels() {
        assert_eq!(registrable_domain("example.com"), "example.com");
    }

    #[test]
    fn registrable_domain_returns_host_when_single_label() {
        assert_eq!(registrable_domain("localhost"), "localhost");
    }

    #[test]
    fn registrable_domain_returns_empty_for_empty_input() {
        assert_eq!(registrable_domain(""), "");
    }

    #[test]
    fn registrable_domain_lowercases_result() {
        assert_eq!(registrable_domain("CDN.Example.COM"), "example.com");
    }

    // ---- render_blocking_detail ----

    #[test]
    fn render_blocking_detail_no_names() {
        // When blocking resource names are unavailable the detail still shows the count and budget.
        assert_eq!(render_blocking_detail(1, 0, &[]), "1 over budget of 0");
    }

    #[test]
    fn render_blocking_detail_single_name() {
        let names = vec!["landingprod.js".to_string()];
        assert_eq!(
            render_blocking_detail(1, 0, &names),
            "1 over budget of 0 — landingprod.js"
        );
    }

    #[test]
    fn render_blocking_detail_multiple_names() {
        let names = vec!["main.css".to_string(), "analytics.js".to_string()];
        assert_eq!(
            render_blocking_detail(2, 0, &names),
            "2 over budget of 0 — main.css, analytics.js"
        );
    }

    #[test]
    fn render_blocking_detail_over_count_reflects_budget_delta() {
        // Budget=1, actual=3 → over=2.
        let names = vec!["a.css".to_string(), "b.css".to_string(), "c.js".to_string()];
        assert_eq!(
            render_blocking_detail(2, 1, &names),
            "2 over budget of 1 — a.css, b.css, c.js"
        );
    }

    // ---- third_party_domains_detail ----

    #[test]
    fn third_party_domains_detail_no_domains() {
        assert_eq!(third_party_domains_detail(1, 0, &[]), "1 over budget of 0");
    }

    #[test]
    fn third_party_domains_detail_single_domain() {
        let domains = vec!["google-analytics.com".to_string()];
        assert_eq!(
            third_party_domains_detail(1, 0, &domains),
            "1 over budget of 0 — google-analytics.com"
        );
    }

    #[test]
    fn third_party_domains_detail_multiple_domains() {
        let domains = vec![
            "doubleclick.net".to_string(),
            "google-analytics.com".to_string(),
            "googlesyndication.com".to_string(),
        ];
        assert_eq!(
            third_party_domains_detail(2, 1, &domains),
            "2 over budget of 1 — doubleclick.net, google-analytics.com, googlesyndication.com"
        );
    }

    // ---- fonts_count_detail ----

    #[test]
    fn fonts_count_detail_no_fonts() {
        assert_eq!(fonts_count_detail(1, 0, &[]), "1 over budget of 0");
    }

    #[test]
    fn fonts_count_detail_single_font() {
        let fonts = vec![("https://cdn.example.com/serif-regular.woff2".to_string(), 46080u64)];
        assert_eq!(
            fonts_count_detail(1, 0, &fonts),
            "1 over budget of 0 — serif-regular.woff2 (45 KiB)"
        );
    }

    #[test]
    fn fonts_count_detail_exactly_two_fonts() {
        // Exactly 2 fonts — both should appear (take(2) on a 2-element slice).
        let fonts = vec![
            ("https://cdn.example.com/serif-regular.woff2".to_string(), 46080u64),
            ("https://cdn.example.com/sans-regular.woff2".to_string(), 12288u64),
        ];
        assert_eq!(
            fonts_count_detail(1, 0, &fonts),
            "1 over budget of 0 — serif-regular.woff2 (45 KiB), sans-regular.woff2 (12 KiB)"
        );
    }

    #[test]
    fn fonts_count_detail_top_two_shown() {
        let fonts = vec![
            ("https://cdn.example.com/serif-regular.woff2".to_string(), 46080u64),
            ("https://cdn.example.com/sans-regular.woff2".to_string(), 12288u64),
            ("https://cdn.example.com/mono.woff2".to_string(), 4096u64),
        ];
        // Only top 2 contributors shown even when 3 fonts are present.
        assert_eq!(
            fonts_count_detail(1, 0, &fonts),
            "1 over budget of 0 — serif-regular.woff2 (45 KiB), sans-regular.woff2 (12 KiB)"
        );
    }

    // ---- bytes_check boundary conditions and format ----

    #[test]
    fn bytes_check_exactly_at_budget_no_violation() {
        // actual == budget: the check is `actual > budget`, so no violation must fire.
        let mut vios: Vec<Violation> = Vec::new();
        bytes_check(&mut vios, "css", 14_336, 14_336, &[]);
        assert!(vios.is_empty(), "exactly at budget must not fire, got: {vios:?}");
    }

    #[test]
    fn bytes_check_one_byte_over_fires() {
        // actual = budget + 1: the boundary that triggers the violation.
        let mut vios: Vec<Violation> = Vec::new();
        bytes_check(&mut vios, "css", 14_337, 14_336, &[]);
        assert_eq!(vios.len(), 1, "one byte over must fire exactly one violation");
        assert_eq!(vios[0].metric, "css");
    }

    #[test]
    fn bytes_check_nonzero_budget_detail_format() {
        // Budget 10 KiB (10240 bytes), actual 512000 bytes (~500 KiB).
        // Detail must read "X over Y KiB budget" — pins the goal's example format
        // so the budget owner can paste the violation and have it stand alone.
        let mut vios: Vec<Violation> = Vec::new();
        bytes_check(&mut vios, "css", 512_000, 10_240, &[]);
        assert_eq!(vios.len(), 1);
        let detail = &vios[0].detail;
        assert!(
            detail.contains("10 KiB budget"),
            "detail must include human-readable budget, got: {detail}"
        );
        assert!(
            detail.contains("over"),
            "detail must include 'over', got: {detail}"
        );
    }

    #[test]
    fn bytes_check_zero_budget_nonzero_actual_fires() {
        // Zero budget (insley preset: js = 0) with any nonzero actual must fire.
        // Pins that the `actual > budget` path fires when budget == 0 and actual == 1,
        // and that the detail string is legible ("X over 0 B budget").
        let mut vios: Vec<Violation> = Vec::new();
        bytes_check(&mut vios, "js", 460_800, 0, &[]);
        assert_eq!(vios.len(), 1, "zero budget with nonzero actual must fire");
        let detail = &vios[0].detail;
        assert!(
            detail.contains("budget"),
            "detail must include 'budget', got: {detail}"
        );
        assert!(
            detail.contains("0 B budget"),
            "zero budget must render as '0 B budget', got: {detail}"
        );
    }

    // ---- bytes_check with inline synthetic contributors ----

    #[test]
    fn css_inline_only_shows_inline_style_contributor() {
        // No external CSS files; all bytes are inline. The synthetic "(inline <style>)"
        // entry must appear as the contributor.
        let mut vios: Vec<Violation> = Vec::new();
        // 1.67 MiB inline, 0 budget → over = 1.67 MiB
        let inline_bytes: u64 = 1_755_109;
        let resources = vec![("(inline <style>)".to_string(), inline_bytes)];
        bytes_check(&mut vios, "css", inline_bytes, 0, &resources);
        assert_eq!(vios.len(), 1);
        let detail = &vios[0].detail;
        assert!(
            detail.contains("(inline <style>)"),
            "detail should name inline contributor, got: {detail}"
        );
    }

    #[test]
    fn js_inline_only_shows_inline_script_contributor() {
        let mut vios: Vec<Violation> = Vec::new();
        let inline_bytes: u64 = 460_800; // ~450 KiB
        let resources = vec![("(inline <script>)".to_string(), inline_bytes)];
        bytes_check(&mut vios, "js", inline_bytes, 0, &resources);
        assert_eq!(vios.len(), 1);
        let detail = &vios[0].detail;
        assert!(
            detail.contains("(inline <script>)"),
            "detail should name inline contributor, got: {detail}"
        );
    }

    #[test]
    fn css_mixed_inline_large_enough_to_rank_in_top_two() {
        // External file: 10 KiB. Inline: 50 KiB. Budget: 0.
        // Inline is larger → must appear first (or at least within top-2).
        let mut vios: Vec<Violation> = Vec::new();
        let mut resources = vec![
            ("https://cdn.example.com/small.css".to_string(), 10_240u64),
            ("(inline <style>)".to_string(), 51_200u64),
        ];
        resources.sort_unstable_by_key(|b| std::cmp::Reverse(b.1));
        let total = 10_240 + 51_200;
        bytes_check(&mut vios, "css", total, 0, &resources);
        assert_eq!(vios.len(), 1);
        let detail = &vios[0].detail;
        assert!(
            detail.contains("(inline <style>)"),
            "inline contributor should appear in top-2, got: {detail}"
        );
    }

    #[test]
    fn css_mixed_inline_too_small_for_top_two() {
        // Three resources: two large external files plus a tiny inline block.
        // Inline is smaller than both external files → must not appear in top-2.
        let mut vios: Vec<Violation> = Vec::new();
        let mut resources = vec![
            ("https://cdn.example.com/main.css".to_string(), 50_000u64),
            ("https://cdn.example.com/vendor.css".to_string(), 40_000u64),
            ("(inline <style>)".to_string(), 100u64),
        ];
        resources.sort_unstable_by_key(|b| std::cmp::Reverse(b.1));
        let total = 50_000 + 40_000 + 100;
        bytes_check(&mut vios, "css", total, 0, &resources);
        assert_eq!(vios.len(), 1);
        let detail = &vios[0].detail;
        // top-2 are main.css and vendor.css; inline is third and must be absent
        assert!(
            !detail.contains("(inline <style>)"),
            "tiny inline contributor must not appear in top-2, got: {detail}"
        );
        assert!(detail.contains("main.css"), "main.css must be in top-2, got: {detail}");
        assert!(detail.contains("vendor.css"), "vendor.css must be in top-2, got: {detail}");
    }

    #[test]
    fn js_mixed_inline_large_enough_to_rank_in_top_two() {
        // External file: 10 KiB. Inline: 50 KiB. Budget: 0.
        // Inline is larger → must appear in the top-2 contributor list.
        let mut vios: Vec<Violation> = Vec::new();
        let mut resources = vec![
            ("https://cdn.example.com/small.js".to_string(), 10_240u64),
            ("(inline <script>)".to_string(), 51_200u64),
        ];
        resources.sort_unstable_by_key(|b| std::cmp::Reverse(b.1));
        let total = 10_240 + 51_200;
        bytes_check(&mut vios, "js", total, 0, &resources);
        assert_eq!(vios.len(), 1);
        let detail = &vios[0].detail;
        assert!(
            detail.contains("(inline <script>)"),
            "inline contributor should appear in top-2, got: {detail}"
        );
    }

    #[test]
    fn js_mixed_inline_too_small_for_top_two() {
        // Three resources: two large external JS files plus a tiny inline block.
        // Inline is smaller than both external files → must not appear in top-2.
        let mut vios: Vec<Violation> = Vec::new();
        let mut resources = vec![
            ("https://cdn.example.com/app.js".to_string(), 50_000u64),
            ("https://cdn.example.com/vendor.js".to_string(), 40_000u64),
            ("(inline <script>)".to_string(), 100u64),
        ];
        resources.sort_unstable_by_key(|b| std::cmp::Reverse(b.1));
        let total = 50_000 + 40_000 + 100;
        bytes_check(&mut vios, "js", total, 0, &resources);
        assert_eq!(vios.len(), 1);
        let detail = &vios[0].detail;
        assert!(
            !detail.contains("(inline <script>)"),
            "tiny inline contributor must not appear in top-2, got: {detail}"
        );
        assert!(detail.contains("app.js"), "app.js must be in top-2, got: {detail}");
        assert!(detail.contains("vendor.js"), "vendor.js must be in top-2, got: {detail}");
    }

    #[test]
    fn third_party_domains_detail_over_count_differs_from_domain_count() {
        // budget=1, actual=3 → over=2, but all 3 domains are listed.
        // The "over" prefix reflects the budget delta; the domain list is exhaustive.
        let domains = vec![
            "google-analytics.com".to_string(),
            "googletagservices.com".to_string(),
            "googlesyndication.com".to_string(),
        ];
        assert_eq!(
            third_party_domains_detail(2, 1, &domains),
            "2 over budget of 1 — google-analytics.com, googletagservices.com, googlesyndication.com"
        );
    }

    // ---- requests_count_detail ----

    fn make_res(url: &str, bytes: u64) -> (String, u64) {
        (url.to_string(), bytes)
    }

    #[test]
    fn requests_count_detail_all_types() {
        // 4 js, 2 css, 2 img, 1 font — sorted by count desc, total in parens.
        let css = vec![make_res("a.css", 100), make_res("b.css", 200)];
        let js = vec![
            make_res("a.js", 100),
            make_res("b.js", 200),
            make_res("c.js", 300),
            make_res("d.js", 400),
        ];
        let images = vec![make_res("a.png", 100), make_res("b.png", 200)];
        let fonts = vec![make_res("a.woff2", 100)];
        // total = 1 html + 4 js + 2 css + 2 img + 1 font = 10; budget = 6
        let detail = requests_count_detail(4, 6, 10, &css, &js, &images, &fonts);
        assert_eq!(detail, "4 over budget of 6 — 4 js, 2 css, 2 img, 1 font (10 total)");
    }

    #[test]
    fn requests_count_detail_zero_omission() {
        // Only JS resources; no CSS, images, or fonts. Zeros must not appear.
        let js = vec![make_res("a.js", 100), make_res("b.js", 200)];
        let detail = requests_count_detail(1, 2, 3, &[], &js, &[], &[]);
        assert_eq!(detail, "1 over budget of 2 — 2 js (3 total)");
    }

    #[test]
    fn requests_count_detail_single_type() {
        // Only CSS; one resource over budget.
        let css = vec![make_res("main.css", 50_000)];
        let detail = requests_count_detail(1, 1, 2, &css, &[], &[], &[]);
        assert_eq!(detail, "1 over budget of 1 — 1 css (2 total)");
    }

    #[test]
    fn requests_count_detail_inline_exclusion() {
        // JS resource list contains a synthetic inline entry — it must not count
        // toward the js type tally (it's not a discrete network request).
        let js = vec![
            make_res("https://cdn.example.com/app.js", 40_000),
            make_res("(inline <script>)", 20_000),
        ];
        // actual = 1 html + 1 external js = 2; inline is not a request
        let detail = requests_count_detail(1, 1, 2, &[], &js, &[], &[]);
        assert_eq!(detail, "1 over budget of 1 — 1 js (2 total)");
    }

    #[test]
    fn requests_count_detail_no_typed_resources() {
        // All requests are HTML/other (unclassified). No named types, but total still shown.
        let detail = requests_count_detail(2, 3, 5, &[], &[], &[], &[]);
        assert_eq!(detail, "2 over budget of 3 (5 total)");
    }

    #[test]
    fn requests_count_detail_inline_exclusion_css() {
        // CSS resource list contains a synthetic inline entry — must not count as a request.
        let css = vec![
            make_res("https://cdn.example.com/main.css", 30_000),
            make_res("(inline <style>)", 15_000),
        ];
        let detail = requests_count_detail(1, 1, 2, &css, &[], &[], &[]);
        assert_eq!(detail, "1 over budget of 1 — 1 css (2 total)");
    }

    #[test]
    fn requests_count_detail_inline_only_css() {
        // CSS list contains only a synthetic inline entry (no external CSS file).
        // Inline entries must not count as requests, so css must not appear in the breakdown.
        let css = vec![make_res("(inline <style>)", 50_000)];
        let detail = requests_count_detail(1, 1, 2, &css, &[], &[], &[]);
        assert_eq!(detail, "1 over budget of 1 (2 total)");
    }

    #[test]
    fn requests_count_detail_inline_only_js() {
        // JS list contains only a synthetic inline entry (no external JS file).
        let js = vec![make_res("(inline <script>)", 50_000)];
        let detail = requests_count_detail(1, 1, 2, &[], &js, &[], &[]);
        assert_eq!(detail, "1 over budget of 1 (2 total)");
    }

    #[test]
    fn requests_count_detail_tie_breaks_alphabetically() {
        // css and img both have 2 resources — secondary sort is alphabetical by label.
        // "css" < "img" alphabetically, so css must appear first on a tie.
        let css = vec![make_res("a.css", 100), make_res("b.css", 200)];
        let images = vec![make_res("a.png", 100), make_res("b.png", 200)];
        let detail = requests_count_detail(3, 2, 5, &css, &[], &images, &[]);
        assert_eq!(detail, "3 over budget of 2 — 2 css, 2 img (5 total)");
    }

    #[test]
    fn requests_count_detail_zero_budget() {
        // Insley preset: requests = 0 (no budget). Any nonzero actual produces
        // "N over budget of 0 — ..." — pins the zero-budget rendering path that
        // the other count detail functions all test but this one did not.
        let js = vec![make_res("https://cdn.example.com/app.js", 40_000)];
        let detail = requests_count_detail(2, 0, 2, &[], &js, &[], &[]);
        assert_eq!(detail, "2 over budget of 0 — 1 js (2 total)");
    }

    // ---- theater_violations ----

    #[test]
    fn theater_violations_lazy_lcp_fires() {
        let analysis = HtmlAnalysis {
            lazy_lcp_candidate: true,
            has_viewport_meta: true, // clean — only lazy_lcp should fire
            has_charset_meta: true,
            ..Default::default()
        };
        let vios = theater_violations(&analysis);
        assert_eq!(vios.len(), 1);
        assert_eq!(vios[0].metric, "lazy_lcp");
        assert_eq!(vios[0].kind, ViolationKind::Theater);
        assert!(
            vios[0].detail.contains("LCP gaming"),
            "detail should name LCP gaming, got: {}",
            vios[0].detail
        );
    }

    #[test]
    fn theater_violations_missing_viewport_fires() {
        let analysis = HtmlAnalysis {
            lazy_lcp_candidate: false,
            has_viewport_meta: false, // missing viewport — should fire
            has_charset_meta: true,
            ..Default::default()
        };
        let vios = theater_violations(&analysis);
        assert_eq!(vios.len(), 1);
        assert_eq!(vios[0].metric, "viewport_meta");
        assert_eq!(vios[0].kind, ViolationKind::Theater);
        assert!(
            vios[0].detail.contains("viewport"),
            "detail should mention viewport, got: {}",
            vios[0].detail
        );
    }

    #[test]
    fn theater_violations_clean_page_returns_empty() {
        let analysis = HtmlAnalysis {
            lazy_lcp_candidate: false,
            has_viewport_meta: true,
            has_charset_meta: true,
            ..Default::default()
        };
        let vios = theater_violations(&analysis);
        assert!(vios.is_empty(), "clean page must produce no theater violations");
    }

    #[test]
    fn theater_violations_both_flags_emit_two_violations() {
        let analysis = HtmlAnalysis {
            lazy_lcp_candidate: true,
            has_viewport_meta: false,
            has_charset_meta: true,
            ..Default::default()
        };
        let vios = theater_violations(&analysis);
        assert_eq!(vios.len(), 2);
        let metrics: Vec<&str> = vios.iter().map(|v| v.metric).collect();
        assert!(metrics.contains(&"lazy_lcp"), "lazy_lcp must be present");
        assert!(metrics.contains(&"viewport_meta"), "viewport_meta must be present");
    }

    #[test]
    fn theater_violations_img_missing_dimensions_fires() {
        let analysis = HtmlAnalysis {
            img_missing_dimensions: 3,
            has_viewport_meta: true,
            has_charset_meta: true,
            ..Default::default()
        };
        let vios = theater_violations(&analysis);
        assert_eq!(vios.len(), 1);
        assert_eq!(vios[0].metric, "img_dimensions");
        assert_eq!(vios[0].kind, ViolationKind::Theater);
        assert_eq!(vios[0].actual, 3);
        assert!(
            vios[0].detail.contains("layout shift"),
            "detail must mention layout shift: {}",
            vios[0].detail
        );
    }

    #[test]
    fn theater_violations_img_dimensions_zero_no_violation() {
        let analysis = HtmlAnalysis {
            img_missing_dimensions: 0,
            has_viewport_meta: true,
            has_charset_meta: true,
            ..Default::default()
        };
        let vios = theater_violations(&analysis);
        let img_vios: Vec<_> = vios.iter().filter(|v| v.metric == "img_dimensions").collect();
        assert!(img_vios.is_empty(), "zero missing dimensions must produce no violation");
    }

    #[test]
    fn theater_violations_img_dimensions_single_element() {
        let analysis = HtmlAnalysis {
            img_missing_dimensions: 1,
            has_viewport_meta: true,
            has_charset_meta: true,
            ..Default::default()
        };
        let vios = theater_violations(&analysis);
        assert_eq!(vios.len(), 1);
        assert_eq!(vios[0].metric, "img_dimensions");
        assert_eq!(vios[0].actual, 1);
    }

    #[test]
    fn theater_violations_missing_charset_meta_fires() {
        let analysis = HtmlAnalysis {
            has_charset_meta: false,
            has_viewport_meta: true,
            ..Default::default()
        };
        let vios = theater_violations(&analysis);
        let charset_vios: Vec<_> = vios.iter().filter(|v| v.metric == "charset_meta").collect();
        assert_eq!(charset_vios.len(), 1);
        assert_eq!(charset_vios[0].kind, ViolationKind::Theater);
        assert_eq!(charset_vios[0].actual, 1);
        assert!(
            charset_vios[0].detail.contains("encoding sniff"),
            "detail must mention encoding sniff: {}",
            charset_vios[0].detail
        );
    }

    #[test]
    fn theater_violations_present_charset_meta_no_violation() {
        let analysis = HtmlAnalysis {
            has_charset_meta: true,
            has_viewport_meta: true,
            ..Default::default()
        };
        let vios = theater_violations(&analysis);
        let charset_vios: Vec<_> = vios.iter().filter(|v| v.metric == "charset_meta").collect();
        assert!(
            charset_vios.is_empty(),
            "present charset meta must produce no violation"
        );
    }
}
