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
