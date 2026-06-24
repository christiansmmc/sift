# applybot — Plan 5: Review/Approve + Scan & Revisar Modes

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give the user two run modes — **Scan** (discover matching jobs only) and **Revisar** (prepare cover letter + answers for review) — and a **Vagas** screen where they read the generated cover letter + answers and Aprovar/Rejeitar each application.

**Architecture:** Extends the Plan 1-4 stack. The Dashboard sends a `mode` to `start_search_batch`; the system prompt gains a mode-specific instruction block. In **Scan** the agent reports jobs with no cover letter — the sink saves the job only. In **Revisar** the agent prepares content — the sink creates an `awaiting_approval` application (today's behavior). The Vagas screen reads two lists from the DB: discovered jobs (Scan output) and the review queue (Revisar output), and approves/rejects applications. No submission yet (Plan 6).

**Tech Stack:** Tauri v2, Rust 2021, rusqlite, React 19 + TS. No new crates.

## Global Constraints

- Platform Windows 11, desktop-only Tauri.
- All code/identifiers/comments/system-prompt in **English**; all user-facing UI strings in **pt-BR**.
- SQLite single source of truth; only Rust writes it; agent reports via stdout markers.
- The agent still NEVER invents and NEVER submits in this plan (submission = Plan 6).
- `mode` is one of `"scan"` or `"revisar"` (validated in the command; default `"revisar"`).
- Status strings: existing `awaiting_approval`/`analyzed`/etc. plus NEW `approved` (an application the user accepted; Plan 6 submits it) and `discarded` (rejected). Jobs from Scan keep status `analyzed` with no application row.
- **Sink rule:** an `APPLYBOT_JOB` marker with an empty `cover_letter` creates the job only (Scan); with a non-empty `cover_letter` it also creates the `awaiting_approval` application (Revisar). This is the one behavior change to Plan 3's sink.
- Reuse Plan 1-4 interfaces verbatim: `db::{jobs, applications, pending, answers, profile}`, `agent::{protocol, sink, prompt, runner}`, the `api` client, the Vagas screen stub, the Dashboard.
- Conventional Commits.

## Scope note

Delivers Scan/Revisar modes + the review/approve UI. Does NOT include submission or the Auto mode — those are **Plan 6**. "Aprovar" here only sets status `approved`; nothing is sent to LinkedIn yet.

---

### Task 1: Sink — empty cover letter saves the job only

**Files:**
- Modify: `src-tauri/src/agent/sink.rs`

**Interfaces:**
- `apply_event` `Job` branch: when `j.cover_letter.trim().is_empty()`, insert/update the job and set status `analyzed` but DO NOT create an application; return `EventOutcome::Queued`. When non-empty, behave as today (job + `awaiting_approval` application with content, deduped).

- [ ] **Step 1: Update the Job branch + tests**

In `src-tauri/src/agent/sink.rs`, change the `AgentEvent::Job(j)` branch so the application is only created when there is a cover letter:

```rust
        AgentEvent::Job(j) => {
            let job_id = jobs::insert(
                conn,
                &jobs::NewJob {
                    title: j.title.clone(),
                    company: j.company.clone(),
                    url: j.url.clone(),
                    source: "linkedin".into(),
                },
            )?;
            jobs::set_status(conn, job_id, "analyzed", Some(&j.match_summary))?;
            // Scan mode reports jobs with no cover letter → save the job only.
            // Revisar mode includes a cover letter → also queue an application.
            if !j.cover_letter.trim().is_empty()
                && !applications::has_open_application(conn, job_id)?
            {
                let answers_json = serde_json::to_string(&j.answers).unwrap_or_else(|_| "[]".into());
                applications::create_with_content(conn, job_id, &j.cover_letter, &answers_json)?;
            }
            Ok(EventOutcome::Queued)
        }
```

Add a test (alongside the existing job test):

```rust
    #[test]
    fn scan_job_without_cover_letter_saves_job_only() {
        let conn = open_in_memory();
        let ev = AgentEvent::Job(JobReport {
            title: "Dev".into(), company: "Acme".into(),
            url: "https://linkedin.com/jobs/9".into(),
            match_summary: "strong".into(),
            cover_letter: "".into(), answers: vec![],
        });
        apply_event(&conn, &ev).unwrap();
        assert_eq!(jobs::list(&conn).unwrap().len(), 1);
        assert_eq!(applications::list(&conn).unwrap().len(), 0);
    }
```

- [ ] **Step 2: Run tests**

Run: `cd src-tauri && cargo test agent::sink`
Expected: existing sink tests + `scan_job_without_cover_letter_saves_job_only` PASS.

- [ ] **Step 3: Commit**

```bash
git add -A && git commit -m "feat: scan-mode jobs (no cover letter) persist as job only"
```

---

### Task 2: Mode-aware prompt + threading the mode through start

**Files:**
- Modify: `src-tauri/src/agent/system_prompt.md` (add `{{MODE_INSTRUCTIONS}}`)
- Modify: `src-tauri/src/agent/prompt.rs` (`build_system_prompt` takes `mode`)
- Modify: `src-tauri/src/agent/runner.rs` (`start` takes `mode`)
- Modify: `src-tauri/src/commands.rs` (`start_search_batch` takes `mode`)
- Modify: `src-tauri/src/lib.rs` (unchanged handler list; signature change only)

**Interfaces:**
- `build_system_prompt(profile, answers, mode: &str, batch_size) -> String`.
- `runner::start(db, app, profile, mode: String, batch_size)`.
- `start_search_batch(state, app, mode: Option<String>, batch_size: Option<u32>)` — defaults `mode="revisar"`.

- [ ] **Step 1: Template placeholder**

In `src-tauri/src/agent/system_prompt.md`, replace the `# Your task this run` paragraph's task description with a placeholder. Add near the top, right after `# Your task this run`:

```markdown
{{MODE_INSTRUCTIONS}}
```

Keep the rest (reporting markers, rules) as-is — they apply to both modes (in Scan the agent simply won't produce cover letters).

- [ ] **Step 2: Build the mode block**

In `src-tauri/src/agent/prompt.rs`:

```rust
fn mode_instructions(mode: &str, batch_size: u32) -> String {
    match mode {
        "scan" => format!(
            "MODE: SCAN. Quickly DISCOVER up to {batch_size} jobs that match the criteria. \
For each good match, report APPLYBOT_JOB with title, company, url, and match_summary ONLY. \
Do NOT open Easy Apply, do NOT write a cover letter, do NOT answer screening questions. \
Leave cover_letter as \"\" and answers as []. This is a fast discovery pass."
        ),
        _ => format!(
            "MODE: REVISAR. For up to {batch_size} good Easy-Apply matches, open the application, \
read the screening questions, prepare a tailored cover letter and the answers, and report \
APPLYBOT_JOB with cover_letter and answers filled in. Do NOT submit — the user reviews first."
        ),
    }
}

pub fn build_system_prompt(
    profile: &Profile,
    answers: &[(String, String)],
    mode: &str,
    batch_size: u32,
) -> String {
    // ... existing profile_block / criteria_block / screening_block / answer_bank ...
    TEMPLATE
        .replace("{{MODE_INSTRUCTIONS}}", &mode_instructions(mode, batch_size))
        .replace("{{BATCH_SIZE}}", &batch_size.to_string())
        .replace("{{PROFILE}}", &profile_block)
        .replace("{{CRITERIA}}", &criteria_block)
        .replace("{{SCREENING}}", &screening_block)
        .replace("{{ANSWER_BANK}}", &answer_bank)
}
```

Update the `fills_placeholders` test to pass a mode and assert mode text appears:
```rust
        let out = build_system_prompt(&p, &answers, "revisar", 10);
        assert!(out.contains("MODE: REVISAR"));
        assert!(!out.contains("{{"));
        let scan = build_system_prompt(&p, &answers, "scan", 5);
        assert!(scan.contains("MODE: SCAN"));
```

- [ ] **Step 3: Thread through runner + command**

In `runner::start`, add `mode: String` param and pass `&mode` to `build_system_prompt(&profile, &answers, &mode, batch_size)`.

In `commands.rs` `start_search_batch`, add `mode: Option<String>`; resolve `let mode = mode.unwrap_or_else(|| "revisar".into());` (and clamp unknown values to `"revisar"`); pass to `runner::start(state.db.clone(), app, profile, mode, batch_size.unwrap_or(10))`.

- [ ] **Step 4: Run tests + build**

Run: `cd src-tauri && cargo test agent::prompt && cargo build`
Expected: PASS + clean build.

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "feat: scan/revisar run modes threaded into the agent prompt"
```

---

### Task 3: Review-queue + found-jobs queries and approve/reject commands

**Files:**
- Modify: `src-tauri/src/db/applications.rs` (review-queue query)
- Modify: `src-tauri/src/db/jobs.rs` (found-jobs query)
- Modify: `src-tauri/src/commands.rs` (commands)
- Modify: `src-tauri/src/lib.rs` (register)

**Interfaces:**
- `applications::review_queue(conn) -> rusqlite::Result<Vec<ReviewItem>>` where
  `ReviewItem { application_id: i64, job_title: String, company: String, url: String, cover_letter: String, answers_json: String }` (Serialize) — joins job + application where `status='awaiting_approval'`, newest first.
- `applications::set_status` already exists — `approve`/`reject` use it with `"approved"`/`"discarded"`.
- `jobs::without_application(conn) -> rusqlite::Result<Vec<Job>>` — jobs that have no application row (Scan discoveries), newest first.
- Commands: `list_review_queue() -> Vec<ReviewItem>`, `list_found_jobs() -> Vec<Job>`, `approve_application(id)`, `reject_application(id)`.

- [ ] **Step 1: Review-queue query + test**

In `src-tauri/src/db/applications.rs`:

```rust
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReviewItem {
    pub application_id: i64,
    pub job_title: String,
    pub company: String,
    pub url: String,
    pub cover_letter: String,
    pub answers_json: String,
}

pub fn review_queue(conn: &Connection) -> rusqlite::Result<Vec<ReviewItem>> {
    let mut stmt = conn.prepare(
        "SELECT a.id, j.title, j.company, j.url, \
                COALESCE(a.cover_letter,''), COALESCE(a.answers_json,'[]') \
         FROM applications a JOIN jobs j ON a.job_id = j.id \
         WHERE a.status = 'awaiting_approval' ORDER BY a.id DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(ReviewItem {
            application_id: r.get(0)?,
            job_title: r.get(1)?,
            company: r.get(2)?,
            url: r.get(3)?,
            cover_letter: r.get(4)?,
            answers_json: r.get(5)?,
        })
    })?;
    rows.collect()
}
```

Add a test:
```rust
    #[test]
    fn review_queue_returns_awaiting_items_with_content() {
        let conn = open_in_memory();
        let job_id = jobs::insert(&conn, &jobs::NewJob {
            title: "Dev".into(), company: "Acme".into(),
            url: "https://linkedin.com/jobs/1".into(), source: "linkedin".into(),
        }).unwrap();
        create_with_content(&conn, job_id, "Dear Acme", r#"[{"question":"Q","answer":"A"}]"#).unwrap();
        let q = review_queue(&conn).unwrap();
        assert_eq!(q.len(), 1);
        assert_eq!(q[0].company, "Acme");
        assert_eq!(q[0].cover_letter, "Dear Acme");
    }
```
(Import `jobs` in the test module as needed.)

- [ ] **Step 2: Found-jobs query + test**

In `src-tauri/src/db/jobs.rs`:
```rust
/// Jobs that have no application row yet (Scan-mode discoveries).
pub fn without_application(conn: &Connection) -> rusqlite::Result<Vec<Job>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, company, url, source, status, match_summary, discovered_at \
         FROM jobs WHERE id NOT IN (SELECT job_id FROM applications) ORDER BY id DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(Job {
            id: r.get(0)?, title: r.get(1)?, company: r.get(2)?, url: r.get(3)?,
            source: r.get(4)?, status: r.get(5)?, match_summary: r.get(6)?, discovered_at: r.get(7)?,
        })
    })?;
    rows.collect()
}
```
Add a test:
```rust
    #[test]
    fn without_application_excludes_jobs_that_have_one() {
        let conn = crate::db::open_in_memory();
        let a = insert(&conn, &NewJob { title:"A".into(), company:"X".into(), url:"u1".into(), source:"linkedin".into() }).unwrap();
        let _b = insert(&conn, &NewJob { title:"B".into(), company:"Y".into(), url:"u2".into(), source:"linkedin".into() }).unwrap();
        crate::db::applications::create_with_content(&conn, a, "cl", "[]").unwrap();
        let found = without_application(&conn).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].url, "u2");
    }
