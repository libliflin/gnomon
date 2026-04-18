use scraper::{Html, Selector};
use serde::Serialize;
use url::Url;

/// What we pull out of the HTML for the budget comparison.
#[derive(Debug, Default, Clone, Serialize)]
pub struct HtmlAnalysis {
    pub inline_script_bytes: u64,
    pub inline_style_bytes: u64,
    pub script_urls: Vec<ResourceRef>,
    pub stylesheet_urls: Vec<ResourceRef>,
    pub image_urls: Vec<ResourceRef>,
    pub font_urls: Vec<ResourceRef>,
    pub other_urls: Vec<ResourceRef>, // iframes, video posters, etc.

    pub render_blocking_in_head: u32,
    pub img_missing_dimensions: u32,
    pub lazy_lcp_candidate: bool,
    pub has_viewport_meta: bool,
    pub has_charset_meta: bool,

    pub preload_hint_count: u32,
    pub preconnect_targets: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourceRef {
    pub url: String,
    /// Whether the element is in <head> without async/defer — i.e. render-blocking.
    pub render_blocking: bool,
    /// Only set for <img> and <link rel=preload as=image>.
    pub is_lcp_candidate: bool,
    /// Only set for <img loading="lazy">.
    pub is_lazy: bool,
}

pub fn analyze_html(html: &str, base: &Url) -> HtmlAnalysis {
    let doc = Html::parse_document(html);

    let mut a = HtmlAnalysis::default();

    let sel_script = Selector::parse("script").unwrap();
    let sel_link = Selector::parse("link").unwrap();
    let sel_style = Selector::parse("style").unwrap();
    let sel_img = Selector::parse("img").unwrap();
    let sel_iframe = Selector::parse("iframe").unwrap();
    let sel_meta = Selector::parse("meta").unwrap();

    // Inline <style>.
    for el in doc.select(&sel_style) {
        let text: String = el.text().collect();
        a.inline_style_bytes += text.len() as u64;
    }

    // Scripts. Inline counted inline; external added to script_urls.
    // Render-blocking = inside <head> (or no parent check needed for
    // simplicity: we treat any script without async/defer that appears before
    // </head> as blocking; scraper gives us the whole tree so we approximate
    // by looking for an ancestor <head>.)
    for el in doc.select(&sel_script) {
        let async_ = el.value().attr("async").is_some();
        let defer = el.value().attr("defer").is_some();
        let in_head = has_ancestor(&el, "head");
        let blocking = in_head && !async_ && !defer;
        if let Some(src) = el.value().attr("src") {
            if let Some(resolved) = resolve(base, src) {
                a.script_urls.push(ResourceRef {
                    url: resolved,
                    render_blocking: blocking,
                    is_lcp_candidate: false,
                    is_lazy: false,
                });
                if blocking {
                    a.render_blocking_in_head += 1;
                }
            }
        } else {
            let body: String = el.text().collect();
            a.inline_script_bytes += body.len() as u64;
        }
    }

    // <link rel="...">
    for el in doc.select(&sel_link) {
        let rel = el.value().attr("rel").unwrap_or("").to_ascii_lowercase();
        let as_attr = el.value().attr("as").unwrap_or("").to_ascii_lowercase();
        let media = el.value().attr("media").unwrap_or("").to_ascii_lowercase();
        let disabled = el.value().attr("disabled").is_some();
        let href = match el.value().attr("href") {
            Some(h) => h,
            None => continue,
        };
        let resolved = match resolve(base, href) {
            Some(r) => r,
            None => continue,
        };
        let in_head = has_ancestor(&el, "head");

        if rel.contains("stylesheet") {
            // Print-only and disabled stylesheets are not render-blocking.
            let blocking = in_head && !disabled && !(media == "print" || media.contains("print"));
            a.stylesheet_urls.push(ResourceRef {
                url: resolved,
                render_blocking: blocking,
                is_lcp_candidate: false,
                is_lazy: false,
            });
            if blocking {
                a.render_blocking_in_head += 1;
            }
        } else if rel.contains("preload") {
            a.preload_hint_count += 1;
            match as_attr.as_str() {
                "font" => a.font_urls.push(ResourceRef {
                    url: resolved,
                    render_blocking: false,
                    is_lcp_candidate: false,
                    is_lazy: false,
                }),
                "image" => a.image_urls.push(ResourceRef {
                    url: resolved,
                    render_blocking: false,
                    is_lcp_candidate: true,
                    is_lazy: false,
                }),
                "script" => a.script_urls.push(ResourceRef {
                    url: resolved,
                    render_blocking: false,
                    is_lcp_candidate: false,
                    is_lazy: false,
                }),
                "style" => {
                    // Anti-theater: preload-as-style that is swapped on load is a
                    // render-blocking stylesheet in disguise. We flag both forms.
                    a.stylesheet_urls.push(ResourceRef {
                        url: resolved,
                        render_blocking: true,
                        is_lcp_candidate: false,
                        is_lazy: false,
                    });
                    a.render_blocking_in_head += 1;
                }
                _ => a.other_urls.push(ResourceRef {
                    url: resolved,
                    render_blocking: false,
                    is_lcp_candidate: false,
                    is_lazy: false,
                }),
            }
        } else if rel.contains("preconnect") || rel.contains("dns-prefetch") {
            if let Ok(u) = Url::parse(&resolved)
                && let Some(h) = u.host_str()
            {
                a.preconnect_targets.push(h.to_string());
            }
        } else if rel.contains("modulepreload") {
            a.script_urls.push(ResourceRef {
                url: resolved,
                render_blocking: false,
                is_lcp_candidate: false,
                is_lazy: false,
            });
        } else if rel.contains("icon") || rel.contains("manifest") {
            a.other_urls.push(ResourceRef {
                url: resolved,
                render_blocking: false,
                is_lcp_candidate: false,
                is_lazy: false,
            });
        }
    }

    // <img>
    let mut first_img = true;
    for el in doc.select(&sel_img) {
        let src = el.value().attr("src").or_else(|| el.value().attr("data-src"));
        let has_w = el.value().attr("width").is_some();
        let has_h = el.value().attr("height").is_some();
        let lazy = el
            .value()
            .attr("loading")
            .map(|v| v.eq_ignore_ascii_case("lazy"))
            .unwrap_or(false);
        if !has_w || !has_h {
            a.img_missing_dimensions += 1;
        }
        if first_img && lazy {
            // Anti-theater: the first <img> is a plausible LCP candidate; if it
            // is lazy-loaded this is often a Lighthouse-gaming trick.
            a.lazy_lcp_candidate = true;
        }
        if let Some(s) = src
            && let Some(resolved) = resolve(base, s)
        {
            a.image_urls.push(ResourceRef {
                url: resolved,
                render_blocking: false,
                is_lcp_candidate: first_img,
                is_lazy: lazy,
            });
        }
        first_img = false;
    }

    // <iframe>
    for el in doc.select(&sel_iframe) {
        if let Some(src) = el.value().attr("src")
            && let Some(resolved) = resolve(base, src)
        {
            a.other_urls.push(ResourceRef {
                url: resolved,
                render_blocking: false,
                is_lcp_candidate: false,
                is_lazy: false,
            });
        }
    }

    // <meta>
    for el in doc.select(&sel_meta) {
        let name = el.value().attr("name").unwrap_or("").to_ascii_lowercase();
        let charset = el.value().attr("charset");
        if charset.is_some() {
            a.has_charset_meta = true;
        }
        if name == "viewport" {
            a.has_viewport_meta = true;
        }
        if el
            .value()
            .attr("http-equiv")
            .map(|v| v.eq_ignore_ascii_case("content-type"))
            .unwrap_or(false)
        {
            a.has_charset_meta = true;
        }
    }

    a
}

fn resolve(base: &Url, href: &str) -> Option<String> {
    let trimmed = href.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with("data:")
        || trimmed.starts_with("javascript:")
        || trimmed.starts_with("mailto:")
        || trimmed.starts_with("tel:")
        || trimmed.starts_with('#')
    {
        return None;
    }
    base.join(trimmed).ok().map(|u| u.to_string())
}

fn has_ancestor(el: &scraper::ElementRef<'_>, tag: &str) -> bool {
    let mut cur = el.parent();
    while let Some(n) = cur {
        if let Some(v) = n.value().as_element()
            && v.name() == tag
        {
            return true;
        }
        cur = n.parent();
    }
    false
}

/// Classify a URL into a broad resource category using extension + content-type
/// hints. Not perfect, but cheap and predictable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AssetKind {
    Html,
    Css,
    Js,
    Image,
    Font,
    Other,
}

