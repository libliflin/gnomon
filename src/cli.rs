use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use crate::budget::PresetName;

#[derive(Parser, Debug)]
#[command(
    name = "gnomon",
    version,
    about = "Performance budget auditor. A CI gate, not a dashboard.",
    long_about = "Gnomon audits a URL against a declarative performance budget and \
                  exits 0 (pass) or 1 (fail). It does not produce a score. It produces \
                  pass or fail. See https://github.com/libliflin/gnomon"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Audit a URL against a budget (default: insley preset).
    Audit(AuditArgs),

    /// Write a starter gnomon.toml for a preset.
    #[command(name = "budget-init")]
    BudgetInit(BudgetInitArgs),

    /// Print the built-in presets to stdout.
    Presets,
}

#[derive(clap::Args, Debug)]
pub struct AuditArgs {
    /// URL to audit, e.g. https://mcmaster.com
    pub url: String,

    /// Which built-in preset to audit against. Ignored if --config is set.
    #[arg(long, value_enum, default_value_t = PresetName::Insley)]
    pub preset: PresetName,

    /// Path to a gnomon.toml. If set, overrides --preset.
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
    pub format: OutputFormat,
}

#[derive(clap::Args, Debug)]
pub struct BudgetInitArgs {
    /// Which preset to initialize from.
    #[arg(long = "preset", value_enum, default_value_t = PresetName::Insley)]
    pub preset_name: PresetName,

    /// Where to write. Defaults to ./gnomon.toml; refuses to overwrite.
    #[arg(long)]
    pub path: Option<PathBuf>,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum OutputFormat {
    Human,
    Json,
}
