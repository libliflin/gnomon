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
    format!(
        "preset = \"{}\"\n\n\
         [bytes]\n\
         html   = {}\n\
         css    = {}\n\
         js     = {}\n\
         images = {}\n\
         fonts  = {}\n\
         total  = {}\n\n\
         [count]\n\
         requests            = {}\n\
         third_party_domains = {}\n\
         render_blocking     = {}\n\
         fonts               = {}\n",
        p.name,
        p.bytes.html, p.bytes.css, p.bytes.js, p.bytes.images, p.bytes.fonts, p.bytes.total,
        p.count.requests, p.count.third_party_domains, p.count.render_blocking, p.count.fonts,
    )
}
