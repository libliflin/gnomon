use std::path::{Path, PathBuf};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresetName {
    /// insley — zero-JS default, 100 KB total, 0.00 CLS. Hostile to bloat.
    Insley,
    /// mcmaster — table stakes. 300 KB total, 50 KB JS ceiling.
    Mcmaster,
}

impl PresetName {
    fn as_str(self) -> &'static str {
        match self {
            Self::Insley => "insley",
            Self::Mcmaster => "mcmaster",
        }
    }
}

/// One preset's budget, fully resolved.
#[derive(Clone, Debug, Serialize)]
pub struct Preset {
    pub name: &'static str,
    pub bytes: ByteBudget,
    pub count: CountBudget,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ByteBudget {
    pub html: u64,
    pub css: u64,
    pub js: u64,
    pub images: u64,
    pub fonts: u64,
    pub total: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CountBudget {
    pub requests: u32,
    pub third_party_domains: u32,
    pub render_blocking: u32,
    pub fonts: u32,
}

/// What an audit run compares against. Currently equivalent to a single preset;
/// per-route overrides + vitals land in a later version.
#[derive(Clone, Debug, Serialize)]
pub struct Budget {
    pub preset: Preset,
}

impl Budget {
    pub fn from_preset(name: PresetName) -> Self {
        Self {
            preset: match name {
                PresetName::Insley => insley(),
                PresetName::Mcmaster => mcmaster(),
            },
        }
    }
}

pub fn insley() -> Preset {
    Preset {
        name: "insley",
        bytes: ByteBudget {
            html: 8 * 1024,
            css: 14 * 1024,
            js: 0,
            images: 80 * 1024,
            fonts: 0,
            total: 100 * 1024,
        },
        count: CountBudget {
            requests: 6,
            third_party_domains: 0,
            render_blocking: 0,
            fonts: 0,
        },
    }
}

pub fn mcmaster() -> Preset {
    Preset {
        name: "mcmaster",
        bytes: ByteBudget {
            html: 10 * 1024,
            css: 20 * 1024,
            js: 50 * 1024,
            images: 150 * 1024,
            fonts: 60 * 1024,
            total: 300 * 1024,
        },
        count: CountBudget {
            requests: 20,
            third_party_domains: 2,
            render_blocking: 1,
            fonts: 2,
        },
    }
}

/// One `[[allowlist]]` entry in gnomon.toml — a temporary budget relaxation.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AllowlistEntry {
    metric: String,
    budget: u64,
    justification: String,
    expires: String,
}

/// TOML-on-disk form. Not all fields from PLAN.md are implemented yet;
/// unknown keys fail, per the fail-closed disposition.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    preset: String,
    #[serde(default)]
    bytes: Option<ByteBudget>,
    #[serde(default)]
    count: Option<CountBudget>,
    #[serde(default)]
    allowlist: Vec<AllowlistEntry>,
}

// Justification strings that look like placeholders — rejected at load time.
const PLACEHOLDER_JUSTIFICATIONS: &[&str] =
    &["", "todo", "temporary", "temp", "placeholder", "fixme", "tbd", "n/a", "wip"];

fn validate_justification(metric: &str, s: &str) -> anyhow::Result<()> {
    let lower = s.trim().to_lowercase();
    if PLACEHOLDER_JUSTIFICATIONS.contains(&lower.as_str()) {
        anyhow::bail!(
            "gnomon: allowlist entry for {:?} has a placeholder justification {:?} — describe the reason and link a ticket",
            metric,
            s
        );
    }
    Ok(())
}

fn validate_expires_format(metric: &str, s: &str) -> anyhow::Result<()> {
    let b = s.as_bytes();
    if b.len() != 10
        || b[4] != b'-'
        || b[7] != b'-'
        || !b[..4].iter().all(u8::is_ascii_digit)
        || !b[5..7].iter().all(u8::is_ascii_digit)
        || !b[8..].iter().all(u8::is_ascii_digit)
    {
        anyhow::bail!(
            "gnomon: allowlist entry for {:?} has an invalid expires date {:?} — use YYYY-MM-DD format",
            metric,
            s
        );
    }
    Ok(())
}

/// Returns today's date as `YYYY-MM-DD` using the Gregorian calendar algorithm
/// from Howard Hinnant's date library (no external crate required).
fn today_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
        / 86400;
    let z = n + 719468;
    let era = if z >= 0 { z / 146097 } else { (z - 146096) / 146097 };
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

