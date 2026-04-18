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

fn build_sarif(r: &AuditReport) -> serde_json::Value {
    use serde_json::{Value, json};
    use std::collections::HashSet;

    // One rule entry per unique ruleId.
    let mut seen: HashSet<String> = HashSet::new();
    let rules: Vec<Value> = r
        .violations
        .iter()
        .filter_map(|v| {
            let id = format!("{}/{}", v.kind.label(), v.metric);
            if seen.insert(id.clone()) {
                Some(json!({
                    "id": id,
                    "shortDescription": { "text": v.metric }
                }))
            } else {
                None
            }
        })
        .collect();

    let results: Vec<Value> = r
        .violations
        .iter()
        .map(|v| {
            json!({
                "ruleId": format!("{}/{}", v.kind.label(), v.metric),
                "level": "error",
                "message": { "text": v.detail.clone() },
                "locations": [{
                    // Gnomon violations are page-level, not line-level. We use the audited
                    // URL as the artifact URI. GitHub code scanning surfaces these as PR
                    // annotations on the run, not as inline diff comments — correct
                    // behaviour for a tool that audits a URL, not a source file.
                    "physicalLocation": {
                        "artifactLocation": { "uri": r.url.clone() }
                    }
                }]
            })
        })
        .collect();

    json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "gnomon",
                    "version": r.gnomon_version,
                    "informationUri": "https://github.com/libliflin/gnomon",
                    "rules": rules
                }
            },
            "results": results
        }]
    })
}

pub fn print_sarif(r: &AuditReport) -> anyhow::Result<()> {
    let s = serde_json::to_string_pretty(&build_sarif(r))?;
    println!("{s}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::build_sarif;
    use crate::audit::{AuditReport, Totals};
    use crate::analyze::HtmlAnalysis;
    use crate::fetch::Fetched;
    use crate::violation::{Violation, ViolationKind};

    fn minimal_fetched() -> Fetched {
        Fetched {
            url: "https://example.com".to_string(),
            status: 200,
            content_type: None,
            wire_bytes: 0,
            raw_bytes: 0,
            brotli_bytes: 0,
            error: None,
            body: None,
        }
    }

    fn minimal_report(violations: Vec<Violation>) -> AuditReport {
        let pass = violations.is_empty();
        AuditReport {
            url: "https://example.com".to_string(),
            gnomon_version: "0.0.0",
            preset: "insley",
            pass,
            elapsed_ms: 0,
            html: minimal_fetched(),
            html_analysis: HtmlAnalysis::default(),
            resources: vec![],
            totals: Totals::default(),
            violations,
        }
    }

    #[test]
    fn sarif_schema_fields_present() {
        let r = minimal_report(vec![]);
        let v = build_sarif(&r);
        assert_eq!(v["version"], "2.1.0");
        assert!(v["$schema"].as_str().unwrap().contains("sarif-schema-2.1.0"));
        assert!(v["runs"].is_array());
        assert!(v["runs"][0]["tool"]["driver"]["name"] == "gnomon");
    }

    #[test]
    fn sarif_empty_violations_produces_empty_results() {
        let r = minimal_report(vec![]);
        let v = build_sarif(&r);
        let results = v["runs"][0]["results"].as_array().unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn sarif_single_violation_maps_correctly() {
        let violation = Violation {
            kind: ViolationKind::Bytes,
            metric: "css",
            budget: 14336,
            actual: 47000,
            detail: "33 KiB over — main.css (28 KiB)".to_string(),
        };
        let r = minimal_report(vec![violation]);
        let v = build_sarif(&r);
        let result = &v["runs"][0]["results"][0];
        assert_eq!(result["level"], "error");
        assert_eq!(result["message"]["text"], "33 KiB over — main.css (28 KiB)");
        let location_uri = &result["locations"][0]["physicalLocation"]["artifactLocation"]["uri"];
        assert_eq!(location_uri, "https://example.com");
    }

    #[test]
    fn sarif_rule_id_format_is_kind_slash_metric() {
        let violation = Violation {
            kind: ViolationKind::Theater,
            metric: "img_dimensions",
            budget: 0,
            actual: 3,
            detail: "3 <img> element(s) missing explicit width/height — layout shift (CLS)"
                .to_string(),
        };
        let r = minimal_report(vec![violation]);
        let v = build_sarif(&r);
        let rule_id = v["runs"][0]["results"][0]["ruleId"].as_str().unwrap();
        assert_eq!(rule_id, "theater/img_dimensions");
        // Also pinned in the rules array.
        let rules_id = v["runs"][0]["tool"]["driver"]["rules"][0]["id"]
            .as_str()
            .unwrap();
        assert_eq!(rules_id, "theater/img_dimensions");
    }

    #[test]
    fn sarif_duplicate_kind_metric_produces_one_rule_entry() {
        // Two violations with the same kind/metric — one rule entry, two results.
        let v1 = Violation {
            kind: ViolationKind::Forbidden,
            metric: "forbidden_domain",
            budget: 0,
            actual: 1,
            detail: "googlesyndication.com — Google ad syndication".to_string(),
        };
        let v2 = Violation {
            kind: ViolationKind::Forbidden,
            metric: "forbidden_domain",
            budget: 0,
            actual: 1,
            detail: "doubleclick.net — Google ad syndication".to_string(),
        };
        let r = minimal_report(vec![v1, v2]);
        let v = build_sarif(&r);
        let rules = v["runs"][0]["tool"]["driver"]["rules"].as_array().unwrap();
        assert_eq!(rules.len(), 1, "two violations with same ruleId must produce one rule entry");
        let results = v["runs"][0]["results"].as_array().unwrap();
        assert_eq!(results.len(), 2, "two violations must produce two result entries");
    }
}
