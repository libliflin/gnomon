use std::process::ExitCode;

use clap::Parser;
use gnomon::cli::{Cli, Command, OutputFormat};

fn main() -> ExitCode {
    let cli = Cli::parse();

    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("gnomon: runtime init failed: {e}");
            return ExitCode::from(2);
        }
    };

    rt.block_on(async move {
        match run(cli).await {
            Ok(code) => code,
            Err(e) => {
                eprintln!("gnomon: {e:#}");
                ExitCode::from(2)
            }
        }
    })
}

async fn run(cli: Cli) -> anyhow::Result<ExitCode> {
    match cli.command {
        Command::Audit(args) => {
            let budget = gnomon::budget::resolve_budget(args.preset, args.config.as_deref())?;
            let report = gnomon::audit_url(&args.url, &budget).await?;

            match args.format {
                OutputFormat::Human => gnomon::report::print_human(&report, &budget),
                OutputFormat::Json => gnomon::report::print_json(&report)?,
                OutputFormat::Sarif => gnomon::report::print_sarif(&report)?,
            }

            Ok(if report.violations.is_empty() {
                ExitCode::from(0)
            } else {
                ExitCode::from(1)
            })
        }
        Command::BudgetInit(args) => {
            gnomon::budget::write_preset(&args.preset_name, args.path.as_deref())?;
            Ok(ExitCode::from(0))
        }
        Command::Presets => {
            gnomon::budget::print_presets();
            Ok(ExitCode::from(0))
        }
    }
}
