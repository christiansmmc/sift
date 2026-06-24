# applybot — Plan 4: Unblock Applications (Screening Data & Pending)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stop jobs from dead-ending as `missing_answer` pendings. Capture the common Easy-Apply screening fields in the profile, add a reusable answer bank that grows from pendings the user resolves, feed both to the agent so it can answer screening questions, and build the Pendências screen where the user answers and resolves blockers.

**Architecture:** Extends the Plan 1-3 stack. Two screening-data sources reach the agent's system prompt: a fixed `screening_json` on the profile (English level, salary expectation, address/CEP, work authorization, availability) and an `answers` table (arbitrary question→answer pairs). When the agent still can't answer a required question, it reports `APPLYBOT_PENDING` with the specific `questions`; the Pendências screen renders those questions, the user answers them (saved to the answer bank), and resolves the pending. Next run, the agent has those answers.

**Tech Stack:** Tauri v2, Rust 2021, rusqlite, React 19 + TS. No new crates.

## Global Constraints

- Platform Windows 11, desktop-only Tauri.
- All code/identifiers/comments/system-prompt in **English**; all user-facing UI strings in **Brazilian Portuguese (pt-BR)**.
- SQLite is the single source of truth; only Rust writes the DB. The agent reports via stdout markers (unchanged from Plan 3).
- The agent still NEVER invents answers and NEVER submits in this plan (submission is Plan 5). Review-before-send unchanged: matches it can fully prepare still become `awaiting_approval`.
- `screening_json` is a JSON object with this exact shape (all keys present; `""` when unknown):
  ```json
  { "english_level": "", "salary_expectation": "", "salary_currency": "", "address": "", "postal_code": "", "work_authorization": "", "availability": "" }
  ```
- Reuse Plan 1-3 interfaces verbatim: `profile::{Profile, get, upsert, is_onboarding_complete}`, `db::pending`, `db::mod::migrate`, `agent::protocol::PendingReport`, `agent::sink::apply_event`, `agent::prompt::build_system_prompt`, the `api` client, the Pendências screen stub.
- Conventional Commits.

## Scope note

This plan delivers screening data + answer bank + the Pendências screen, so a (Plan 3) run can actually queue jobs once the profile has the data. It does NOT include: the 3 execution modes, the Vagas approval screen, or submission — those are **Plan 5**.

---

### Task 1: Schema migration + answer-bank store + profile screening field

**Files:**
- Modify: `src-tauri/src/db/mod.rs` (extend `migrate`)
- Create: `src-tauri/src/db/answers.rs`
- Modify: `src-tauri/src/db/mod.rs` (add `pub mod answers;`)
- Modify: `src-tauri/src/db/profile.rs` (add `screening_json` to `Profile`, `get`, `upsert`)

**Interfaces:**
- Produces:
  - migration adds `profile.screening_json TEXT NOT NULL DEFAULT '{}'`, `pending_actions.questions_json TEXT`, and a new `answers` table.
  - `Profile` gains `screening_json: String` (defaults to `"{}"` when absent).
  - `answers::upsert(conn, question: &str, answer: &str) -> rusqlite::Result<()>` (unique on question; updates on conflict).
  - `answers::list(conn) -> rusqlite::Result<Vec<(String, String)>>` (question, answer), newest first.

- [ ] **Step 1: Extend the migration**

In `src-tauri/src/db/mod.rs`, add to the `migrate` function's statement list (it already swallows "duplicate column" errors):

```rust
        "ALTER TABLE profile ADD COLUMN screening_json TEXT NOT NULL DEFAULT '{}'",
        "ALTER TABLE pending_actions ADD COLUMN questions_json TEXT",
```

And after the column loop, create the answers table (idempotent):

```rust
    conn.execute(
        "CREATE TABLE IF NOT EXISTS answers (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            question   TEXT NOT NULL UNIQUE,
            answer     TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;
```

- [ ] **Step 2: Answer-bank store + test**

Create `src-tauri/src/db/answers.rs`:

