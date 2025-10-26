# RUST COPILOT INSTRUCTIONS — sugar-cli

## Purpose
Give an AI assistant (Copilot-style) exactly the information it needs to make safe, low-risk edits in this repository. Keep it short, concrete, and actionable.

---

## Quick project summary
- **Language:** Rust (stable toolchain pinned via `rust-toolchain.toml`).
- **Main binary crate:** `sugar-cli` (root of repo).
- **Entry point:** `src/main.rs`.
- **Key modules and where to look first:**
  - `src/cli/mod.rs` — Clap CLI definition (enum `Commands`).
  - `src/import_nfts/` — importing existing NFTs (mod + process). New command wiring here.
  - `src/cache.rs` — cache structure and serialization helpers (`Cache`, `CacheItem`, `CacheItems`, `CacheProgram`).
  - `src/*` other feature modules: `launch`, `upload`, `deploy`, `mint`, etc. Each has a `mod.rs` and `process.rs` style.

---

## Coding conventions & API patterns
- **Error handling:** Most modules use `anyhow::Result` for command-level functions. Some define typed errors (e.g. `cache::CacheError`); follow the local style.
- **Re-exports:** Most modules re-export from `process.rs`. When exposing functions to `main.rs`, either re-export from the module root or import from the `process` submodule.
- **Serialization:** `Cache` and `CacheItem` use Serde + `IndexMap` wrappers (`CacheItems`). Respect field names and `#[serde(rename = "...")]`.
- **Types:** Match existing ones (`String`, `Option<String>`, `bool` for `on_chain`).
- **Cache writing:** Use `Cache::write_to_file`; older functions like `write_cache` are obsolete.
- **Paths:** Keep `file_path` as `String`, not `PathBuf`. Use `.to_string_lossy().to_string()` when converting.
- **Exposure:** Functions callable from `main.rs` should be `pub` and/or re-exported in their module’s `mod.rs`.

---

## Build / Test / Lint
- **Build:** `cargo build`
- **Format:** `cargo fmt` (uses `rustfmt.toml`)
- **Test:** `cargo test`
- **Optional Nix environment:** `nix develop --command cargo build`

---

## File map / scope of safe edits
**Safe to modify:**
- `src/cli/mod.rs` — CLI argument definitions.
- `src/<feature>/{mod.rs,process.rs}` — new or extended handlers.
- `src/cache.rs` — type/serialization fixes.
- `src/main.rs` — dispatch and imports only.

**Avoid editing:**
- `Cargo.toml`, `flake.nix`, `.github/`, `scripts/`, or `target/`.
- `/tests/` unless explicitly asked.
- Serialized structs/enums without confirming backward compatibility.

---

## Command wiring checklist
When adding a new CLI command:
1. **Define CLI args** in `src/cli/mod.rs` using `#[derive(Parser)]` or `#[clap(...)]`.
2. **Create handler** in `src/<feature>/process.rs`, returning `anyhow::Result<()>`.
3. **Re-export handler** in `src/<feature>/mod.rs` if needed.
4. **Wire it** in `src/main.rs` under the `Commands` match arm.
5. **Build & test** with `cargo build && cargo test`.

---

## Module conventions
- Each feature lives in `src/<feature>/` with:
  - `mod.rs` (args, pub re-exports)
  - `process.rs` (logic)
- Use `async fn` only if necessary (e.g., network I/O).
- Follow `launch` or `upload` module patterns for new features.
- Prefer functions over traits unless the module already uses traits.

---

## Testing & logging
- Use `tracing` (`info!`, `warn!`, `error!`) — never `println!`.
- Tests go under `tests/`, using `#[tokio::test]` for async handlers.
- When writing to files, prefer `tempfile` in tests.
- Always verify with `cargo test`.

---

## Common pitfalls / gotchas
- Corrupted markdown/code fences in `.rs` files cause parser errors — remove them.
- `CacheItems` wraps `IndexMap` and must be constructed via `CacheItems::new()` + `.insert(...)`.
- Prefer `Cache::write_to_file()`; `write_cache()` does not exist.
- Keep `file_path` as a `String`.
- Re-export or make functions `pub` when wiring through `main.rs`.

---

## Common Copilot pitfalls to avoid
- ❌ Don’t invent functions like `write_cache()` or `get_cache_path()`.
- ❌ Don’t unwrap results (`.unwrap()`) — use `?`.
- ❌ Don’t remove existing `#[derive(...)]` attributes.
- ✅ Use `anyhow::Context` for error context.
- ✅ Use `Cache::write_to_file()` instead of manual I/O.

---

## Imports & idioms quick reference
```rust
use anyhow::{Result, Context};
use clap::Parser;
use indexmap::IndexMap;
use serde::{Serialize, Deserialize};
use tokio::fs;
```

- Prefer `PathBuf` for file args.
- Convert to string via:  
  ```rust
  let s = path.to_string_lossy().to_string();
  ```

---

## PR / Commit guidelines
- Keep PRs atomic: one feature or fix per PR.
- Run:
  ```bash
  cargo fmt
  cargo clippy --fix --allow-dirty --allow-staged
  ```
- Add a one-line summary to `CHANGELOG.md` for new commands.

---

## How to wire a CLI command (example)
1. Add a variant to `Commands` in `src/cli/mod.rs`:
   ```rust
   Import {
       import: PathBuf,
       output: PathBuf,
   }
   ```
2. In `src/import_nfts/mod.rs`, expose:
   ```rust
   pub async fn process_import_nfts_cmd(args: ImportNFTsArgs) -> anyhow::Result<()> { ... }
   ```
3. In `src/main.rs`:
   ```rust
   Commands::Import { import, output } => {
       process_import_nfts_cmd(ImportNFTsArgs { import, output }).await?;
   }
   ```

That’s it — keep changes small, run `cargo build`, and report any errors with file context and compiler output.

---

## Suggested prompts for Copilot
- “Open `src/cache.rs` and ensure new code uses `Cache::write_to_file`. Replace any `write_cache` calls.”
- “Add a `Commands::Import` Clap variant in `src/cli/mod.rs` with `import` and `output` fields, then wire it in `main.rs`.”
- “Fix a compilation error caused by private function exports — make it `pub` or re-export it at the module root.”

---

## Small safety rules
- Don’t modify `Cargo.toml` or dependency versions unless explicitly requested.
- Don’t reformat unrelated files.
- Keep edits minimal and scoped.
- Run `cargo build` after changes; fix compiler errors up to three quick iterations.
- If stuck, output exact rustc errors and file context.

---

## Contact points in code
- `src/import_nfts/mod.rs` — entry and args for import command.
- `src/import_nfts/process.rs` — import implementation.
- `src/cache.rs` — cache schema and file I/O.
- `src/cli/mod.rs` — Clap definitions.
- `src/main.rs` — dispatch logic.
