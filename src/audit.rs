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
    fn bytes_check(
        vios: &mut Vec<Violation>,
        metric: &'static str,
        actual: u64,
        budget: u64,
        top: &[(String, u64)],
    ) {
        if actual > budget {
            let mut detail = format!(
                "{} over",
                humansize::format_size(actual.saturating_sub(budget), humansize::BINARY)
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
    bytes_check(&mut violations, "html", totals.html_brotli, budget.preset.bytes.html, &[]);
    bytes_check(&mut violations, "css", totals.css_bytes, budget.preset.bytes.css, &css_resources);
    bytes_check(&mut violations, "js", totals.js_bytes, budget.preset.bytes.js, &js_resources);
    bytes_check(&mut violations, "images", totals.image_bytes, budget.preset.bytes.images, &image_resources);
    bytes_check(&mut violations, "fonts", totals.font_bytes, budget.preset.bytes.fonts, &font_resources);
    bytes_check(&mut violations, "total", totals.total_bytes, budget.preset.bytes.total, &[]);

    // Count budgets.
    fn count_check(vios: &mut Vec<Violation>, metric: &'static str, actual: u32, budget: u32) {
        if actual > budget {
            vios.push(Violation {
                kind: ViolationKind::Count,
                metric,
                budget: budget as u64,
                actual: actual as u64,
                detail: format!("{} over", actual - budget),
            });
        }
    }
    count_check(&mut violations, "requests", totals.requests, budget.preset.count.requests);
    {
        let actual = totals.third_party_domains;
        let bgt = budget.preset.count.third_party_domains;
        if actual > bgt {
            violations.push(Violation {
                kind: ViolationKind::Count,
                metric: "third_party_domains",
                budget: bgt as u64,
                actual: actual as u64,
                detail: third_party_domains_detail(actual - bgt, &totals.third_party_list),
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
                detail: render_blocking_detail(actual - bgt, &blocking_names),
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
                detail: fonts_count_detail(actual - bgt, &font_resources),
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
    if analysis.lazy_lcp_candidate {
        violations.push(Violation {
            kind: ViolationKind::Theater,
            metric: "lazy_lcp",
            budget: 0,
            actual: 1,
            detail: "first <img> has loading=\"lazy\" — likely LCP gaming".into(),
        });
    }
    if !analysis.has_viewport_meta {
        violations.push(Violation {
            kind: ViolationKind::Theater,
            metric: "viewport_meta",
            budget: 0,
            actual: 1,
            detail: "missing <meta name=\"viewport\">".into(),
        });
    }

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

/// Formats the detail string for a render_blocking count violation.
/// `over` is `actual - budget`; `names` are the blocking resource filenames.
fn render_blocking_detail(over: u32, names: &[String]) -> String {
    let mut s = format!("{over} over");
    if !names.is_empty() {
        s.push_str(" — ");
        s.push_str(&names.join(", "));
    }
    s
}

/// Formats the detail string for a third_party_domains count violation.
fn third_party_domains_detail(over: u32, domains: &[String]) -> String {
    let mut s = format!("{over} over");
    if !domains.is_empty() {
        s.push_str(" — ");
        s.push_str(&domains.join(", "));
    }
    s
}

/// Formats the detail string for a fonts count violation.
/// Shows top-2 contributors with their brotli-compressed sizes.
fn fonts_count_detail(over: u32, top: &[(String, u64)]) -> String {
    let mut s = format!("{over} over");
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
        // When blocking resource names are unavailable the detail still shows the count.
        assert_eq!(render_blocking_detail(1, &[]), "1 over");
    }

    #[test]
    fn render_blocking_detail_single_name() {
        let names = vec!["landingprod.js".to_string()];
        assert_eq!(
            render_blocking_detail(1, &names),
            "1 over — landingprod.js"
        );
    }

    #[test]
    fn render_blocking_detail_multiple_names() {
        let names = vec!["main.css".to_string(), "analytics.js".to_string()];
        assert_eq!(
            render_blocking_detail(2, &names),
            "2 over — main.css, analytics.js"
        );
    }

    #[test]
    fn render_blocking_detail_over_count_reflects_budget_delta() {
        // Budget=1, actual=3 → over=2.
        let names = vec!["a.css".to_string(), "b.css".to_string(), "c.js".to_string()];
        assert_eq!(
            render_blocking_detail(2, &names),
            "2 over — a.css, b.css, c.js"
        );
    }

    // ---- third_party_domains_detail ----

    #[test]
    fn third_party_domains_detail_no_domains() {
        assert_eq!(third_party_domains_detail(1, &[]), "1 over");
    }

    #[test]
    fn third_party_domains_detail_single_domain() {
        let domains = vec!["google-analytics.com".to_string()];
        assert_eq!(
            third_party_domains_detail(1, &domains),
            "1 over — google-analytics.com"
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
            third_party_domains_detail(2, &domains),
            "2 over — doubleclick.net, google-analytics.com, googlesyndication.com"
        );
    }

    // ---- fonts_count_detail ----

    #[test]
    fn fonts_count_detail_no_fonts() {
        assert_eq!(fonts_count_detail(1, &[]), "1 over");
    }

    #[test]
    fn fonts_count_detail_single_font() {
        let fonts = vec![("https://cdn.example.com/serif-regular.woff2".to_string(), 46080u64)];
        assert_eq!(
            fonts_count_detail(1, &fonts),
            "1 over — serif-regular.woff2 (45 KiB)"
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
            fonts_count_detail(1, &fonts),
            "1 over — serif-regular.woff2 (45 KiB), sans-regular.woff2 (12 KiB)"
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
            fonts_count_detail(1, &fonts),
            "1 over — serif-regular.woff2 (45 KiB), sans-regular.woff2 (12 KiB)"
        );
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
            third_party_domains_detail(2, &domains),
            "2 over — google-analytics.com, googletagservices.com, googlesyndication.com"
        );
    }
}