```rust
use rusqlite::Connection;

/// Insert or update an answer for a screening question (unique by question).
pub fn upsert(conn: &Connection, question: &str, answer: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO answers (question, answer) VALUES (?1, ?2) \
         ON CONFLICT(question) DO UPDATE SET answer = ?2",
        (question, answer),
    )?;
    Ok(())
}

/// All saved (question, answer) pairs, newest first.
pub fn list(conn: &Connection) -> rusqlite::Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare("SELECT question, answer FROM answers ORDER BY id DESC")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
    rows.collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    #[test]
    fn upsert_then_list_roundtrips() {
        let conn = open_in_memory();
        upsert(&conn, "Years of Java?", "8").unwrap();
        upsert(&conn, "English level?", "Advanced").unwrap();
        let all = list(&conn).unwrap();
        assert_eq!(all.len(), 2);
        assert!(all.iter().any(|(q, a)| q == "Years of Java?" && a == "8"));
    }

    #[test]
    fn upsert_same_question_updates() {
        let conn = open_in_memory();
        upsert(&conn, "Salary?", "10000").unwrap();
        upsert(&conn, "Salary?", "12000").unwrap();
        let all = list(&conn).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].1, "12000");
    }
}
```

Add `pub mod answers;` to `src-tauri/src/db/mod.rs`.

- [ ] **Step 3: Add `screening_json` to Profile**

In `src-tauri/src/db/profile.rs`:
- Add `pub screening_json: String` to the `Profile` struct (after `criteria_json`).
- In `get`, select `screening_json` (8th column) and map it; in the not-found fallback, set `screening_json: "{}".into()` alongside `criteria_json: "{}".into()`.
- In `upsert`, include `screening_json` in the INSERT column list and the `ON CONFLICT DO UPDATE SET`.

Concretely, `get`'s query becomes:
```rust
            "SELECT full_name, email, phone, location, cv_text, criteria_json, screening_json \
             FROM profile WHERE id = 1",
```
with `screening_json: r.get(6)?` added to the row mapping, and the blank fallback:
```rust
    Ok(found.unwrap_or(Profile {
        criteria_json: "{}".into(),
        screening_json: "{}".into(),
        ..Default::default()
    }))
```
`upsert` becomes:
```rust
        "INSERT INTO profile (id, full_name, email, phone, location, cv_text, criteria_json, screening_json, updated_at) \
         VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now')) \
         ON CONFLICT(id) DO UPDATE SET \
            full_name=?1, email=?2, phone=?3, location=?4, cv_text=?5, criteria_json=?6, screening_json=?7, updated_at=datetime('now')",
        (&p.full_name, &p.email, &p.phone, &p.location, &p.cv_text, &p.criteria_json, &p.screening_json),
```
Update the existing `upsert_then_get_roundtrips` and other profile tests that construct `Profile { ... }` literally to include `screening_json: "{}".into()` (or use `..Default::default()`), so they compile.

- [ ] **Step 4: Run tests**

Run: `cd src-tauri && cargo test db::`
Expected: existing db tests (updated) + the two `answers` tests PASS.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: screening_json on profile, answer-bank store, pending questions column"
```

---

### Task 2: Protocol carries unanswered questions; sink persists them

**Files:**
- Modify: `src-tauri/src/agent/protocol.rs` (`PendingReport.questions`)
- Modify: `src-tauri/src/agent/sink.rs` (store `questions_json`)
- Modify: `src-tauri/src/db/pending.rs` (carry `questions_json` through)

**Interfaces:**
- `PendingReport` gains `#[serde(default)] pub questions: Vec<String>`.
- `pending::PendingAction` gains `pub questions: Vec<String>` (parsed from `questions_json`, empty when null).
- `pending::create` signature unchanged; add `pending::create_with_questions(conn, job_id, category, description, questions: &[String]) -> rusqlite::Result<i64>` storing `questions_json`.
- `sink::apply_event` Pending branch calls `create_with_questions` with `p.questions`.

- [ ] **Step 1: Extend PendingReport**

In `src-tauri/src/agent/protocol.rs`, add to `PendingReport`:
```rust
    #[serde(default)]
    pub questions: Vec<String>,
```
Update the existing `parses_pending_and_signals` test's expected struct if it constructs `PendingReport` literally (it pattern-matches fields, so add `questions: _` or assert the new default is empty). Add an assertion:
```rust
    // a missing_answer pending may carry the unanswered questions
    let p2 = parse_line(r#"APPLYBOT_PENDING {"category":"missing_answer","description":"2 fields","questions":["English level?","Expected salary (USD/month)?"]}"#).unwrap();
    if let AgentEvent::Pending(pr) = p2 {
        assert_eq!(pr.questions.len(), 2);
    } else { panic!("expected Pending"); }
```

- [ ] **Step 2: Extend the pending store**

