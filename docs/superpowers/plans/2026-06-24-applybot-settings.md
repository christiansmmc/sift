# applybot — Plan (Settings): Configurações tab + cover-letter style

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Configurações tab whose first setting lets the user choose how the cover letter is written (Curta e simples / Equilibrada / Detalhada / Personalizada), feeding that choice into the agent's prompt so future Revisar runs follow it.

**Architecture:** A key-value `settings` table (Rust-owned). The hardcoded cover-letter rule in `system_prompt.md` becomes a `{{COVER_LETTER_STYLE}}` placeholder; `runner::start` loads the setting, computes the style instruction, and `build_system_prompt` fills the placeholder. A new Configurações React screen reads/writes settings via two generic commands.

**Tech Stack:** Tauri v2, Rust 2021, rusqlite, React 19 + TS. No new deps.

## Global Constraints

- Platform Windows 11, desktop-only Tauri.
- Code/identifiers/comments and the system prompt in **English**; ALL UI strings in **pt-BR**.
- SQLite single source of truth; only Rust writes it.
- Setting keys: `cover_letter_style` ∈ `short|balanced|detailed|custom` (default `balanced`); `cover_letter_custom` (free text, used only for `custom`).
- Applies to FUTURE Revisar generations only; existing queued letters are unchanged. Submission untouched.
- Reuse interfaces verbatim: `db::mod::migrate`, `agent::prompt::build_system_prompt`, `agent::runner::start`, `AppState`, the `api` client, `App.tsx` NAV.
- Conventional Commits.

---

### Task 1: Settings store

**Files:**
- Modify: `src-tauri/src/db/mod.rs` (migrate: create `settings` table; add `pub mod settings;`)
- Create: `src-tauri/src/db/settings.rs`

**Interfaces:**
- `settings::get(conn, key: &str) -> rusqlite::Result<Option<String>>`
- `settings::set(conn, key: &str, value: &str) -> rusqlite::Result<()>` (upsert)
- `settings::get_or(conn, key: &str, default: &str) -> rusqlite::Result<String>`

- [ ] **Step 1: Migration**

In `src-tauri/src/db/mod.rs` `migrate`, after the answers table creation, add (idempotent):
```rust
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;
```
Add `pub mod settings;` with the other module declarations.

- [ ] **Step 2: Store + tests**

