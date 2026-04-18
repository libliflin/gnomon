# Goal — Cycle 21

## What

Add SARIF 2.1.0 output to gnomon. When `--format sarif` is passed, gnomon writes a SARIF document to stdout. This is the standard format GitHub Code Scanning accepts — it enables violations to appear as PR annotations and in the Security tab without any additional tooling.

**Expected output structure (one violation):**

```json
{
  "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
  "version": "2.1.0",
  "runs": [
    {
      "tool": {
        "driver": {
          "name": "gnomon",
          "version": "0.0.2",
          "informationUri": "https://github.com/libliflin/gnomon"
        }
      },
      "results": [
        {
          "ruleId": "theater/charset_meta",
          "level": "error",
          "message": {
            "text": "missing <meta charset> or http-equiv Content-Type — forces encoding sniff"
          },
          "locations": [
            {
              "physicalLocation": {
                "artifactLocation": {
                  "uri": "https://example.com/"
                }
              }
            }
          ]
        }
      ]
    }
  ]
}
```

**Rules:**
- Each violation maps to a SARIF result with `ruleId = "{kind_label}/{metric}"` (e.g., `"theater/charset_meta"`, `"bytes/css"`, `"count/render_blocking"`, `"forbidden/forbidden"`)
- Use `ViolationKind::label()` (already exists in `violation.rs`) for the kind string
- `level` is always `"error"` — gnomon has no warning state
- `message.text` is the violation's `detail` string
- `locations[0].physicalLocation.artifactLocation.uri` is `report.url`
- When no violations: `results: []`
- `tool.driver.version` is `report.gnomon_version`
- Exit codes unchanged: 0 pass, 1 violations, 2 config/network error

**Files to change:**
- `src/cli.rs` — add `Sarif` to `OutputFormat` enum
- `src/report.rs` — add `pub fn print_sarif(r: &AuditReport) -> anyhow::Result<()>`; use `serde_json::json!` macro or manual `serde_json::Value` — no new named structs required
- `src/main.rs` — handle `OutputFormat::Sarif` in the match arm, same pattern as `OutputFormat::Json`
- `README.md` — update the CI Integration section to show the SARIF workflow

**README CI Integration addition** (append after the existing snippet):

```markdown
To surface violations as GitHub PR annotations via Code Scanning:

```yaml
- name: Run gnomon
  run: gnomon audit https://staging.your-site.com --format sarif > gnomon.sarif
  continue-on-error: true
- name: Upload SARIF to GitHub Code Scanning
  uses: github/codeql-action/upload-sarif@v3
  with:
    sarif_file: gnomon.sarif
```
```

Add tests pinning the new output in `src/report.rs` under `#[cfg(test)] mod tests`:

1. `sarif_output_schema_version_and_tool_set` — SARIF document has `$schema == "https://json.schemastore.org/sarif-2.1.0.json"`, `version == "2.1.0"`, `runs[0].tool.driver.name == "gnomon"`
2. `sarif_output_empty_results_when_no_violations` — AuditReport with empty violations vec → `runs[0].results` is an empty array
3. `sarif_output_single_violation_maps_correctly` — one Theater/charset_meta violation → `results[0].ruleId == "theater/charset_meta"`, `results[0].level == "error"`, `results[0].message.text` equals the detail string, `results[0].locations[0].physicalLocation.artifactLocation.uri == "https://example.com/"`
4. `sarif_output_rule_id_is_kind_slash_metric` — a Bytes/css violation maps to `ruleId == "bytes/css"`

No new dependencies needed — `serde_json` is already in `Cargo.toml`. No changes to the existing JSON or human output paths.

## Which Stakeholder

**The CI integrator** (stakeholder 2). Last served cycle 17 — four cycles ago. Most under-served stakeholder in the current rotation.

Step 5 of their journey: "Try to understand SARIF output (for GitHub code scanning) — doesn't exist yet."

The CI integrator wiring gnomon into GitHub Actions wants violations to appear as PR annotations — the diff view shows exactly where to look, the Security tab tracks them over time, and reviewers see the finding without opening CI logs. All of this works out of the box with `github/codeql-action/upload-sarif@v3`. The only missing piece is a SARIF-formatted output from gnomon.

## Why Now

Four cycles have passed since the CI integrator was served (cycles 18–21 served budget owner, contributor, web perf engineer).

The specific moment that failed: I built the CI pipeline. `gnomon audit https://staging.my-site.com --format json > gnomon.json; echo $?` — exit codes work. The JSON has `pass` and `preset`. I can parse violations. I wrote a shell script to format them as GitHub PR comments using `jq`. It works, but it's fragile — I'm writing JSON parsing in bash.

Then I looked for `--format sarif`. Not there. The GitHub code scanning workflow (`upload-sarif`) is the standard path — no bash parsing, violations become first-class PR annotations that reviewers can dismiss, suppress, and track over time. SARIF is the format that turns gnomon from "a shell command that fails CI" into "a CI tool whose findings appear in the PR diff."

The CI integrator's confidence signal — "when this fails, it means something" — holds for exit codes and `pass: bool`. But confidence in the pipeline means more than a red CI step: it means the reviewer SEES the violation where it matters, in the PR view, not buried in CI logs. SARIF closes that gap.

SARIF is explicitly named in the CI integrator journey as missing. The implementation is self-contained: one new output format, one new function in `report.rs`, four tests. No new dependencies. No changes to existing output.

## Lived-Experience Note

*I became the CI integrator. Build clean, 136 tests, clippy clean. I read the CI Integration section in README — clear install snippet, `gnomon audit https://staging.example.com`. Exit codes documented. I ran `gnomon audit https://example.com --format json; echo $?` — `"pass": false`, exit 1. The confidence signal held: when it fails, it means something.*

*I wanted violations in GitHub PR annotations. That's the standard pattern — `github/codeql-action/upload-sarif@v3` takes a SARIF file and posts violations as inline annotations in the PR diff. I searched the `--help` output: `--format` accepts `human`, `json`. No `sarif`.*

*I tried to work around it. I wrote a bash script using `jq` to parse the JSON violations and post them as PR comments via the GitHub API. It worked, mostly. But it was fragile — different violation formats, special characters in detail strings breaking the JSON, no way to dismiss a violation as "accepted risk." And the violations appeared as PR comments, not as diff annotations. The reviewer had to navigate to the comment and then manually find the relevant code.*

*The worst moment: reading GitHub's documentation for code scanning, seeing the `upload-sarif` action example, realizing gnomon needed exactly one more `--format` option to plug straight in. The data is all there — `violations` array, `kind`, `metric`, `detail`, `url`. The SARIF format is just a different envelope around it. Gnomon has everything needed to produce it.*

*The confidence signal — "when this fails, it means something, and reviewers can see it" — breaks without SARIF. A red CI step means something. A PR annotation in the diff means something different: it means the specific thing that's wrong is pointed at. SARIF is the difference between "build failed, go look at CI logs" and "this violation fires here."*