In `src-tauri/src/db/pending.rs`:
- Add `pub questions: Vec<String>` to `PendingAction`.
- In `list_open`, select `questions_json` and parse it: `serde_json::from_str(&qj).unwrap_or_default()` when non-null, else `Vec::new()`. (Add `use serde_json;` as needed; the column may be NULL → read as `Option<String>`.)
- Add:
```rust
pub fn create_with_questions(
    conn: &Connection,
    job_id: Option<i64>,
    category: &str,
    description: &str,
    questions: &[String],
) -> rusqlite::Result<i64> {
    let qj = serde_json::to_string(questions).unwrap_or_else(|_| "[]".into());
    conn.execute(
        "INSERT INTO pending_actions (job_id, category, description, questions_json) \
         VALUES (?1, ?2, ?3, ?4)",
        (job_id, category, description, qj),
    )?;
    Ok(conn.last_insert_rowid())
}
```
`list_open`'s row mapping adds:
```rust
            questions: match r.get::<_, Option<String>>(6)? {
                Some(qj) => serde_json::from_str(&qj).unwrap_or_default(),
                None => Vec::new(),
            },
```
(adjust the SELECT to include `questions_json` as the 7th column, index 6).

- [ ] **Step 3: Sink stores questions**

In `src-tauri/src/agent/sink.rs`, the `AgentEvent::Pending(p)` branch becomes:
```rust
        AgentEvent::Pending(p) => {
            let desc = match &p.url {
                Some(u) => format!("{} ({})", p.description, u),
                None => p.description.clone(),
            };
            pending::create_with_questions(conn, None, &p.category, &desc, &p.questions)?;
            Ok(EventOutcome::Pending)
        }
```
The `LoginRequired` branch keeps using `pending::create` (no questions) — or `create_with_questions(conn, None, "login_required", "...", &[])`. Either is fine; keep `create`.

- [ ] **Step 4: Test the round-trip**

Add to `sink.rs` tests:
```rust
    #[test]
    fn missing_answer_pending_persists_questions() {
        let conn = open_in_memory();
        let ev = AgentEvent::Pending(super::super::protocol::PendingReport {
            category: "missing_answer".into(),
            description: "2 fields".into(),
            url: None,
            questions: vec!["English level?".into(), "Salary (USD/month)?".into()],
        });
        apply_event(&conn, &ev).unwrap();
        let p = pending::list_open(&conn).unwrap();
        assert_eq!(p[0].questions.len(), 2);
    }
```
(Update other `PendingReport { ... }` constructions in tests to include `questions: vec![]`.)

- [ ] **Step 5: Run tests**

Run: `cd src-tauri && cargo test agent:: && cargo test db::pending`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: pendings carry the specific unanswered screening questions"
```

---

### Task 3: Feed screening data + answer bank into the system prompt

**Files:**
- Modify: `src-tauri/src/agent/system_prompt.md`
- Modify: `src-tauri/src/agent/prompt.rs`

**Interfaces:**
- `build_system_prompt(profile: &Profile, answers: &[(String, String)], batch_size: u32) -> String` — NEW signature: now also takes the answer bank. Update the one caller (`runner::start`) to pass it.

- [ ] **Step 1: Add prompt sections**

In `src-tauri/src/agent/system_prompt.md`, add after the `# Search criteria` block:

```markdown
# Screening data (use these to answer application questions)
{{SCREENING}}

# Answer bank (previously confirmed answers — reuse them verbatim when a question matches)
{{ANSWER_BANK}}
```

And add a rule under `# Rules`:

```markdown
8. Answer screening questions using the Screening data and Answer bank above plus the candidate profile. Only when a REQUIRED question cannot be answered from any of these, report APPLYBOT_PENDING with category missing_answer AND include the exact unanswered question texts in a "questions" array, e.g. APPLYBOT_PENDING {"category":"missing_answer","description":"...","questions":["English level?","Expected salary (USD/month)?"]}.
```

- [ ] **Step 2: Update the builder**

In `src-tauri/src/agent/prompt.rs`, change `build_system_prompt`:

```rust
pub fn build_system_prompt(
    profile: &Profile,
    answers: &[(String, String)],
    batch_size: u32,
) -> String {
    let profile_block = format!(
        "Name: {}\nEmail: {}\nPhone: {}\nLocation: {}\n\nResume:\n{}",
        profile.full_name, profile.email, profile.phone, profile.location, profile.cv_text
    );
    let criteria_block = if profile.criteria_json.trim().is_empty() {
        "{}".to_string()
    } else {
        profile.criteria_json.clone()
    };
    let screening_block = if profile.screening_json.trim().is_empty() {
        "{}".to_string()
    } else {
        profile.screening_json.clone()
    };
    let answer_bank = if answers.is_empty() {
        "(none yet)".to_string()
    } else {
        answers
            .iter()
            .map(|(q, a)| format!("- Q: {q}\n  A: {a}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    TEMPLATE
        .replace("{{BATCH_SIZE}}", &batch_size.to_string())
        .replace("{{PROFILE}}", &profile_block)
        .replace("{{CRITERIA}}", &criteria_block)
        .replace("{{SCREENING}}", &screening_block)
        .replace("{{ANSWER_BANK}}", &answer_bank)
}
```