fn apply_allowlist(budget: &mut Budget, entries: &[AllowlistEntry]) -> anyhow::Result<()> {
    let today = today_iso();
    for entry in entries {
        validate_justification(&entry.metric, &entry.justification)?;
        validate_expires_format(&entry.metric, &entry.expires)?;

        if today.as_str() > entry.expires.as_str() {
            anyhow::bail!(
                "gnomon: allowlist entry for {:?} expired on {} ({}) — tighten the budget or update the expiry",
                entry.metric,
                entry.expires,
                entry.justification
            );
        }

        match entry.metric.as_str() {
            "html"                => budget.preset.bytes.html                        = entry.budget,
            "css"                 => budget.preset.bytes.css                         = entry.budget,
            "js"                  => budget.preset.bytes.js                          = entry.budget,
            "images"              => budget.preset.bytes.images                      = entry.budget,
            "bytes.fonts"         => budget.preset.bytes.fonts                       = entry.budget,
            "total"               => budget.preset.bytes.total                       = entry.budget,
            "requests"            => budget.preset.count.requests            = entry.budget as u32,
            "third_party_domains" => budget.preset.count.third_party_domains = entry.budget as u32,
            "render_blocking"     => budget.preset.count.render_blocking     = entry.budget as u32,
            "count.fonts"         => budget.preset.count.fonts               = entry.budget as u32,
            "fonts" => anyhow::bail!(
                "gnomon: allowlist metric \"fonts\" is ambiguous — use \"bytes.fonts\" or \"count.fonts\""
            ),
            other => anyhow::bail!(
                "gnomon: allowlist metric {:?} is not recognized — valid metrics: html, css, js, images, bytes.fonts, total, requests, third_party_domains, render_blocking, count.fonts",
                other
            ),
        }
    }
    Ok(())
}

pub fn resolve_budget(
    preset: PresetName,
    config_path: Option<&Path>,
) -> anyhow::Result<Budget> {
    let auto = Path::new("gnomon.toml");
    let effective = config_path.or_else(|| if auto.exists() { Some(auto) } else { None });

    if let Some(p) = effective {
        let text = std::fs::read_to_string(p)
            .map_err(|e| anyhow::anyhow!("read {}: {e}", p.display()))?;
        let cfg: ConfigFile = toml::from_str(&text)
            .map_err(|e| anyhow::anyhow!("parse {}: {e}", p.display()))?;

        let base_name = match cfg.preset.as_str() {
            "insley" => PresetName::Insley,
            "mcmaster" => PresetName::Mcmaster,
            other => {
                anyhow::bail!(
                    "gnomon.toml: preset = {other:?} is not recognized. Use \"insley\" or \"mcmaster\"."
                );
            }
        };
        let mut budget = Budget::from_preset(base_name);

        if let Some(b) = cfg.bytes {
            budget.preset.bytes = b;
        }
        if let Some(c) = cfg.count {
            budget.preset.count = c;
        }

        apply_allowlist(&mut budget, &cfg.allowlist)?;

        Ok(budget)
    } else {
        Ok(Budget::from_preset(preset))
    }
}

pub fn write_preset(name: &PresetName, path: Option<&Path>) -> anyhow::Result<()> {
    let target: PathBuf = path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("gnomon.toml"));

    if target.exists() {
        anyhow::bail!(
            "gnomon.toml already exists at {}; refusing to overwrite",
            target.display()
        );
    }

    let body = preset_toml(*name);
    std::fs::write(&target, body)?;
    eprintln!("wrote {} ({})", target.display(), name.as_str());
    Ok(())
}

pub fn print_presets() {
    println!("# insley (default)\n");
    println!("{}", preset_toml(PresetName::Insley));
    println!("\n# mcmaster\n");
    println!("{}", preset_toml(PresetName::Mcmaster));
}