pub fn classify(url: &str, content_type: Option<&str>) -> AssetKind {
    if let Some(ct) = content_type {
        let ct = ct.to_ascii_lowercase();
        if ct.starts_with("text/html") {
            return AssetKind::Html;
        }
        if ct.starts_with("text/css") {
            return AssetKind::Css;
        }
        if ct.contains("javascript") || ct.contains("ecmascript") || ct.contains("/json") {
            return AssetKind::Js;
        }
        if ct.starts_with("image/") {
            return AssetKind::Image;
        }
        if ct.starts_with("font/") || ct.contains("woff") || ct.contains("opentype") || ct.contains("truetype") {
            return AssetKind::Font;
        }
    }
    let u = url.split(['?', '#']).next().unwrap_or(url);
    let ext = u.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "html" | "htm" => AssetKind::Html,
        "css" => AssetKind::Css,
        "js" | "mjs" | "cjs" | "json" => AssetKind::Js,
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "avif" | "svg" | "ico" | "bmp" => AssetKind::Image,
        "woff" | "woff2" | "ttf" | "otf" | "eot" => AssetKind::Font,
        _ => AssetKind::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn analyze(html: &str) -> HtmlAnalysis {
        let base = url::Url::parse("https://example.com/").unwrap();
        analyze_html(html, &base)
    }

    // ── lazy_lcp_candidate ────────────────────────────────────────────────────

    #[test]
    fn lazy_lcp_candidate_fires_on_first_lazy_img() {
        let html = r#"<!doctype html><html><body>
            <img src="/hero.jpg" loading="lazy" width="800" height="400">
        </body></html>"#;
        let a = analyze(html);
        assert!(a.lazy_lcp_candidate, "first lazy <img> must set lazy_lcp_candidate");
    }

    #[test]
    fn lazy_lcp_candidate_clear_when_first_img_not_lazy() {
        let html = r#"<!doctype html><html><body>
            <img src="/hero.jpg" width="800" height="400">
            <img src="/thumb.jpg" loading="lazy" width="200" height="100">
        </body></html>"#;
        let a = analyze(html);
        assert!(!a.lazy_lcp_candidate, "first eager <img> must not set lazy_lcp_candidate");
    }

    // ── render_blocking_in_head ───────────────────────────────────────────────

    #[test]
    fn render_blocking_script_in_head_is_counted() {
        let html = r#"<!doctype html><html><head>
            <script src="/app.js"></script>
        </head><body></body></html>"#;
        let a = analyze(html);
        assert_eq!(a.render_blocking_in_head, 1);
        assert!(a.script_urls[0].render_blocking);
    }

    #[test]
    fn deferred_script_in_head_is_not_blocking() {
        let html = r#"<!doctype html><html><head>
            <script src="/app.js" defer></script>
        </head><body></body></html>"#;
        let a = analyze(html);
        assert_eq!(a.render_blocking_in_head, 0);
        assert!(!a.script_urls[0].render_blocking);
    }

    #[test]
    fn script_in_body_is_not_blocking() {
        let html = r#"<!doctype html><html><head></head><body>
            <script src="/app.js"></script>
        </body></html>"#;
        let a = analyze(html);
        assert_eq!(a.render_blocking_in_head, 0);
    }

    // ── inline_style_bytes ────────────────────────────────────────────────────

    #[test]
    fn inline_style_bytes_counted() {
        let css = "body { color: red; }";
        let html = format!(r#"<!doctype html><html><head><style>{css}</style></head><body></body></html>"#);
        let a = analyze(&html);
        assert_eq!(a.inline_style_bytes, css.len() as u64);
    }

    #[test]
    fn multiple_style_blocks_are_summed() {
        let css1 = "body { margin: 0; }";
        let css2 = "h1 { font-size: 2rem; }";
        let html = format!(
            r#"<!doctype html><html><head><style>{css1}</style></head><body><style>{css2}</style></body></html>"#
        );
        let a = analyze(&html);
        assert_eq!(a.inline_style_bytes, (css1.len() + css2.len()) as u64);
    }

    // ── preload_as_style_is_render_blocking ───────────────────────────────────

    #[test]
    fn preload_as_style_is_render_blocking() {
        let html = r#"<!doctype html><html><head>
            <link rel="preload" as="style" href="/fonts.css">
        </head><body></body></html>"#;
        let a = analyze(html);
        assert_eq!(a.render_blocking_in_head, 1, "preload as=style must count as render-blocking");
        assert!(
            a.stylesheet_urls.iter().any(|r| r.render_blocking),
            "the preloaded stylesheet must be flagged render_blocking"
        );
    }

    #[test]
    fn preload_as_font_is_not_render_blocking() {
        let html = r#"<!doctype html><html><head>
            <link rel="preload" as="font" href="/font.woff2">
        </head><body></body></html>"#;
        let a = analyze(html);
        assert_eq!(a.render_blocking_in_head, 0);
    }

    #[test]
    fn preload_as_script_is_not_render_blocking() {
        // preload-as-script hints the fetch but does not execute the script — not render-blocking.
        // This is the anti-theater boundary: <link rel=preload as=script> ≠ <script src=...>.
        let html = r#"<!doctype html><html><head>
            <link rel="preload" as="script" href="/app.js">
        </head><body></body></html>"#;
        let a = analyze(html);
        assert_eq!(a.render_blocking_in_head, 0);
        assert_eq!(a.script_urls.len(), 1);
        assert!(!a.script_urls[0].render_blocking);
    }

    // ── async script ──────────────────────────────────────────────────────────

    #[test]
    fn async_script_in_head_is_not_blocking() {
        let html = r#"<!doctype html><html><head>
            <script src="/app.js" async></script>
        </head><body></body></html>"#;
        let a = analyze(html);
        assert_eq!(a.render_blocking_in_head, 0);
        assert!(!a.script_urls[0].render_blocking);
    }

    // ── img_missing_dimensions ────────────────────────────────────────────────

    #[test]
    fn img_missing_both_dimensions_is_counted() {
        let html = r#"<!doctype html><html><body>
            <img src="/hero.jpg">
        </body></html>"#;
        let a = analyze(html);
        assert_eq!(a.img_missing_dimensions, 1);
    }

    #[test]
    fn img_missing_one_dimension_is_counted() {
        let html = r#"<!doctype html><html><body>
            <img src="/hero.jpg" width="800">
        </body></html>"#;
        let a = analyze(html);
        assert_eq!(a.img_missing_dimensions, 1);
    }

    #[test]
    fn img_with_both_dimensions_is_not_counted() {
        let html = r#"<!doctype html><html><body>
            <img src="/hero.jpg" width="800" height="400">
        </body></html>"#;
        let a = analyze(html);
        assert_eq!(a.img_missing_dimensions, 0);
    }

    // ── inline_script_bytes ───────────────────────────────────────────────────

    #[test]
    fn inline_script_body_is_counted() {
        let body = r#"console.log("hello");"#;
        let html = format!(r#"<!doctype html><html><head><script>{body}</script></head><body></body></html>"#);
        let a = analyze(&html);
        assert_eq!(a.inline_script_bytes, body.len() as u64);
        assert!(a.script_urls.is_empty(), "inline script must not produce a script_url entry");
    }

    // ── script with src + inline body ─────────────────────────────────────────

    #[test]
    fn script_with_src_and_body_does_not_count_inline_bytes() {
        // When <script src="..."> also has inline content, the external src
        // path is taken and the inline body must NOT roll into inline_script_bytes.
        let html = r#"<!doctype html><html><head>
            <script src="/app.js">console.log("this body is ignored");</script>
        </head><body></body></html>"#;
        let a = analyze(html);
        assert_eq!(a.inline_script_bytes, 0, "inline body of a src-script must not count");
        assert_eq!(a.script_urls.len(), 1);
    }

    // ── data: URI filtering ───────────────────────────────────────────────────

    #[test]
    fn data_uri_img_src_is_not_added_to_image_urls() {
        let html = r#"<!doctype html><html><body>
            <img src="data:image/gif;base64,R0lGOD" width="1" height="1">
        </body></html>"#;
        let a = analyze(html);
        assert!(a.image_urls.is_empty(), "data: URI must not be added to image_urls");
    }

    // ── resolve() edge cases ──────────────────────────────────────────────────

    #[test]
    fn relative_script_url_resolves_against_base() {
        // /app.js relative to https://example.com/ must resolve to the full URL.
        let html = r#"<!doctype html><html><head>
            <script src="/app.js"></script>
        </head><body></body></html>"#;
        let a = analyze(html);
        assert_eq!(a.script_urls.len(), 1);
        assert_eq!(a.script_urls[0].url, "https://example.com/app.js");
    }

    #[test]
    fn protocol_relative_url_is_resolved() {
        // //cdn.example.com/lib.js must be treated as https: (inheriting the base scheme).
        let html = r#"<!doctype html><html><head>
            <script src="//cdn.example.com/lib.js"></script>
        </head><body></body></html>"#;
        let a = analyze(html);
        assert_eq!(a.script_urls.len(), 1);
        assert_eq!(a.script_urls[0].url, "https://cdn.example.com/lib.js");
    }

    #[test]
    fn javascript_uri_in_script_src_is_filtered() {
        // javascript: URIs must be silently dropped — they are not real resource loads.
        let html = r#"<!doctype html><html><head>
            <script src="javascript:void(0)"></script>
        </head><body></body></html>"#;
        let a = analyze(html);
        assert!(a.script_urls.is_empty(), "javascript: URI must not produce a script_url entry");
    }

    // ── classify ─────────────────────────────────────────────────────────────

    #[test]
    fn classify_url_with_query_string_uses_extension() {
        // Query string before the extension must not confuse the classifier.
        assert_eq!(classify("/style.css?v=123", None), AssetKind::Css);
        assert_eq!(classify("/app.js?v=abc", None), AssetKind::Js);
    }

    #[test]
    fn classify_content_type_with_charset_suffix() {
        // Content-type with charset must still match.
        assert_eq!(classify("/x", Some("text/css; charset=utf-8")), AssetKind::Css);
        assert_eq!(classify("/x", Some("text/html; charset=utf-8")), AssetKind::Html);
    }

    #[test]
    fn classify_json_extension_is_js() {
        // .json is intentionally classified as Js — confirm the intent holds.
        assert_eq!(classify("/data.json", None), AssetKind::Js);
    }

    #[test]
    fn classify_no_extension_no_content_type_is_other() {
        assert_eq!(classify("/api/endpoint", None), AssetKind::Other);
        assert_eq!(classify("/no-ext", None), AssetKind::Other);
    }

    #[test]
    fn classify_content_type_wins_over_extension() {
        // Content-type takes priority over extension when both are present.
        assert_eq!(classify("/file.js", Some("text/css")), AssetKind::Css);
    }

    // ── stylesheet render-blocking ────────────────────────────────────────────

    #[test]
    fn stylesheet_in_head_is_render_blocking() {
        // Positive case: a plain <link rel="stylesheet"> in <head> must increment
        // render_blocking_in_head and be flagged render_blocking on the ResourceRef.
        let html = r#"<!doctype html><html><head>
            <link rel="stylesheet" href="/main.css">
        </head><body></body></html>"#;
        let a = analyze(html);
        assert_eq!(a.render_blocking_in_head, 1);
        assert_eq!(a.stylesheet_urls.len(), 1);
        assert!(a.stylesheet_urls[0].render_blocking);
    }

    #[test]
    fn disabled_stylesheet_is_not_render_blocking() {
        // The `disabled` attribute on a <link rel="stylesheet"> prevents the browser
        // from loading it — gnomon must not count it as render-blocking.
        let html = r#"<!doctype html><html><head>
            <link rel="stylesheet" href="/late.css" disabled>
        </head><body></body></html>"#;
        let a = analyze(html);
        assert_eq!(a.render_blocking_in_head, 0);
        assert!(!a.stylesheet_urls[0].render_blocking);
    }

    // ── stylesheet in body / print media ─────────────────────────────────────

    #[test]
    fn stylesheet_in_body_is_not_render_blocking() {
        let html = r#"<!doctype html><html><head></head><body>
            <link rel="stylesheet" href="/late.css">
        </body></html>"#;
        let a = analyze(html);
        assert_eq!(a.render_blocking_in_head, 0);
        assert_eq!(a.stylesheet_urls.len(), 1);
        assert!(!a.stylesheet_urls[0].render_blocking);
    }

    #[test]
    fn print_media_stylesheet_is_not_render_blocking() {
        let html = r#"<!doctype html><html><head>
            <link rel="stylesheet" href="/print.css" media="print">
        </head><body></body></html>"#;
        let a = analyze(html);
        assert_eq!(a.render_blocking_in_head, 0);
        assert!(!a.stylesheet_urls[0].render_blocking);
    }

    // ── inline + external aggregation ────────────────────────────────────────

    #[test]
    fn inline_and_external_styles_are_independent() {
        // inline_style_bytes counts <style> content; stylesheet_urls tracks <link>.
        // They must not interfere — both present on a page co-exist cleanly.
        let css = "body { color: red; }";
        let html = format!(
            r#"<!doctype html><html><head>
                <style>{css}</style>
                <link rel="stylesheet" href="/main.css">
            </head><body></body></html>"#
        );
        let a = analyze(&html);
        assert_eq!(a.inline_style_bytes, css.len() as u64);
        assert_eq!(a.stylesheet_urls.len(), 1);
        assert_eq!(a.stylesheet_urls[0].url, "https://example.com/main.css");
    }

    // ── meta tags ─────────────────────────────────────────────────────────────

    #[test]
    fn viewport_meta_is_detected() {
        let html = r#"<!doctype html><html><head>
            <meta name="viewport" content="width=device-width, initial-scale=1">
        </head><body></body></html>"#;
        let a = analyze(html);
        assert!(a.has_viewport_meta);
    }

    #[test]
    fn missing_viewport_meta_is_false() {
        let html = r#"<!doctype html><html><head></head><body></body></html>"#;
        let a = analyze(html);
        assert!(!a.has_viewport_meta);
    }

    #[test]
    fn charset_meta_via_charset_attr_is_detected() {
        let html = r#"<!doctype html><html><head>
            <meta charset="utf-8">
        </head><body></body></html>"#;
        let a = analyze(html);
        assert!(a.has_charset_meta);
    }

    #[test]
    fn charset_meta_via_http_equiv_is_detected() {
        let html = r#"<!doctype html><html><head>
            <meta http-equiv="Content-Type" content="text/html; charset=utf-8">
        </head><body></body></html>"#;
        let a = analyze(html);
        assert!(a.has_charset_meta);
    }
}