Update the test `fills_placeholders` to pass an answer bank and assert no leftover placeholders:
```rust
    #[test]
    fn fills_placeholders() {
        let p = Profile {
            full_name: "Ada".into(), email: "ada@x.com".into(), phone: "".into(),
            location: "Brazil".into(), cv_text: "8 years backend".into(),
            criteria_json: r#"{"role":"backend"}"#.into(),
            screening_json: r#"{"english_level":"Advanced"}"#.into(),
        };
        let answers = vec![("Years of Java?".to_string(), "8".to_string())];
        let out = build_system_prompt(&p, &answers, 10);
        assert!(out.contains("at most 10 jobs"));
        assert!(out.contains("Advanced"));
        assert!(out.contains("Years of Java?"));
        assert!(!out.contains("{{"));
    }
```

- [ ] **Step 3: Update the caller**

In `src-tauri/src/agent/runner.rs` `start(...)`, load the answer bank and pass it:
```rust
    let answers = {
        let conn = db.lock().unwrap_or_else(|p| p.into_inner());
        crate::db::answers::list(&conn).unwrap_or_default()
    };
    let prompt = crate::agent::prompt::build_system_prompt(&profile, &answers, batch_size);
```

- [ ] **Step 4: Run tests + build**

Run: `cd src-tauri && cargo test agent::prompt && cargo build`
Expected: PASS + clean build.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: inject screening data and answer bank into the agent prompt"
```

---

### Task 4: Commands for answers + screening, and pending list with questions

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs` (register)

**Interfaces:**
- `list_pending` already returns `Vec<PendingAction>` — now each carries `questions` (no signature change, just richer data).
- NEW commands:
  - `list_answers() -> Vec<AnswerPair>` where `AnswerPair { question: String, answer: String }` (Serialize).
  - `save_answer(question: String, answer: String) -> ()`.
- `get_profile`/`save_profile` already round-trip the whole `Profile` including the new `screening_json` (no change needed).

- [ ] **Step 1: Add commands**

In `src-tauri/src/commands.rs`:
```rust
#[derive(Debug, serde::Serialize)]
pub struct AnswerPair {
    pub question: String,
    pub answer: String,
}

#[tauri::command]
pub fn list_answers(state: State<AppState>) -> CmdResult<Vec<AnswerPair>> {
    let conn = state.db.lock().map_err(err)?;
    let rows = crate::db::answers::list(&conn).map_err(err)?;
    Ok(rows.into_iter().map(|(question, answer)| AnswerPair { question, answer }).collect())
}

#[tauri::command]
pub fn save_answer(state: State<AppState>, question: String, answer: String) -> CmdResult<()> {
    let conn = state.db.lock().map_err(err)?;
    crate::db::answers::upsert(&conn, &question, &answer).map_err(err)
}
```
Register `list_answers`, `save_answer` in `lib.rs`'s `generate_handler!`.

- [ ] **Step 2: Build**

