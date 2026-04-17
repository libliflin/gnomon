# Stakeholder Journeys

Concrete first-encounter journeys the champion walks each cycle. Run these steps. Notice where you feel the emotional signal and where you don't.

---

## Journey 1: Web Developer Evaluating Gnomon

**Emotional signal to track:** Trust — does the output feel like a knowledgeable peer reviewed the site?

**Steps:**

1. Find the project. Read `PLAN.md` (there is no `README.md` — the plan doc serves this role for now). Note: does the framing land in 2 minutes or does it require reading the whole document?

2. Install: `cargo install gnomon` (or download from GitHub Releases if binaries exist). Time it. If it takes more than 30 seconds, that's friction.

3. Run against a real site you can audit without permission: `gnomon audit --url https://mcmaster.com`
   - Does it complete without hanging?
   - How long does it take?
   - Does the output appear incrementally or all at once?

4. Read the human output.
   - Can you tell at a glance which budgets passed and which failed?
   - For each violation, can you tell *what* fired and *why it matters*?
   - Is there an ordering — worst violation first, or by category?
   - If there are many violations, is the list navigable or overwhelming?

5. Try a site you expect to pass (you may not find one — that's information). Or run with `--preset mcmaster` against a site you expect to be "okay."

6. Look at the JSON output: `gnomon audit --url https://mcmaster.com --format json`
   - Is it machine-readable and self-explanatory?
   - Are the field names clear without reading the source?

**Watch for:** The moment trust builds (a violation is so specific you immediately know what caused it) and the moment trust breaks (a violation is labeled but not explained, or you don't know what to do with the information).

---

## Journey 2: CI/CD Pipeline Operator

**Emotional signal to track:** Confidence — the gate fires reliably and exits correctly.

**Steps:**

1. Look for CI integration docs. Search `PLAN.md` for "GitHub Actions" and "CI." Note: the plan describes a `gnomon-action` (v0.2 roadmap) and `gnomon ci` subcommand — neither is implemented yet.

2. Try `gnomon ci` — note that it's not a recognized subcommand. What error message do you get?

3. Try the fallback: `gnomon audit --url $URL` in a shell script. Does exit code 1 fire when violations exist?

4. Try `gnomon audit --url $URL --format json`. Pipe to `jq .violations`. Is this usable in a CI script?

5. Look for precompiled binaries on the GitHub Releases page. Are there binaries for common runner platforms (ubuntu-latest x86_64, macos arm64)?

6. Draft a minimal GitHub Actions step in your head. What's the shortest path from "install gnomon" to "fail the build on violations"? How many lines of YAML?

**Watch for:** Every extra step between "I want to add a perf gate" and "the gate is working in CI" is friction for this stakeholder. The moment confidence breaks is when they realize there's no prebuilt path and they have to improvise.

---

## Journey 3: Contributor

**Emotional signal to track:** Momentum — can you go from "I want to fix this" to "I have a working, tested PR" without hitting a wall?

**Steps:**

1. Clone the repo: `git clone https://github.com/libliflin/gnomon && cd gnomon`

2. Build: `cargo build`
   - Does it succeed?
   - Any warnings during build?

3. Run tests: `cargo test`
   - How many tests exist?
   - Are any of them meaningful (testing actual gnomon behavior)?
   - Does `cargo test` output tell you what the project does?

4. Run clippy: `cargo clippy -- -D warnings`
   - Are there diagnostics? How many?
   - Are they actionable?

5. Find where to add a new check (e.g., "detect when `<img>` is missing `width` and `height` and add that to violations").
   - `src/analyze.rs` — analysis already tracks `img_missing_dimensions`
   - `src/audit.rs` — violations are assembled here
   - Is the pattern obvious?

6. Try adding a test for an existing behavior. What test infrastructure exists? (Answer: only `src/lib.rs` has a sanity test; there are no integration tests, no test fixtures, no test HTML documents.)

**Watch for:** Momentum breaks at clippy errors, at the absence of meaningful tests, at unclear module boundaries. Momentum builds when making a change is obvious and adding a test is obvious.

---

## Journey 4: insley-web Author

**Emotional signal to track:** Conviction — does using gnomon feel like the standard is real and enforced?

**Steps:**

1. Assume the insley-web site is built to `./dist`. Try: `gnomon audit --dir ./dist`
   - This will fail — `--dir` is not implemented.

2. Workaround: Spin up a local server, e.g., `python3 -m http.server 8080 --directory ./dist` and run `gnomon audit --url http://localhost:8080`.

3. Check that the audit produces meaningful results:
   - Are violations real or are they artifacts of the localhost setup?
   - Does the HTML byte budget make sense for a site built to the insley standard?

4. After a perf win (e.g., reduced a CSS file), try `gnomon budget tighten` to lock in the win.
   - This will fail — `budget tighten` is not implemented.

5. Check the budget file format. Try `gnomon budget-init --preset insley` to generate a starter `gnomon.toml`.
   - Does it generate a valid TOML that `gnomon audit` will accept via `--config`?

6. Check that per-route overrides in `gnomon.toml` work (they don't yet — per-route config is not parsed).

**Watch for:** Every missing feature is a workaround. Conviction breaks when the path from "I built the site" to "I know if it meets the standard" requires more than two commands.

---

## Journey 5: Library Consumer

**Emotional signal to track:** Predictability — you don't have to think about gnomon.

**Steps:**

1. In a new Rust project: `cargo add gnomon`

2. Add to `Cargo.toml`:
   ```toml
   [dependencies]
   gnomon = "0.0.2"
   tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
   ```

3. Write a minimal usage:
   ```rust
   use gnomon::{audit_url, Budget, Preset};
   
   #[tokio::main]
   async fn main() {
       let budget = Budget::from_preset(gnomon::budget::PresetName::Insley);
       let report = audit_url("https://example.com", &budget).await.unwrap();
       for v in &report.violations {
           println!("{}: {}", v.metric, v.detail);
       }
   }
   ```

4. Does it compile? Are the imports clear from the `pub use` exports in `lib.rs`?
   - `audit_url` is exported
   - `Budget` is exported
   - `Preset` is exported — but `Budget::from_preset` needs `PresetName` which is in `gnomon::budget`
   - Note: `PresetName` is not re-exported from the top level — must be accessed as `gnomon::budget::PresetName`

5. Run against a real URL. Does it produce a meaningful `AuditReport`?

6. Pattern-match on `ViolationKind`. Is the enum self-documenting?

**Watch for:** Any time the consumer has to read gnomon's source to understand how to use it is a predictability failure. The moment trust builds is when `AuditReport` gives you everything you need without needing internal types.