```

- [ ] **Step 3: Commands**

In `src-tauri/src/commands.rs`:
```rust
#[tauri::command]
pub fn list_review_queue(state: State<AppState>) -> CmdResult<Vec<applications::ReviewItem>> {
    let conn = state.db.lock().map_err(err)?;
    applications::review_queue(&conn).map_err(err)
}

#[tauri::command]
pub fn list_found_jobs(state: State<AppState>) -> CmdResult<Vec<jobs::Job>> {
    let conn = state.db.lock().map_err(err)?;
    jobs::without_application(&conn).map_err(err)
}

#[tauri::command]
pub fn approve_application(state: State<AppState>, id: i64) -> CmdResult<()> {
    let conn = state.db.lock().map_err(err)?;
    applications::set_status(&conn, id, "approved").map_err(err)
}

#[tauri::command]
pub fn reject_application(state: State<AppState>, id: i64) -> CmdResult<()> {
    let conn = state.db.lock().map_err(err)?;
    applications::set_status(&conn, id, "discarded").map_err(err)
}
```
Register `list_review_queue`, `list_found_jobs`, `approve_application`, `reject_application` in `lib.rs`'s `generate_handler!`.

- [ ] **Step 4: Run tests + build**

Run: `cd src-tauri && cargo test db:: && cargo build`
Expected: PASS + clean build.

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "feat: review-queue/found-jobs queries and approve/reject commands"
```