Run: `cd src-tauri && cargo build` → success.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat: answer-bank commands"
```

---

### Task 5: Profile screen — screening fields section

**Files:**
- Modify: `src/types.ts` (add `Screening`, extend `Profile`)
- Modify: `src/screens/Profile.tsx` (screening fields)

**Interfaces:**
- `Profile` (TS) gains `screening_json: string`.
- New `Screening` interface mirrors the `screening_json` shape.

- [ ] **Step 1: Types**

In `src/types.ts`:
```ts
export interface Screening {
  english_level: string;
  salary_expectation: string;
  salary_currency: string;
  address: string;
  postal_code: string;
  work_authorization: string;
  availability: string;
}
```
Add `screening_json: string;` to the `Profile` interface.

- [ ] **Step 2: Render + persist screening in Profile.tsx**

In `src/screens/Profile.tsx`, mirror the criteria pattern:
- `const EMPTY_SCREENING: Screening = { english_level:"", salary_expectation:"", salary_currency:"", address:"", postal_code:"", work_authorization:"", availability:"" };`
- `function parseScreening(json: string): Screening { try { return { ...EMPTY_SCREENING, ...JSON.parse(json) }; } catch { return EMPTY_SCREENING; } }`
- Add `const [screening, setScreening] = useState<Screening>(EMPTY_SCREENING);` and set it from `parseScreening(p.screening_json)` in the `useEffect` load.
- Add a `setScreen` helper and a new `<h2>Dados de triagem</h2>` section with pt-BR labelled inputs:
  - Nível de inglês (english_level), Pretensão salarial (salary_expectation) + Moeda (salary_currency, a select: BRL/USD/EUR), Endereço (address), CEP (postal_code), Autorização de trabalho (work_authorization), Disponibilidade (availability).
- In `save()`, include the screening when saving:
  ```tsx
  await api.saveProfile({ ...profile!, criteria_json: JSON.stringify(criteria), screening_json: JSON.stringify(screening) });
  ```
  (Ensure the `analyze()` flow's `saveProfile` calls, if any, also pass `screening_json` — only `save()` persists, so update `save()`.)

Full example of the new section (place after the criteria block, before `prof-actions`):
```tsx
      <h2>Dados de triagem</h2>
      <p className="hint">Respostas comuns que vagas pedem. O agente usa isto para responder sem te incomodar.</p>
      <label>Nível de inglês<input value={screening.english_level} onChange={(e) => setScreen("english_level", e.target.value)} placeholder="Básico / Intermediário / Avançado / Fluente" /></label>
      <label>Pretensão salarial<input value={screening.salary_expectation} onChange={(e) => setScreen("salary_expectation", e.target.value)} placeholder="ex.: 12000" /></label>
      <label>Moeda
        <select value={screening.salary_currency} onChange={(e) => setScreen("salary_currency", e.target.value)}>
          <option value="">—</option><option value="BRL">BRL</option><option value="USD">USD</option><option value="EUR">EUR</option>
        </select>
      </label>
      <label>Endereço<input value={screening.address} onChange={(e) => setScreen("address", e.target.value)} /></label>
      <label>CEP<input value={screening.postal_code} onChange={(e) => setScreen("postal_code", e.target.value)} /></label>
      <label>Autorização de trabalho<input value={screening.work_authorization} onChange={(e) => setScreen("work_authorization", e.target.value)} placeholder="ex.: CLT, PJ, cidadania, visto" /></label>
      <label>Disponibilidade<input value={screening.availability} onChange={(e) => setScreen("availability", e.target.value)} placeholder="ex.: imediata, 30 dias" /></label>
```
with:
```tsx
  const setScreen = (k: keyof Screening, v: string) => setScreening({ ...screening, [k]: v });
```

- [ ] **Step 3: Build**

Run: `npm run build` → tsc 0 errors + vite success.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: screening data fields on the profile screen"
```

---

### Task 6: Pendências screen — answer questions and resolve

**Files:**
- Modify: `src/lib/api.ts` (answer-bank methods + ensure pending type carries questions)
- Modify: `src/types.ts` (`PendingAction.questions`)
- Modify: `src/screens/Pending.tsx` (replace stub)

**Interfaces:**
- `PendingAction` (TS) gains `questions: string[]`.
- `api` gains `listAnswers()`, `saveAnswer(question, answer)`; `listPending()` and `resolvePending(id)` already exist.

- [ ] **Step 1: Types + API**

In `src/types.ts`, add `questions: string[];` to `PendingAction`.
In `src/lib/api.ts` add to `api`:
```ts
  listAnswers: () => invoke<{ question: string; answer: string }[]>("list_answers"),
  saveAnswer: (question: string, answer: string) =>
    invoke<void>("save_answer", { question, answer }),
```

- [ ] **Step 2: Implement the Pendências screen**

