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

/// TOML-on-disk form. Not all fields from PLAN.md are implemented yet (v0.0.2);
/// unknown keys fail, per the fail-closed disposition.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    preset: String,
    #[serde(default)]
    bytes: Option<ByteBudget>,
    #[serde(default)]
    count: Option<CountBudget>,
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
         # NOTE (v0.0.2): allowlist justifications and per-route overrides are not yet\n\
         # implemented. Adding [[allowlist]] or [routes.*] will fail with an\n\
         # unknown-field error. To temporarily loosen a budget today: raise the number\n\
         # above, explain the reason in your PR description, and link the ticket.\n\
         # Justifications with expiries arrive in the next release.\n"
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

    // ── v0.0.2 exception note ────────────────────────────────────────────────

    #[test]
    fn preset_toml_closing_note_explains_allowlist_not_yet_implemented() {
        // A developer who adds [[allowlist]] from PLAN.md §8 hits an unknown-field
        // error with no guidance. This note must be present to turn that dead end
        // into a clear next step.
        let out = preset_toml(PresetName::Insley);
        assert!(
            out.contains("allowlist justifications and per-route overrides are not yet"),
            "v0.0.2 note missing: {out}"
        );
        assert!(
            out.contains("unknown-field error"),
            "note must name the error the developer will see: {out}"
        );
        assert!(
            out.contains("raise the number"),
            "note must name the available workaround: {out}"
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
}
