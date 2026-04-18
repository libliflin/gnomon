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
    if let Some(p) = config_path {
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
         fonts               = {cfonts:<4} {fonts_c}\n"
    )
}
