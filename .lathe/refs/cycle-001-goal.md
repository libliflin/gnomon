# Goal — Cycle 1

## What

Fix all 9 clippy errors so `cargo clippy -- -D warnings` exits clean.

The errors are:

**7× `collapsible_if` in `src/analyze.rs` and `src/audit.rs`:**
- `analyze.rs:151` — nested `if let Ok(u)` / `if let Some(h)`
- `analyze.rs:192` — nested `if let Some(s)` / `if let Some(resolved)`
- `analyze.rs:207` — nested `if let Some(src)` / `if let Some(resolved)`
- `analyze.rs:261` — nested `if let Some(v)` / `if v.name() == tag`
- `audit.rs:68` — nested `if let` (run `cargo clippy` for exact span)
- `audit.rs:121` — nested `if let` (run `cargo clippy` for exact span)
- `audit.rs:225` — nested `if let Some((pat, reason))` / `if flagged.insert(...)`

Clippy's suggested fix in each case: collapse with `&&` let-chain syntax:
```rust
// before
if let Some(x) = foo {
    if let Some(y) = bar(x) {
        ...
    }
}
// after
if let Some(x) = foo
    && let Some(y) = bar(x)
{
    ...
}
```

**1× `needless_lifetimes` in `src/forbidden.rs:50`:**
```rust
// before
pub fn find<'a>(&self, url: &'a str) -> Option<(&'static str, &'static str)>
// after
pub fn find(&self, url: &str) -> Option<(&'static str, &'static str)>
```

## Which Stakeholder

**The contributor** (stakeholder 3). Step 4 of their journey is `cargo clippy -- -D warnings`. Nine errors on first checkout kills momentum immediately. A project that bills itself as a CI gate — something that enforces quality standards — should itself pass its own quality gates. The irony of gnomon shipping with clippy errors is not lost on a Rust developer evaluating whether to contribute.

## Why Now

The snapshot says `Clippy: FAIL — 9 diagnostics`. The floor is violated. No stakeholder journey can be walked cleanly until this is fixed: `cargo clippy` is a prerequisite for contributing, and a broken clippy in a project that's meant to enforce standards sends the wrong signal at the worst moment.

This is cycle 1 — no previous goals exist. The floor must be clean before the first real goal lands.

## Lived-Experience Note

*I became the contributor. I cloned the repo (step 1). Build was clean (step 2). Then I ran `cargo clippy -- -D warnings` (step 4). Nine errors immediately — all preventable, none complex. Momentum stopped cold. The codebase is tight, the module layout is clear, and the logic is easy to follow. The clippy errors are not a sign of bad code — they're just not-yet-tidied code. But for a contributor evaluating whether to trust the project, "not yet tidied" reads as "not yet serious." The worst moment was seeing `collapsible_if` errors in `analyze.rs` — the very file they'd most likely touch to add a new check. The fix is 10 minutes of mechanical work. The signal it sends when clean is worth far more than 10 minutes.*
