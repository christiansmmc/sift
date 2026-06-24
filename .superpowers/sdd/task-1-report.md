# Task 1 Report: Scaffold applybot Tauri + React-TS Project

## Status: DONE

## Branch
`plan-1-foundation` — confirmed throughout, no branch change.

## Commands Run

### 1. Directory inspection
```
Get-ChildItem C:\Users\csequ\projects\applybot -Force
# Result: only .git, .superpowers, docs (non-empty dir — cannot scaffold in place)
```

### 2. Scaffold to temp directory
The target directory was non-empty (contained `.git`, `.superpowers/`, `docs/`), so the
strategy was to scaffold into a temp path and copy only the needed files.

```
cd C:\Users\csequ\AppData\Local\Temp
npm create tauri-app@latest applybot-scaffold -- --template react-ts --manager npm --yes
# Installed create-tauri-app@4.6.2
# Template: react-ts
# Result: "Template created!"
```

### 3. Copy scaffold files to project
Copied from `C:\Users\csequ\AppData\Local\Temp\applybot-scaffold\` into `C:\Users\csequ\projects\applybot\`:
- Directories: `src/`, `src-tauri/`, `public/`, `.vscode/`
- Files: `index.html`, `package.json`, `vite.config.ts`, `tsconfig.json`, `tsconfig.node.json`, `README.md`
- **NOT copied**: `docs/`, `.superpowers/`, `.git/` (preserved from original repo)

### 4. .gitignore handling
No top-level `.gitignore` existed in the project. Wrote a merged `.gitignore` containing:
- All scaffold entries (logs, `node_modules`, `dist`, `dist-ssr`, editor files)
- Additional entry: `src-tauri/target/`

### 5. Identity fixes applied
- `src-tauri/Cargo.toml`:
  - `name` changed from `applybot-scaffold` to `applybot`
  - `description` set to `"Job-application agent powered by Claude + Chrome"`
  - `[lib] name` changed from `applybot_scaffold_lib` to `applybot_lib`
- `src-tauri/src/main.rs`:
  - `applybot_scaffold_lib::run()` → `applybot_lib::run()`
- `src-tauri/tauri.conf.json`:
  - `productName`: `applybot`
  - `identifier`: `com.applybot.app` (was `com.csequ.applybot-scaffold`)
  - window `title`: `applybot`
- `package.json`:
  - `name`: `applybot` (was `applybot-scaffold`)

### 6. Data-layer dependencies added to `src-tauri/Cargo.toml`
Scaffold already had `serde` and `serde_json`. Added:
```toml
rusqlite = { version = "0.31", features = ["bundled"] }
chrono = { version = "0.4", features = ["serde"] }
```
Final `[dependencies]` section:
```toml
tauri = { version = "2", features = [] }
tauri-plugin-opener = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
chrono = { version = "0.4", features = ["serde"] }
```

### 7. npm install
```
cd C:\Users\csequ\projects\applybot
npm install
# Added 73 packages in 8s — 1 low severity vulnerability (not blocking)
```

### 8. cargo build (Rust)
First attempt failed because `main.rs` still referenced `applybot_scaffold_lib::run()`.
After fix to `applybot_lib::run()`:
```
cd C:\Users\csequ\projects\applybot\src-tauri
cargo build
# Compiling applybot v0.1.0
# Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.33s
# SUCCESS
```

### 9. npm run build (frontend)
```
cd C:\Users\csequ\projects\applybot
npm run build
# vite v7.x build successful
# dist/ output created
# SUCCESS
```

### 10. git commit
```
git add -A  (excluding node_modules/ and src-tauri/target/ via .gitignore)
git commit -m "chore: scaffold applybot tauri + react-ts project"
# [plan-1-foundation d5d2fb6] 41 files changed, 7551 insertions(+)
```

## Protected Files/Directories
- `docs/` — PRESERVED (already tracked in git, untouched)
- `.superpowers/` — PRESERVED (untouched during copy operation)
- `.git/` — PRESERVED (no destructive git ops)

## Build Results
| Step | Result |
|------|--------|
| `cargo build` (src-tauri) | **SUCCESS** — `Finished dev profile in 8.33s` |
| `npm run build` (frontend) | **SUCCESS** — vite build complete |

## Final Cargo.toml [lib] block
```toml
[lib]
name = "applybot_lib"
crate-type = ["staticlib", "cdylib", "rlib"]
```

## Notes
- The temp scaffold dir (`C:\Users\csequ\AppData\Local\Temp\applybot-scaffold`) can be deleted; it was only used for bootstrapping.
- One npm audit low-severity vulnerability exists (from upstream deps) — not blocking for scaffolding.
- The scaffold used create-tauri-app@4.6.2 with react-ts template, producing Tauri v2 + React 19 + TypeScript 5.8 + Vite 7.