Replace `src/screens/Pending.tsx`:
```tsx
import { useEffect, useState } from "react";
import { api } from "../lib/api";
import type { PendingAction } from "../types";

export default function Pending() {
  const [items, setItems] = useState<PendingAction[]>([]);
  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const [status, setStatus] = useState<string | null>(null);

  async function refresh() {
    setItems(await api.listPending());
  }
  useEffect(() => { refresh(); }, []);

  function setDraft(key: string, v: string) {
    setDrafts((d) => ({ ...d, [key]: v }));
  }

  async function saveAnswers(p: PendingAction) {
    setStatus(null);
    try {
      for (const q of p.questions) {
        const key = `${p.id}:${q}`;
        const a = drafts[key]?.trim();
        if (a) await api.saveAnswer(q, a);
      }
      await api.resolvePending(p.id);
      setStatus("Respostas salvas — o agente vai usá-las na próxima busca.");
      await refresh();
    } catch (e) {
      setStatus(`Erro: ${e}`);
    }
  }

  async function dismiss(p: PendingAction) {
    await api.resolvePending(p.id);
    await refresh();
  }

  if (items.length === 0) {
    return <section><h1>Pendências</h1><p>Nada pendente. 🎉</p></section>;
  }

  return (
    <section>
      <h1>Pendências</h1>
      {status && <p className="hint">{status}</p>}
      {items.map((p) => (
        <div key={p.id} style={{ border: "1px solid #ddd", borderRadius: 8, padding: 12, margin: "12px 0" }}>
          <strong>{labelFor(p.category)}</strong>
          <p style={{ color: "#555", margin: "4px 0" }}>{p.description}</p>
          {p.questions.length > 0 ? (
            <>
              {p.questions.map((q) => {
                const key = `${p.id}:${q}`;
                return (
                  <label key={key} style={{ display: "flex", flexDirection: "column", gap: 4, marginBottom: 8 }}>
                    {q}
                    <input value={drafts[key] ?? ""} onChange={(e) => setDraft(key, e.target.value)} />
                  </label>
                );
              })}
              <button onClick={() => saveAnswers(p)}>Salvar respostas e resolver</button>
            </>
          ) : (
            <button onClick={() => dismiss(p)}>Resolver</button>
          )}
        </div>
      ))}
    </section>
  );
}

function labelFor(category: string): string {
  switch (category) {
    case "missing_answer": return "Falta resposta";
    case "login_required": return "Login necessário";
    case "external_application": return "Candidatura externa";
    case "captcha": return "Captcha";
    default: return category;
  }
}
```

- [ ] **Step 3: Build**

Run: `npm run build` → tsc 0 errors + vite success.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: pendencias screen to answer screening questions and resolve blockers"
```

---

### Task 7: Manual validation (end-to-end)

- [ ] **Step 1: Verify the loop closes**

Prereqs: logged into LinkedIn in Chrome, Claude-in-Chrome connected, `claude` on PATH. Run `npm run tauri dev`. Then:
1. Go to **Perfil**, fill the new screening fields (English level, salary + currency, address, CEP, work authorization, availability), Salvar.
2. **Painel** → batch = 2 → **Iniciar**. The agent now has screening data; jobs whose questions are covered should queue as `awaiting_approval` (Vagas count rises) instead of becoming pendings.
3. For any remaining `missing_answer` pending, go to **Pendências**, answer the listed questions, "Salvar respostas e resolver".
4. Run **Iniciar** again — the previously-missing answers are now in the bank; those questions should no longer block.

Record what happened (counts before/after, which questions still blocked, any prompt tuning needed). This step gates the plan.

---

## Plan 4 Self-Review

- **Spec coverage:** fixed screening fields on profile (Tasks 1, 5) ✓; answer bank that grows from resolved pendings (Tasks 1, 2, 6) ✓; both fed to the agent (Task 3) ✓; agent reports the specific unanswered questions (Task 2) ✓; Pendências screen to answer + resolve (Task 6) ✓; the loop closes so future runs improve (Tasks 3, 6, 7) ✓; pt-BR UI / English code ✓; agent still never invents / never submits (prompt unchanged on those, Task 3 only adds answer sources) ✓. Deferred to Plan 5: the 3 modes, Vagas approval screen, submission.
- **Placeholder scan:** No TBD. All code blocks are complete.
- **Type consistency:** `Screening` matches `screening_json` shape across Rust (Global Constraints), TS (`types.ts`), and the Profile form. `PendingReport.questions` ↔ `PendingAction.questions` ↔ TS `PendingAction.questions`. `build_system_prompt`'s new `answers` param is threaded from `runner::start` and `answers::list`. New commands `list_answers`/`save_answer` match the `api` client. `AnswerPair` fields match the TS shape `{question, answer}`.

## Hand-off to Plan 5

Plan 5 adds the three execution modes (Scan / Revisar / Auto) — each selecting a different system-prompt variant and flow — the **Vagas** approval screen (review the generated cover letter + answers, Aprovar/Rejeitar), and the **submission** agent run that actually completes Easy Apply for approved (Revisar) or matched (Auto) applications, marking them `submitted` or creating pendings on blockers.
