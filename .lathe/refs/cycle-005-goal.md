# Goal — Cycle 5

## What

Auto-discover `gnomon.toml` in the working directory when `--config` is not explicitly passed.

Today: if `--config` is absent, `resolve_budget` ignores any `gnomon.toml` on disk and uses the `--preset` default (`insley`). A `gnomon.toml` committed to the repo is silently bypassed unless the caller explicitly passes `--config gnomon.toml`.

The change: in `resolve_budget` (in `src/budget.rs`), when `config_path` is `None`, check if `./gnomon.toml` exists in the working directory. If it does, load it the same way as if `--config gnomon.toml` had been passed. If it doesn't exist, fall through to the preset default.

The README usage examples should be updated to show that a committed `gnomon.toml` is picked up automatically — the CI workflow is simply:
```sh
gnomon audit https://staging.their-site.com
```
not:
```sh
gnomon audit --config gnomon.toml https://staging.their-site.com
```

`--config` remains available to point at a non-standard path. `--preset` remains the fallback when no file exists.

There is no priority ambiguity: if `--config` is explicitly passed, use it. If not, and `./gnomon.toml` exists, use it. Otherwise, use `--preset`.

## Which Stakeholder

**The CI integrator** (stakeholder 2). Not served in any previous cycle.

## Why Now

Four cycles have served the contributor (cycles 1, 2), the budget owner (cycle 3), and the web performance engineer (cycle 4). The CI integrator has waited longest.

The specific moment that failed: the CI integrator runs `gnomon budget-init --preset mcmaster`, commits the resulting `gnomon.toml`, then adds `gnomon audit https://staging.their-site.com` to their CI pipeline. The audit runs — and reports `preset: insley`. Their committed budget file is silently ignored. The gate is checking against a different budget than the one they configured and committed.

For a CI gate, silent misconfiguration is the worst possible failure mode. "When this passes, I trust it" breaks immediately when the tool is checking a different budget than what the team agreed on. The CI integrator can't defend the gate if they can't verify what it's checking against.

This is a two-line fix in one function. The blast radius is minimal. The improvement to the CI integrator's confidence is immediate: the human output's `preset:` line becomes their verification that the right budget loaded.

## Lived-Experience Note

*I became the CI integrator. The README was clean — exit codes documented, `--format json` mentioned. I ran `gnomon budget-init --preset mcmaster` (step 2). The file was written. I ran `gnomon audit https://example.com` from the same directory, reading the output to verify my budget was in effect. The `preset:` line said `insley`. I looked at the working directory — `gnomon.toml` was right there. I ran `gnomon audit --help`. No mention of auto-discovery. I added `--config gnomon.toml` and ran again. Now it said `mcmaster`.*

*The worst moment: realizing that if I hadn't noticed the `preset:` line in the output, my CI pipeline would have been checking the wrong budget silently for however long it ran. The team committed a performance contract. Gnomon ignored it. That's not a CI gate — that's a footgun.*

*The confidence signal for the CI integrator is: "when this fails, it means something." It means something only if gnomon is enforcing the right contract. Today it can't guarantee that.*