---

### Task 4: Dashboard mode selector

**Files:**
- Modify: `src/lib/api.ts` (`startSearchBatch` takes mode)
- Modify: `src/screens/Dashboard.tsx`

**Interfaces:**
- `api.startSearchBatch(mode: string, batchSize: number)`.

- [ ] **Step 1: API**

In `src/lib/api.ts` change:
```ts
  startSearchBatch: (mode: string, batchSize: number) =>
    invoke<void>("start_search_batch", { mode, batchSize }),
```

- [ ] **Step 2: Dashboard mode dropdown**

In `src/screens/Dashboard.tsx`, add a `mode` state (`const [mode, setMode] = useState<"scan" | "revisar">("revisar");`) and a `<select>` before the batch input:
```tsx
      <label>Modo
        <select value={mode} onChange={(e) => setMode(e.target.value as "scan" | "revisar")} disabled={running} style={{ marginLeft: 8 }}>
          <option value="revisar">Revisar (preparar p/ aprovar)</option>
          <option value="scan">Scan (só descobrir)</option>
        </select>
      </label>
```
Change `start()` to `await api.startSearchBatch(mode, batch);`.

- [ ] **Step 3: Build**

Run: `npm run build` → tsc + vite success.

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "feat: dashboard mode selector (scan/revisar)"
```

---

### Task 5: Vagas screen — found jobs + review/approve

**Files:**
- Modify: `src/types.ts` (`ReviewItem`, `Answer`)
- Modify: `src/lib/api.ts` (review/found/approve/reject)
- Modify: `src/screens/Jobs.tsx` (replace stub)

**Interfaces:**
- `ReviewItem { application_id, job_title, company, url, cover_letter, answers_json }`.
- `api.listReviewQueue()`, `api.listFoundJobs()`, `api.approveApplication(id)`, `api.rejectApplication(id)`.

- [ ] **Step 1: Types + API**

In `src/types.ts`:
```ts
export interface ReviewItem {
  application_id: number;
  job_title: string;
  company: string;
  url: string;
  cover_letter: string;
  answers_json: string;
}
```
In `src/lib/api.ts` add to `api`:
```ts
  listReviewQueue: () => invoke<ReviewItem[]>("list_review_queue"),
  listFoundJobs: () => invoke<Job[]>("list_found_jobs"),
  approveApplication: (id: number) => invoke<void>("approve_application", { id }),
  rejectApplication: (id: number) => invoke<void>("reject_application", { id }),