Create `src-tauri/src/db/settings.rs`:
```rust
use rusqlite::{Connection, OptionalExtension};

pub fn get(conn: &Connection, key: &str) -> rusqlite::Result<Option<String>> {
    conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| r.get(0))
        .optional()
}

pub fn set(conn: &Connection, key: &str, value: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) \
         ON CONFLICT(key) DO UPDATE SET value = ?2",
        (key, value),
    )?;
    Ok(())
}

pub fn get_or(conn: &Connection, key: &str, default: &str) -> rusqlite::Result<String> {
    Ok(get(conn, key)?.unwrap_or_else(|| default.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    #[test]
    fn set_get_roundtrip_and_default() {
        let conn = open_in_memory();
        assert_eq!(get(&conn, "k").unwrap(), None);
        assert_eq!(get_or(&conn, "k", "def").unwrap(), "def");
        set(&conn, "k", "v1").unwrap();
        set(&conn, "k", "v2").unwrap();
        assert_eq!(get(&conn, "k").unwrap().as_deref(), Some("v2"));
        assert_eq!(get_or(&conn, "k", "def").unwrap(), "v2");
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cd src-tauri && cargo test db::settings`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "feat: key-value settings store"
```

---

### Task 2: Cover-letter style in the prompt

**Files:**
- Modify: `src-tauri/src/agent/system_prompt.md` (Rule 4 → placeholder)
- Modify: `src-tauri/src/agent/prompt.rs` (`cover_letter_instruction` + `build_system_prompt` param)
- Modify: `src-tauri/src/agent/runner.rs` (`start` loads settings, passes instruction)

**Interfaces:**
- `prompt::cover_letter_instruction(style: &str, custom: &str) -> String`
- `build_system_prompt(profile, answers, cover_letter: &str, mode, batch_size) -> String`

- [ ] **Step 1: Placeholder in the template**

In `src-tauri/src/agent/system_prompt.md`, replace the current Rule 4 line (the one about cover letters being specific/quantified/4 paragraphs/no clichés) with:
```markdown
4. Cover letter style — follow this exactly: {{COVER_LETTER_STYLE}}
```

- [ ] **Step 2: Instruction builder + prompt param**

In `src-tauri/src/agent/prompt.rs`:
```rust
pub fn cover_letter_instruction(style: &str, custom: &str) -> String {
    match style {
        "short" => "Keep the cover letter SHORT and simple: at most 2 short paragraphs, first person, casual but professional, as if the candidate wrote it quickly themselves. No clichés, no formal template, plain prose.".to_string(),
        "detailed" => "Write a detailed, specific cover letter: exactly 4 short paragraphs, a concrete company-specific hook, quantified proof of achievements, no clichés ('passionate', 'results-driven'), plain prose.".to_string(),
        "custom" => {
            let c = custom.trim();
            if c.is_empty() {
                // fall back to balanced if custom is selected but empty
                "A balanced cover letter: 3 short paragraphs, specific to the company, one quantified proof. Professional and natural.".to_string()
            } else {
                format!("Follow the candidate's own instructions exactly: {c}")
            }
        }
        _ => "A balanced cover letter: 3 short paragraphs, specific to the company, one quantified proof. Professional and natural.".to_string(),
    }
}
```
Change `build_system_prompt` to take `cover_letter: &str` (place it after `answers`) and add `.replace("{{COVER_LETTER_STYLE}}", cover_letter)` to the replace chain. Update the `fills_placeholders` test to pass a cover-letter string and assert it appears + no leftover `{{`.

- [ ] **Step 3: Runner loads the setting**

In `src-tauri/src/agent/runner.rs` `start`, before building the prompt:
```rust
    let (style, custom) = {
        let conn = db.lock().map_err(|e| e.to_string())?;
        (
            crate::db::settings::get_or(&conn, "cover_letter_style", "balanced").map_err(|e| e.to_string())?,
            crate::db::settings::get(&conn, "cover_letter_custom").map_err(|e| e.to_string())?.unwrap_or_default(),
        )
    };
    let cover_letter = crate::agent::prompt::cover_letter_instruction(&style, &custom);
```
And pass `&cover_letter` into `build_system_prompt(&profile, &answers, &cover_letter, &mode, batch_size)`.

- [ ] **Step 4: Tests + build**

Run: `cd src-tauri && cargo test agent::prompt && cargo build`
Expected: PASS + clean. Add a small test:
```rust
    #[test]
    fn cover_letter_instruction_variants() {
        assert!(cover_letter_instruction("short", "").contains("SHORT"));
        assert!(cover_letter_instruction("detailed", "").contains("4 short paragraphs"));
        assert!(cover_letter_instruction("custom", "use British English").contains("British English"));
        assert!(cover_letter_instruction("custom", "").contains("balanced")); // empty custom falls back
        assert!(cover_letter_instruction("anything", "").contains("balanced"));
    }
```

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "feat: cover-letter style instruction driven by settings"
```

---

### Task 3: Settings commands

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs` (register)

**Interfaces:**
- `get_setting(key: String) -> Option<String>`
- `set_setting(key: String, value: String) -> ()`

- [ ] **Step 1: Commands**

In `src-tauri/src/commands.rs`:
```rust
#[tauri::command]
pub fn get_setting(state: State<AppState>, key: String) -> CmdResult<Option<String>> {
    let conn = state.db.lock().map_err(err)?;
    crate::db::settings::get(&conn, &key).map_err(err)
}

#[tauri::command]
pub fn set_setting(state: State<AppState>, key: String, value: String) -> CmdResult<()> {
    let conn = state.db.lock().map_err(err)?;
    crate::db::settings::set(&conn, &key, &value).map_err(err)
}
```
Register `get_setting`, `set_setting` in `lib.rs`'s `generate_handler!`.

- [ ] **Step 2: Build**

Run: `cd src-tauri && cargo build` → success.

- [ ] **Step 3: Commit**

```bash
git add -A && git commit -m "feat: get_setting/set_setting commands"
```

---

### Task 4: Configurações screen + nav

**Files:**
- Modify: `src/lib/api.ts` (getSetting/setSetting)
- Create: `src/screens/Settings.tsx`
- Modify: `src/App.tsx` (NAV + screen route + Screen type)

**Interfaces:**
- `api.getSetting(key)`, `api.setSetting(key, value)`.

- [ ] **Step 1: API**

In `src/lib/api.ts` add to `api`:
```ts
  getSetting: (key: string) => invoke<string | null>("get_setting", { key }),
  setSetting: (key: string, value: string) => invoke<void>("set_setting", { key, value }),
```

- [ ] **Step 2: Settings screen**

Create `src/screens/Settings.tsx`:
```tsx
import { useEffect, useState } from "react";
import { api } from "../lib/api";

type Style = "short" | "balanced" | "detailed" | "custom";

const LABELS: Record<Style, string> = {
  short: "Curta e simples (parece escrita por você)",
  balanced: "Equilibrada",
  detailed: "Detalhada (formal)",
  custom: "Personalizada",
};

export default function Settings() {
  const [style, setStyle] = useState<Style>("balanced");
  const [custom, setCustom] = useState("");
  const [status, setStatus] = useState<string | null>(null);

  useEffect(() => {
    api.getSetting("cover_letter_style").then((v) => { if (v) setStyle(v as Style); });
    api.getSetting("cover_letter_custom").then((v) => { if (v) setCustom(v); });
  }, []);

  async function save() {
    setStatus(null);
    try {
      await api.setSetting("cover_letter_style", style);
      await api.setSetting("cover_letter_custom", custom);
      setStatus("Configurações salvas — valem para as próximas buscas.");
    } catch (e) {
      setStatus(`Erro ao salvar: ${e}`);
    }
  }

  return (
    <section>
      <h1>Configurações</h1>

      <div className="card">
        <h2>Estilo da carta de apresentação</h2>
        <p className="hint">Como o agente escreve a carta em cada candidatura (modo Revisar).</p>
        <label className="field">
          Estilo
          <select value={style} onChange={(e) => setStyle(e.target.value as Style)}>
            {(Object.keys(LABELS) as Style[]).map((s) => (
              <option key={s} value={s}>{LABELS[s]}</option>
            ))}
          </select>
        </label>
        {style === "custom" && (
          <label className="field">
            Suas instruções
            <textarea
              rows={5}
              value={custom}
              onChange={(e) => setCustom(e.target.value)}
              placeholder="Ex.: 2 parágrafos, tom informal, em português, foco em impacto e números, sem jargão."
            />
          </label>
        )}
        <div style={{ display: "flex", gap: 12, alignItems: "center", marginTop: 8 }}>
          <button className="btn btn-primary" onClick={save}>Salvar</button>
          {status && <span className="hint">{status}</span>}
        </div>
      </div>
    </section>
  );
}
```

- [ ] **Step 3: Add to nav**

In `src/App.tsx`:
- Add `"settings"` to the `Screen` type union.
- Add `{ key: "settings", label: "Configurações" }` to `NAV`.
- Import `Settings` and add `{screen === "settings" && <Settings />}` in the content area.

- [ ] **Step 4: Build**

Run: `npm run build` → tsc + vite success.

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "feat: configuracoes screen with cover-letter style selector"
```

---

### Task 5: Manual validation

- [ ] **Step 1: Style changes the letter**

Run `npm run tauri dev`. Go to **Configurações** → set **Curta e simples** → Salvar. Then **Painel** → Revisar, batch 1 → Iniciar. In **Vagas**, open the generated cover letter — it should be noticeably shorter/simpler than the detailed style. Switch to **Personalizada** with a custom instruction (e.g. "in English, 2 paragraphs"), Salvar, run again, confirm the letter follows it. (This dovetails with the still-pending Plan 6 submission validation — once the letter style is right, approve + Enviar aprovadas to validate submission end-to-end.)

Record what happened. This step gates the plan.

---

## Plan Self-Review

- **Spec coverage:** Configurações tab (Task 4) ✓; key-value settings store (Task 1) ✓; cover-letter style dropdown with 3 presets + custom free-text (Task 4) ✓; choice drives the prompt via `{{COVER_LETTER_STYLE}}` filled from the setting (Task 2) ✓; applies to future Revisar runs (runner loads per-run, Task 2) ✓; pt-BR UI / English code ✓; no new deps ✓.
- **Placeholder scan:** No TBD. Custom-empty falls back to balanced (explicit).
- **Type consistency:** `Style` union matches the `cover_letter_style` values; `cover_letter_instruction(style, custom)` handles all + default; `build_system_prompt`'s new `cover_letter` param threaded from `runner::start`; `get_setting`/`set_setting` match the `api` client. The submit path (`build_submit_prompt`) is unaffected (it doesn't generate letters).

## Hand-off

After this, resume the **Plan 6 live submission validation** (now with a cover-letter style the user is happy with): Revisar → review the shorter letter → Aprovar → Enviar aprovadas → confirm real submission.
