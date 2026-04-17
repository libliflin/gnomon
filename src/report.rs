use humansize::{BINARY, format_size};
use owo_colors::OwoColorize;

use crate::audit::AuditReport;
use crate::budget::Budget;
use crate::violation::ViolationKind;

pub fn print_human(r: &AuditReport, budget: &Budget) {
    println!();
    println!(
        "{} {}  {}",
        "gnomon".bold(),
        r.gnomon_version.dimmed(),
        format!("({} ms)", r.elapsed_ms).dimmed(),
    );
    println!("  url:    {}", r.url);
    println!("  preset: {}", budget.preset.name);
    println!();

    // --- bytes table ---
    println!("{}", "bytes".bold());
    byte_row("html (brotli)", r.totals.html_brotli, budget.preset.bytes.html);
    byte_row("css", r.totals.css_bytes, budget.preset.bytes.css);
    byte_row("js", r.totals.js_bytes, budget.preset.bytes.js);
    byte_row("images", r.totals.image_bytes, budget.preset.bytes.images);
    byte_row("fonts", r.totals.font_bytes, budget.preset.bytes.fonts);
    byte_row("total", r.totals.total_bytes, budget.preset.bytes.total);
    println!();

    // --- counts table ---
    println!("{}", "counts".bold());
    count_row("requests", r.totals.requests, budget.preset.count.requests);
    count_row(
        "third-party domains",
        r.totals.third_party_domains,
        budget.preset.count.third_party_domains,
    );
    count_row(
        "render-blocking",
        r.totals.render_blocking,
        budget.preset.count.render_blocking,
    );
    count_row("fonts", r.totals.fonts_count, budget.preset.count.fonts);
    println!();

    // Third-party list (contextual, non-fatal info).
    if !r.totals.third_party_list.is_empty() {
        println!("{}", "third parties".bold());
        for host in &r.totals.third_party_list {
            println!("  {host}");
        }
        println!();
    }

    // --- anti-theater notes ---
    let theater_count = r
        .violations
        .iter()
        .filter(|v| v.kind == ViolationKind::Theater)
        .count();
    let forbidden_count = r
        .violations
        .iter()
        .filter(|v| v.kind == ViolationKind::Forbidden)
        .count();

    if theater_count > 0 || forbidden_count > 0 {
        println!("{}", "anti-theater / forbidden".bold());
        for v in &r.violations {
            if matches!(v.kind, ViolationKind::Theater | ViolationKind::Forbidden) {
                let marker = "✗".red();
                println!("  {} {}  {}", marker, v.metric.bold(), v.detail);
            }
        }
        println!();
    }

    // --- verdict ---
    if r.violations.is_empty() {
        println!("{}  {}", "PASS".green().bold(), "all budgets met".dimmed());
    } else {
        println!(
            "{}  {} violation{}",
            "FAIL".red().bold(),
            r.violations.len(),
            if r.violations.len() == 1 { "" } else { "s" }
        );
        println!();
        for v in &r.violations {
            let kind = match v.kind {
                ViolationKind::Bytes => "bytes",
                ViolationKind::Count => "count",
                ViolationKind::Forbidden => "forbidden",
                ViolationKind::Theater => "theater",
                ViolationKind::FetchError => "fetch",
            };
            println!(
                "  {} {:10}  {:24}  {}",
                "✗".red(),
                kind.dimmed(),
                v.metric,
                v.detail
            );
        }
    }
    println!();
}

fn byte_row(label: &str, actual: u64, budget: u64) {
    let (marker, budget_str) = if budget == 0 {
        if actual == 0 {
            ("✓".green().to_string(), "0 B".to_string())
        } else {
            ("✗".red().to_string(), "0 B (zero budget)".to_string())
        }
    } else if actual > budget {
        ("✗".red().to_string(), format_size(budget, BINARY))
    } else {
        ("✓".green().to_string(), format_size(budget, BINARY))
    };
    let actual_str = format_size(actual, BINARY);
    println!("  {marker}  {:18}  {:>10}  /  {:>10}", label, actual_str, budget_str);
}

fn count_row(label: &str, actual: u32, budget: u32) {
    let marker = if actual > budget {
        "✗".red().to_string()
    } else {
        "✓".green().to_string()
    };
    println!("  {marker}  {:18}  {:>10}  /  {:>10}", label, actual, budget);
}

pub fn print_json(r: &AuditReport) -> anyhow::Result<()> {
    let s = serde_json::to_string_pretty(r)?;
    println!("{s}");
    Ok(())
}