```
(import `ReviewItem` and `Job` types.)

- [ ] **Step 2: Implement the Vagas screen**

Replace `src/screens/Jobs.tsx`:
```tsx
import { useEffect, useState } from "react";
import { api } from "../lib/api";
import type { Job, ReviewItem } from "../types";

export default function Jobs() {
  const [review, setReview] = useState<ReviewItem[]>([]);
  const [found, setFound] = useState<Job[]>([]);

  async function refresh() {
    setReview(await api.listReviewQueue());
    setFound(await api.listFoundJobs());
  }
  useEffect(() => { refresh(); }, []);

  async function approve(id: number) { await api.approveApplication(id); await refresh(); }
  async function reject(id: number) { await api.rejectApplication(id); await refresh(); }

  function answers(json: string): { question: string; answer: string }[] {
    try { return JSON.parse(json); } catch { return []; }
  }

  return (
    <section>
      <h1>Vagas</h1>

      <h2>Aguardando aprovação ({review.length})</h2>
      {review.length === 0 && <p className="hint">Nada para revisar agora.</p>}
      {review.map((r) => (
        <div key={r.application_id} style={{ border: "1px solid #ddd", borderRadius: 8, padding: 12, margin: "12px 0" }}>
          <strong>{r.job_title}</strong> — {r.company}{" "}
          <a href={r.url} target="_blank" rel="noreferrer">ver vaga</a>
          <details style={{ margin: "8px 0" }}>
            <summary>Carta de apresentação</summary>
            <pre style={{ whiteSpace: "pre-wrap", fontFamily: "inherit" }}>{r.cover_letter}</pre>
          </details>
          {answers(r.answers_json).length > 0 && (
            <details>
              <summary>Respostas ({answers(r.answers_json).length})</summary>
              <ul>{answers(r.answers_json).map((a, i) => <li key={i}><b>{a.question}</b> — {a.answer}</li>)}</ul>
            </details>
          )}
          <div style={{ display: "flex", gap: 8, marginTop: 8 }}>
            <button onClick={() => approve(r.application_id)}>Aprovar</button>
            <button onClick={() => reject(r.application_id)}>Rejeitar</button>
          </div>
        </div>
      ))}

      <h2 style={{ marginTop: 24 }}>Encontradas — Scan ({found.length})</h2>
      {found.length === 0 && <p className="hint">Nenhuma vaga só-descoberta.</p>}
      <ul>
        {found.map((j) => (
          <li key={j.id}>
            <a href={j.url} target="_blank" rel="noreferrer">{j.title}</a> — {j.company}
            {j.match_summary ? ` · ${j.match_summary}` : ""}
          </li>
        ))}
      </ul>
    </section>
  );
}
```

- [ ] **Step 3: Build**

Run: `npm run build` → tsc + vite success.

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "feat: vagas screen with review/approve and scan discoveries"
```