fn preset_toml(name: PresetName) -> String {
    let p = match name {
        PresetName::Insley => insley(),
        PresetName::Mcmaster => mcmaster(),
    };

    let bc = |v: u64, zero: &str| -> String {
        if v == 0 {
            format!("# 0 — {zero}")
        } else {
            format!("# {} KiB", v / 1024)
        }
    };
    let cc = |v: u32, zero: &str, nonzero: &str| -> String {
        if v == 0 {
            format!("# 0 — {zero}")
        } else {
            nonzero.to_string()
        }
    };

    let html_c    = bc(p.bytes.html,   "zero HTML");
    let css_c     = bc(p.bytes.css,    "zero CSS");
    let js_c      = bc(p.bytes.js,     "zero JS; the insley standard");
    let images_c  = bc(p.bytes.images, "no images");
    let fonts_b_c = bc(p.bytes.fonts,  "no web fonts; system fonts only");
    let total_c   = bc(p.bytes.total,  "zero total");

    let requests_c = cc(p.count.requests,            "zero requests",              "# includes the HTML document itself");
    let tpd_c      = cc(p.count.third_party_domains, "no third-party domains",     "# max distinct third-party domains");
    let rb_c       = cc(p.count.render_blocking,     "nothing may block first paint", "# count — render-blocking resources allowed");
    let fonts_c    = cc(p.count.fonts,               "no web fonts; system fonts only", "# max web font files");

    let bhtml   = p.bytes.html;
    let bcss    = p.bytes.css;
    let bjs     = p.bytes.js;
    let bimages = p.bytes.images;
    let bfonts  = p.bytes.fonts;
    let btotal  = p.bytes.total;
    let creq    = p.count.requests;
    let ctpd    = p.count.third_party_domains;
    let crb     = p.count.render_blocking;
    let cfonts  = p.count.fonts;
    let pname   = p.name;

    format!(
        "# gnomon.toml — performance budget ({pname} preset)\n\
         # All byte values are brotli-recompressed transfer sizes. CDN headers are not trusted.\n\
         # Edit these values to tighten or loosen your budget. Unknown keys will fail the audit.\n\
         \n\
         preset = \"{pname}\"\n\
         \n\
         [bytes]\n\
         html   = {bhtml:<8} {html_c}\n\
         css    = {bcss:<8} {css_c}\n\
         js     = {bjs:<8} {js_c}\n\
         images = {bimages:<8} {images_c}\n\
         fonts  = {bfonts:<8} {fonts_b_c}\n\
         total  = {btotal:<8} {total_c}\n\
         \n\
         [count]\n\
         requests            = {creq:<4} {requests_c}\n\
         third_party_domains = {ctpd:<4} {tpd_c}\n\
         render_blocking     = {crb:<4} {rb_c}\n\
         fonts               = {cfonts:<4} {fonts_c}\n\
         \n\
         # To temporarily relax a budget, add an [[allowlist]] entry. gnomon will\n\
         # fail the audit when the expiry passes or the justification is a placeholder.\n\
         #\n\
         # Example:\n\
         #   [[allowlist]]\n\
         #   metric        = \"third_party_domains\"\n\
         #   budget        = 3\n\
         #   justification = \"Analytics vendor — PERF-42 — replace by 2026-05-01\"\n\
         #   expires       = \"2026-05-01\"\n\
         #\n\
         # Recognized metrics: html, css, js, images, bytes.fonts, total,\n\
         #   requests, third_party_domains, render_blocking, count.fonts\n\
         # Placeholder justifications (empty, \"TODO\", \"temporary\", etc.) are rejected.\n\
         # Expired entries fail the audit with exit code 2.\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── header ──────────────────────────────────────────────────────────────

    #[test]
    fn preset_toml_header_explains_units() {
        let out = preset_toml(PresetName::Insley);
        assert!(out.contains("# gnomon.toml — performance budget (insley preset)"), "header line missing");
        assert!(out.contains("brotli-recompressed transfer sizes"), "units explanation missing");
        assert!(out.contains("Unknown keys will fail the audit"), "fail-closed notice missing");
    }

    // ── byte comments ───────────────────────────────────────────────────────

    #[test]
    fn nonzero_byte_value_shows_kib_comment() {
        let out = preset_toml(PresetName::Insley);
        // html = 8192 → # 8 KiB
        assert!(out.contains("html   = 8192     # 8 KiB"), "html KiB comment wrong: {out}");
        // css = 14336 → # 14 KiB
        assert!(out.contains("css    = 14336    # 14 KiB"), "css KiB comment wrong: {out}");
    }

    #[test]
    fn zero_byte_value_shows_zero_enforcement_comment() {
        let out = preset_toml(PresetName::Insley);
        // js = 0 in insley
        assert!(out.contains("js     = 0        # 0 — zero JS; the insley standard"), "js zero comment wrong: {out}");
        // fonts = 0 in insley
        assert!(out.contains("fonts  = 0        # 0 — no web fonts; system fonts only"), "fonts zero comment wrong: {out}");
    }

    // ── count comments ──────────────────────────────────────────────────────

    #[test]
    fn nonzero_count_render_blocking_names_count_not_boolean() {
        // The specific ambiguity the goal was written to fix: render_blocking = 1
        // must say "count" not just a bare description.
        let out = preset_toml(PresetName::Mcmaster);
        assert!(
            out.contains("render_blocking     = 1    # count — render-blocking resources allowed"),
            "render_blocking comment must disambiguate 'count' from boolean: {out}"
        );
    }

    #[test]
    fn zero_count_render_blocking_explains_enforcement() {
        let out = preset_toml(PresetName::Insley);
        assert!(
            out.contains("render_blocking     = 0    # 0 — nothing may block first paint"),
            "zero render_blocking comment wrong: {out}"
        );
    }

    #[test]
    fn nonzero_count_requests_names_what_is_included() {
        let out = preset_toml(PresetName::Insley);
        assert!(
            out.contains("requests            = 6    # includes the HTML document itself"),
            "requests comment wrong: {out}"
        );
    }

    #[test]
    fn nonzero_count_third_party_domains_is_labeled() {
        let out = preset_toml(PresetName::Mcmaster);
        assert!(
            out.contains("third_party_domains = 2    # max distinct third-party domains"),
            "third_party_domains comment wrong: {out}"
        );
    }

    #[test]
    fn zero_count_third_party_domains_explains_enforcement() {
        let out = preset_toml(PresetName::Insley);
        assert!(
            out.contains("third_party_domains = 0    # 0 — no third-party domains"),
            "zero third_party_domains comment wrong: {out}"
        );
    }

    // ── mcmaster nonzero bytes ───────────────────────────────────────────────

    #[test]
    fn mcmaster_byte_comments_are_correct_kib() {
        let out = preset_toml(PresetName::Mcmaster);
        assert!(out.contains("html   = 10240    # 10 KiB"), "mcmaster html KiB wrong: {out}");
        assert!(out.contains("css    = 20480    # 20 KiB"), "mcmaster css KiB wrong: {out}");
        assert!(out.contains("js     = 51200    # 50 KiB"), "mcmaster js KiB wrong: {out}");
        assert!(out.contains("images = 153600   # 150 KiB"), "mcmaster images KiB wrong: {out}");
        assert!(out.contains("fonts  = 61440    # 60 KiB"), "mcmaster fonts KiB wrong: {out}");
        assert!(out.contains("total  = 307200   # 300 KiB"), "mcmaster total KiB wrong: {out}");
    }

    #[test]
    fn insley_byte_comments_are_correct_kib() {
        let out = preset_toml(PresetName::Insley);
        assert!(out.contains("images = 81920    # 80 KiB"), "insley images KiB wrong: {out}");
        assert!(out.contains("total  = 102400   # 100 KiB"), "insley total KiB wrong: {out}");
    }

    // ── count.fonts nonzero ──────────────────────────────────────────────────

    #[test]
    fn nonzero_count_fonts_is_labeled() {
        // mcmaster allows 2 web font files — distinct nonzero string in cc()
        let out = preset_toml(PresetName::Mcmaster);
        assert!(
            out.contains("fonts               = 2    # max web font files"),
            "count.fonts nonzero comment wrong: {out}"
        );
    }

    #[test]
    fn zero_count_fonts_explains_enforcement() {
        let out = preset_toml(PresetName::Insley);
        assert!(
            out.contains("fonts               = 0    # 0 — no web fonts; system fonts only"),
            "count.fonts zero comment wrong: {out}"
        );
    }

    // ── allowlist usage documentation ────────────────────────────────────────

    #[test]
    fn preset_toml_closing_section_documents_allowlist_syntax() {
        // The generated gnomon.toml must teach users how to use [[allowlist]].
        // A budget owner who needs a temporary exception must find the syntax here.
        let out = preset_toml(PresetName::Insley);
        assert!(
            out.contains("To temporarily relax a budget, add an [[allowlist]] entry"),
            "allowlist usage section missing: {out}"
        );
        assert!(
            out.contains("metric        = \"third_party_domains\""),
            "allowlist example metric missing: {out}"
        );
        assert!(
            out.contains("justification ="),
            "allowlist justification field missing: {out}"
        );
        assert!(
            out.contains("expires       ="),
            "allowlist expires field missing: {out}"
        );
        assert!(
            out.contains("Placeholder justifications"),
            "rejection behavior not documented: {out}"
        );
        assert!(
            out.contains("exit code 2"),
            "exit code behavior not documented: {out}"
        );
    }

    #[test]
    fn preset_toml_allowlist_docs_appear_after_count_block() {
        // The allowlist docs must be at the end of the file, not mid-file.
        let out = preset_toml(PresetName::Insley);
        let docs_pos = out.find("To temporarily relax a budget").expect("allowlist docs not found");
        let count_pos = out.find("[count]").expect("[count] block not found");
        assert!(docs_pos > count_pos, "allowlist docs must appear after [count] block, not mid-file");
    }

    #[test]
    fn preset_toml_allowlist_docs_present_for_mcmaster() {
        let out = preset_toml(PresetName::Mcmaster);
        assert!(
            out.contains("To temporarily relax a budget"),
            "allowlist usage docs missing from mcmaster preset: {out}"
        );
    }

    // ── resolve_budget: auto-discovery ──────────────────────────────────────

    /// CWD is process-global state. Serialize all tests that touch it.
    static CWD_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Restores the working directory on drop — panic-safe.
    ///
    /// Without this, a panic inside a CWD test leaves the process in the temp
    /// directory. Subsequent CWD tests then run with the wrong working directory,
    /// corrupting their assertions — even after `CWD_LOCK` is re-acquired.
    /// (`CWD_LOCK` is still poisoned by the panic; `CwdGuard` doesn't prevent
    /// that. It prevents the wrong CWD from bleeding into tests that don't hold
    /// the lock.)
    struct CwdGuard(std::path::PathBuf);
    impl CwdGuard {
        fn new() -> Self {
            Self(std::env::current_dir().unwrap())
        }
    }
    impl Drop for CwdGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.0);
        }
    }

    #[test]
    fn resolve_budget_auto_discovers_gnomon_toml_in_cwd() {
        let _lock = CWD_LOCK.lock().unwrap();
        let _cwd = CwdGuard::new();

        let tmp = std::env::temp_dir().join("gnomon_test_autodiscover");
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("gnomon.toml"), preset_toml(PresetName::Mcmaster)).unwrap();

        std::env::set_current_dir(&tmp).unwrap();
        let result = resolve_budget(PresetName::Insley, None);

        let budget = result.unwrap();
        // Loaded mcmaster from gnomon.toml — not the insley default that was passed in.
        assert_eq!(budget.preset.name, "mcmaster");
    }

    #[test]
    fn resolve_budget_explicit_config_overrides_auto_discovery() {
        let _lock = CWD_LOCK.lock().unwrap();
        let _cwd = CwdGuard::new();

        // CWD has a mcmaster gnomon.toml — but explicit --config should win.
        let tmp = std::env::temp_dir().join("gnomon_test_explicit_override");
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("gnomon.toml"), preset_toml(PresetName::Mcmaster)).unwrap();
        let explicit = tmp.join("explicit.toml");
        std::fs::write(&explicit, preset_toml(PresetName::Insley)).unwrap();

        std::env::set_current_dir(&tmp).unwrap();
        let result = resolve_budget(PresetName::Mcmaster, Some(&explicit));

        let budget = result.unwrap();
        assert_eq!(budget.preset.name, "insley");
    }

    #[test]
    fn resolve_budget_falls_back_to_preset_when_no_file() {
        let _lock = CWD_LOCK.lock().unwrap();
        let _cwd = CwdGuard::new();

        // Use a temp dir that definitely has no gnomon.toml.
        let tmp = std::env::temp_dir().join("gnomon_test_no_file");
        std::fs::create_dir_all(&tmp).unwrap();
        let _ = std::fs::remove_file(tmp.join("gnomon.toml")); // ensure absent

        std::env::set_current_dir(&tmp).unwrap();
        let result = resolve_budget(PresetName::Mcmaster, None);

        let budget = result.unwrap();
        assert_eq!(budget.preset.name, "mcmaster");
    }

    #[test]
    fn resolve_budget_auto_discovery_fails_closed_on_malformed_toml() {
        let _lock = CWD_LOCK.lock().unwrap();
        let _cwd = CwdGuard::new();

        let tmp = std::env::temp_dir().join("gnomon_test_malformed");
        std::fs::create_dir_all(&tmp).unwrap();
        // Write a gnomon.toml that is not valid TOML — should error, not fall through to preset.
        std::fs::write(tmp.join("gnomon.toml"), b"this is not valid toml ][[[").unwrap();

        std::env::set_current_dir(&tmp).unwrap();
        let result = resolve_budget(PresetName::Insley, None);

        assert!(result.is_err(), "malformed gnomon.toml must error, not silently fall through");
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("parse"), "error message should mention 'parse': {msg}");
    }

    // ── round-trip: generated TOML parses back ───────────────────────────────

    #[test]
    fn insley_toml_round_trips_through_config_parser() {
        let toml_str = preset_toml(PresetName::Insley);
        // The generated file must parse without error — unknown keys would panic here.
        let cfg: super::ConfigFile = toml::from_str(&toml_str)
            .expect("generated insley TOML must parse cleanly");
        assert_eq!(cfg.preset, "insley");
    }

    #[test]
    fn mcmaster_toml_round_trips_through_config_parser() {
        let toml_str = preset_toml(PresetName::Mcmaster);
        let cfg: super::ConfigFile = toml::from_str(&toml_str)
            .expect("generated mcmaster TOML must parse cleanly");
        assert_eq!(cfg.preset, "mcmaster");
    }

    // ── today_iso ────────────────────────────────────────────────────────────

    #[test]
    fn today_iso_returns_yyyy_mm_dd_format() {
        let d = super::today_iso();
        assert_eq!(d.len(), 10, "today_iso must be 10 chars: {d}");
        assert_eq!(d.as_bytes()[4], b'-', "expected dash at pos 4: {d}");
        assert_eq!(d.as_bytes()[7], b'-', "expected dash at pos 7: {d}");
        assert!(d[..4].chars().all(|c| c.is_ascii_digit()), "year must be digits: {d}");
        assert!(d[5..7].chars().all(|c| c.is_ascii_digit()), "month must be digits: {d}");
        assert!(d[8..].chars().all(|c| c.is_ascii_digit()), "day must be digits: {d}");
        // Should be in a plausible range for a real clock
        assert!(d.as_str() >= "2026-01-01", "date should not be before 2026: {d}");
    }

    // ── allowlist: validation ─────────────────────────────────────────────────

    #[test]
    fn allowlist_placeholder_justification_todo_is_rejected() {
        let mut budget = Budget::from_preset(PresetName::Mcmaster);
        let entries = vec![super::AllowlistEntry {
            metric: "third_party_domains".to_string(),
            budget: 5,
            justification: "TODO".to_string(),
            expires: "2099-01-01".to_string(),
        }];
        let err = super::apply_allowlist(&mut budget, &entries).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("placeholder justification"), "wrong error: {msg}");
        assert!(msg.contains("third_party_domains"), "metric name missing from error: {msg}");
    }

    #[test]
    fn allowlist_empty_justification_is_rejected() {
        let mut budget = Budget::from_preset(PresetName::Mcmaster);
        let entries = vec![super::AllowlistEntry {
            metric: "third_party_domains".to_string(),
            budget: 5,
            justification: "".to_string(),
            expires: "2099-01-01".to_string(),
        }];
        let err = super::apply_allowlist(&mut budget, &entries).unwrap_err();
        assert!(err.to_string().contains("placeholder justification"));
    }

    #[test]
    fn allowlist_temporary_justification_is_rejected() {
        let mut budget = Budget::from_preset(PresetName::Mcmaster);
        let entries = vec![super::AllowlistEntry {
            metric: "render_blocking".to_string(),
            budget: 3,
            justification: "temporary".to_string(),
            expires: "2099-01-01".to_string(),
        }];
        let err = super::apply_allowlist(&mut budget, &entries).unwrap_err();
        assert!(err.to_string().contains("placeholder justification"));
    }

    #[test]
    fn allowlist_invalid_expires_format_is_rejected() {
        let mut budget = Budget::from_preset(PresetName::Mcmaster);
        let entries = vec![super::AllowlistEntry {
            metric: "third_party_domains".to_string(),
            budget: 3,
            justification: "Analytics vendor — PERF-42".to_string(),
            expires: "01/01/2099".to_string(),
        }];
        let err = super::apply_allowlist(&mut budget, &entries).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("invalid expires date"), "wrong error: {msg}");
        assert!(msg.contains("YYYY-MM-DD"), "format hint missing: {msg}");
    }

    #[test]
    fn allowlist_expired_entry_fails_with_actionable_message() {
        let mut budget = Budget::from_preset(PresetName::Mcmaster);
        let entries = vec![super::AllowlistEntry {
            metric: "third_party_domains".to_string(),
            budget: 5,
            justification: "Analytics vendor — PERF-42 — replace by 2020-01-01".to_string(),
            expires: "2020-01-01".to_string(), // definitely in the past
        }];
        let err = super::apply_allowlist(&mut budget, &entries).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("expired on"), "wrong error: {msg}");
        assert!(msg.contains("third_party_domains"), "metric missing: {msg}");
        assert!(msg.contains("2020-01-01"), "expiry date missing: {msg}");
        assert!(msg.contains("tighten the budget or update the expiry"), "action missing: {msg}");
    }

    #[test]
    fn allowlist_unrecognized_metric_is_rejected() {
        let mut budget = Budget::from_preset(PresetName::Mcmaster);
        let entries = vec![super::AllowlistEntry {
            metric: "made_up_metric".to_string(),
            budget: 5,
            justification: "Real justification — PERF-99".to_string(),
            expires: "2099-01-01".to_string(),
        }];
        let err = super::apply_allowlist(&mut budget, &entries).unwrap_err();
        assert!(err.to_string().contains("not recognized"), "wrong error: {err}");
    }

    #[test]
    fn allowlist_ambiguous_fonts_metric_is_rejected() {
        let mut budget = Budget::from_preset(PresetName::Mcmaster);
        let entries = vec![super::AllowlistEntry {
            metric: "fonts".to_string(),
            budget: 5,
            justification: "Need more fonts — PERF-99".to_string(),
            expires: "2099-01-01".to_string(),
        }];
        let err = super::apply_allowlist(&mut budget, &entries).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("ambiguous"), "wrong error: {msg}");
        assert!(msg.contains("bytes.fonts"), "disambiguation hint missing: {msg}");
        assert!(msg.contains("count.fonts"), "disambiguation hint missing: {msg}");
    }

    // ── allowlist: budget application ─────────────────────────────────────────

    #[test]
    fn allowlist_valid_entry_loosens_third_party_domains() {
        let mut budget = Budget::from_preset(PresetName::Mcmaster);
        assert_eq!(budget.preset.count.third_party_domains, 2);
        let entries = vec![super::AllowlistEntry {
            metric: "third_party_domains".to_string(),
            budget: 5,
            justification: "Analytics vendor — PERF-42 — replace by 2099-01-01".to_string(),
            expires: "2099-01-01".to_string(),
        }];
        super::apply_allowlist(&mut budget, &entries).unwrap();
        assert_eq!(budget.preset.count.third_party_domains, 5);
    }

    #[test]
    fn allowlist_valid_entry_loosens_css_bytes() {
        let mut budget = Budget::from_preset(PresetName::Mcmaster);
        assert_eq!(budget.preset.bytes.css, 20 * 1024);
        let entries = vec![super::AllowlistEntry {
            metric: "css".to_string(),
            budget: 50 * 1024,
            justification: "Legacy stylesheet migration — PERF-55 — replace by 2099-01-01".to_string(),
            expires: "2099-01-01".to_string(),
        }];
        super::apply_allowlist(&mut budget, &entries).unwrap();
        assert_eq!(budget.preset.bytes.css, 50 * 1024);
    }

    #[test]
    fn allowlist_valid_entry_loosens_render_blocking() {
        let mut budget = Budget::from_preset(PresetName::Mcmaster);
        assert_eq!(budget.preset.count.render_blocking, 1);
        let entries = vec![super::AllowlistEntry {
            metric: "render_blocking".to_string(),
            budget: 3,
            justification: "Legacy CSS pipeline — PERF-60 — replace by 2099-01-01".to_string(),
            expires: "2099-01-01".to_string(),
        }];
        super::apply_allowlist(&mut budget, &entries).unwrap();
        assert_eq!(budget.preset.count.render_blocking, 3);
    }

    #[test]
    fn allowlist_bytes_fonts_disambiguates_from_count_fonts() {
        let mut budget = Budget::from_preset(PresetName::Mcmaster);
        let original_count_fonts = budget.preset.count.fonts;
        let entries = vec![super::AllowlistEntry {
            metric: "bytes.fonts".to_string(),
            budget: 200 * 1024,
            justification: "Brand fonts — PERF-70 — replace by 2099-01-01".to_string(),
            expires: "2099-01-01".to_string(),
        }];
        super::apply_allowlist(&mut budget, &entries).unwrap();
        assert_eq!(budget.preset.bytes.fonts, 200 * 1024);
        // count.fonts must be unchanged
        assert_eq!(budget.preset.count.fonts, original_count_fonts);
    }

    // ── allowlist: round-trip through TOML parser ─────────────────────────────

    #[test]
    fn gnomon_toml_with_allowlist_parses_and_applies() {
        let toml_str = format!(
            "{}\n\
             [[allowlist]]\n\
             metric        = \"third_party_domains\"\n\
             budget        = 5\n\
             justification = \"Analytics vendor — PERF-42 — replace by 2099-01-01\"\n\
             expires       = \"2099-01-01\"\n",
            preset_toml(PresetName::Mcmaster)
        );
        let cfg: super::ConfigFile = toml::from_str(&toml_str)
            .expect("TOML with [[allowlist]] must parse cleanly");
        assert_eq!(cfg.allowlist.len(), 1);
        assert_eq!(cfg.allowlist[0].metric, "third_party_domains");
        assert_eq!(cfg.allowlist[0].budget, 5);
    }

    #[test]
    fn allowlist_expires_exactly_today_is_accepted() {
        // The expiry date is inclusive: an entry with expires = today must pass.
        // today > today is false, so the entry should not be considered expired.
        let mut budget = Budget::from_preset(PresetName::Mcmaster);
        let today = super::today_iso();
        let entries = vec![super::AllowlistEntry {
            metric: "third_party_domains".to_string(),
            budget: 4,
            justification: "Vendor transition — PERF-10 — expires today".to_string(),
            expires: today,
        }];
        super::apply_allowlist(&mut budget, &entries).unwrap();
        assert_eq!(budget.preset.count.third_party_domains, 4);
    }

    #[test]
    fn allowlist_multiple_entries_different_metrics_all_applied() {
        let mut budget = Budget::from_preset(PresetName::Mcmaster);
        assert_eq!(budget.preset.count.third_party_domains, 2);
        assert_eq!(budget.preset.bytes.css, 20 * 1024);
        let entries = vec![
            super::AllowlistEntry {
                metric: "third_party_domains".to_string(),
                budget: 5,
                justification: "Analytics vendor — PERF-42 — replace by 2099-01-01".to_string(),
                expires: "2099-01-01".to_string(),
            },
            super::AllowlistEntry {
                metric: "css".to_string(),
                budget: 40 * 1024,
                justification: "Legacy stylesheet — PERF-55 — replace by 2099-01-01".to_string(),
                expires: "2099-01-01".to_string(),
            },
        ];
        super::apply_allowlist(&mut budget, &entries).unwrap();
        assert_eq!(budget.preset.count.third_party_domains, 5);
        assert_eq!(budget.preset.bytes.css, 40 * 1024);
    }

    #[test]
    fn allowlist_count_fonts_metric_loosens_count_budget() {
        let mut budget = Budget::from_preset(PresetName::Mcmaster);
        assert_eq!(budget.preset.count.fonts, 2);
        let original_bytes_fonts = budget.preset.bytes.fonts;
        let entries = vec![super::AllowlistEntry {
            metric: "count.fonts".to_string(),
            budget: 6,
            justification: "Brand typeface — PERF-71 — replace by 2099-01-01".to_string(),
            expires: "2099-01-01".to_string(),
        }];
        super::apply_allowlist(&mut budget, &entries).unwrap();
        assert_eq!(budget.preset.count.fonts, 6);
        // bytes.fonts must be unchanged
        assert_eq!(budget.preset.bytes.fonts, original_bytes_fonts);
    }

    #[test]
    fn allowlist_whitespace_padded_placeholder_is_rejected() {
        // "  TODO  " trims to "todo" — must still be rejected.
        let mut budget = Budget::from_preset(PresetName::Mcmaster);
        let entries = vec![super::AllowlistEntry {
            metric: "third_party_domains".to_string(),
            budget: 5,
            justification: "  TODO  ".to_string(),
            expires: "2099-01-01".to_string(),
        }];
        let err = super::apply_allowlist(&mut budget, &entries).unwrap_err();
        assert!(err.to_string().contains("placeholder justification"));
    }

    #[test]
    fn resolve_budget_with_valid_allowlist_loosens_budget_end_to_end() {
        // Full wiring test: write a gnomon.toml with a valid [[allowlist]] entry,
        // call resolve_budget, confirm the returned budget reflects the loosened value.
        // This is the happy path that the expired-entry test cannot cover.
        let _lock = CWD_LOCK.lock().unwrap();
        let _cwd = CwdGuard::new();

        let toml_str = format!(
            "{}\n\
             [[allowlist]]\n\
             metric        = \"third_party_domains\"\n\
             budget        = 7\n\
             justification = \"Analytics vendor — PERF-42 — replace by 2099-01-01\"\n\
             expires       = \"2099-01-01\"\n",
            preset_toml(PresetName::Mcmaster)
        );
        let tmp = std::env::temp_dir().join("gnomon_test_valid_allowlist_e2e");
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("gnomon.toml"), &toml_str).unwrap();

        std::env::set_current_dir(&tmp).unwrap();
        let budget = resolve_budget(PresetName::Insley, None)
            .expect("valid allowlist must not error");
        // preset loaded from file is mcmaster (third_party_domains default = 2)
        // allowlist entry raises it to 7
        assert_eq!(
            budget.preset.count.third_party_domains, 7,
            "allowlist must loosen third_party_domains to 7, got {}",
            budget.preset.count.third_party_domains
        );
    }

    #[test]
    fn gnomon_toml_with_expired_allowlist_fails_resolve_budget() {
        let _lock = CWD_LOCK.lock().unwrap();
        let _cwd = CwdGuard::new();

        let toml_str = format!(
            "{}\n\
             [[allowlist]]\n\
             metric        = \"third_party_domains\"\n\
             budget        = 5\n\
             justification = \"Analytics vendor — PERF-42 — replace by 2020-01-01\"\n\
             expires       = \"2020-01-01\"\n",
            preset_toml(PresetName::Mcmaster)
        );
        let tmp = std::env::temp_dir().join("gnomon_test_expired_allowlist");
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("gnomon.toml"), &toml_str).unwrap();

        std::env::set_current_dir(&tmp).unwrap();
        let result = resolve_budget(PresetName::Insley, None);
        assert!(result.is_err(), "expired allowlist must fail resolve_budget");
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("expired on"), "error must mention expiry: {msg}");
    }
}
