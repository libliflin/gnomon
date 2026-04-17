use aho_corasick::{AhoCorasick, MatchKind};

/// The built-in blocklist. Patterns are substrings matched against the full URL
/// of every fetched resource. This is intentionally blunt: if one of these
/// strings appears in a URL the site loads, gnomon fails.
///
/// Curated from PLAN.md §5. Not exhaustive.
pub const FORBIDDEN: &[(&str, &str)] = &[
    ("googletagmanager.com/gtm.js", "Google Tag Manager — opens the door to anything"),
    ("assets.adobedtm.com", "Adobe DTM — tag manager"),
    ("connect.facebook.net", "Meta / Facebook pixel"),
    ("cdn.segment.com", "Segment analytics — usually fires N more things"),
    ("widget.intercom.io", "Intercom chat widget — use a mailto link"),
    ("static.intercom.io", "Intercom chat widget — use a mailto link"),
    ("hotjar.com", "Hotjar session replay"),
    ("fullstory.com", "FullStory session replay"),
    ("mouseflow.com", "Mouseflow session replay"),
    ("fonts.googleapis.com", "Google Fonts — self-host and subset your fonts"),
    ("code.jquery.com", "jQuery CDN — justify why in 2026"),
    ("ajax.googleapis.com", "Google-hosted jQuery/Angular — justify why"),
    ("polyfill.io", "polyfill.io — compromised in 2024"),
    ("cdn.polyfill.io", "polyfill.io — compromised in 2024"),
    ("cdn.taboola.com", "Taboola sponsored-content widget"),
    ("outbrain.com", "Outbrain sponsored-content widget"),
    ("doubleclick.net", "DoubleClick ad network"),
    ("googlesyndication.com", "Google ad syndication"),
    ("criteo.com", "Criteo ad network"),
    ("google-analytics.com", "Google Analytics — pick a lighter analytics option"),
    ("googletagservices.com", "Google tag services"),
    ("amazon-adsystem.com", "Amazon ads"),
    ("onetrust.com", "OneTrust cookie banner — fix consent at the source"),
    ("cookielaw.org", "OneTrust cookie banner — fix consent at the source"),
];

pub struct ForbiddenMatcher {
    ac: AhoCorasick,
}

impl ForbiddenMatcher {
    pub fn new() -> Self {
        let patterns: Vec<&str> = FORBIDDEN.iter().map(|(pat, _)| *pat).collect();
        let ac = AhoCorasick::builder()
            .match_kind(MatchKind::LeftmostFirst)
            .ascii_case_insensitive(true)
            .build(&patterns)
            .expect("forbidden patterns compile");
        Self { ac }
    }

    pub fn find(&self, url: &str) -> Option<(&'static str, &'static str)> {
        self.ac
            .find(url)
            .map(|m| FORBIDDEN[m.pattern().as_usize()])
    }
}

impl Default for ForbiddenMatcher {
    fn default() -> Self {
        Self::new()
    }
}