---

### Task 6: Manual validation

- [ ] **Step 1: Verify both modes + approval**

Run `npm run tauri dev` (logged into LinkedIn, Claude-in-Chrome connected). Then:
1. **Painel** → Modo = **Scan**, batch = 2 → **Iniciar**. After it finishes, **Vagas** → "Encontradas — Scan" lists the jobs (with links), and "Aguardando aprovação" stays empty.
2. **Painel** → Modo = **Revisar**, batch = 2 → **Iniciar**. After it finishes, **Vagas** → "Aguardando aprovação" lists applications; expand "Carta de apresentação" to read the generated letter and "Respostas".
3. Click **Aprovar** on one (it disappears from the queue — status `approved`; submission is Plan 6) and **Rejeitar** on another.

Record what happened (counts, whether the cover letter looked right). This step gates the plan.

---

## Plan 5 Self-Review

- **Spec coverage:** Scan mode discovers jobs only (Tasks 1, 2) ✓; Revisar prepares applications (Task 2, unchanged sink path) ✓; mode selector on Dashboard (Task 4) ✓; Vagas screen shows the generated cover letter + answers with Aprovar/Rejeitar (Task 5) ✓; found-jobs list for Scan (Tasks 3, 5) ✓; pt-BR UI / English code ✓; agent still never submits (prompt MODE blocks both say "do not submit" / "discovery only") ✓. Deferred to Plan 6: submission + Auto mode.
- **Placeholder scan:** No TBD. All code blocks complete.
- **Type consistency:** `ReviewItem` matches across Rust (`applications.rs`) and TS (`types.ts`); `answers_json` is a string parsed in the UI into `{question,answer}` (matches `protocol::Answer` / the stored `answers_json`). Mode strings `"scan"`/`"revisar"` consistent across Dashboard, command default, and `mode_instructions`. New commands match the `api` client. `build_system_prompt`'s new `mode` param threaded from `runner::start` ← `start_search_batch`.

## Hand-off to Plan 6

Plan 6 adds **submission** and the **Auto** mode: an agent run that takes the `approved` applications (and, in Auto, prepares+submits inline), navigates each LinkedIn Easy Apply, fills it with the prepared/answer-bank answers, submits, and reports `APPLYBOT_SUBMITTED {application_id}` → status `submitted` (or a pending on a blocker). Adds a third Dashboard mode (Auto) and a "Enviar aprovadas" action. Likely needs live prompt tuning like Plan 3.
