# applybot — Plan (Edit before approve)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let the user edit a prepared application's cover letter AND its screening answers in the Vagas review queue before approving, so what gets sent is exactly what they want.

**Architecture:** A new `update_application_content(id, cover_letter, answers_json)` command persists edits to the `applications` row. The Vagas "Aguardando aprovação" items become an editable `ReviewCard` (cover-letter textarea + one input per answer); Salvar persists, Aprovar persists-then-approves.

**Tech Stack:** Tauri v2, Rust, React 19 + TS. No new deps.

## Global Constraints

- Code/identifiers/comments in **English**; UI strings in **pt-BR**.
- SQLite single source of truth; only Rust writes. Edits persist immediately on Salvar/Aprovar.
- `answers_json` stays a JSON array `[{question, answer}]`; only the `answer` values are user-editable (questions are from LinkedIn).
- Reuse: `db::applications`, the `api` client, the Vagas screen (`Jobs.tsx`).
- Conventional Commits.

---

### Task 1: Backend — persist edited content

**Files:**
- Modify: `src-tauri/src/db/applications.rs`
- Modify: `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`

**Interfaces:**
- `applications::update_content(conn, id: i64, cover_letter: &str, answers_json: &str) -> rusqlite::Result<()>`
- command `update_application_content(id: i64, cover_letter: String, answers_json: String) -> ()`

- [ ] **Step 1: Store fn + test**

In `src-tauri/src/db/applications.rs`:
```rust
/// Overwrite the generated content of an application (user edits before approval).
pub fn update_content(
    conn: &Connection,
    id: i64,
    cover_letter: &str,
    answers_json: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE applications SET cover_letter = ?2, answers_json = ?3 WHERE id = ?1",
        (id, cover_letter, answers_json),
    )?;
    Ok(())
}
```
Test:
```rust
    #[test]
    fn update_content_overwrites_letter_and_answers() {
        let conn = open_in_memory();
        let j = jobs::insert(&conn, &jobs::NewJob{title:"D".into(),company:"A".into(),url:"u1".into(),source:"linkedin".into()}).unwrap();
        let id = create_with_content(&conn, j, "old", "[]").unwrap();
        update_content(&conn, id, "new letter", r#"[{"question":"Q","answer":"A"}]"#).unwrap();
        let q = review_queue(&conn).unwrap();
        assert_eq!(q[0].cover_letter, "new letter");
        assert!(q[0].answers_json.contains("\"answer\":\"A\""));
    }
```

- [ ] **Step 2: Command**

In `src-tauri/src/commands.rs`:
```rust
#[tauri::command]
pub fn update_application_content(
    state: State<AppState>,
    id: i64,
    cover_letter: String,
    answers_json: String,
) -> CmdResult<()> {
    let conn = state.db.lock().map_err(err)?;
    applications::update_content(&conn, id, &cover_letter, &answers_json).map_err(err)
}
```
Register `update_application_content` in `lib.rs`'s `generate_handler!`.

- [ ] **Step 3: Tests + build**

Run: `cd src-tauri && cargo test db::applications && cargo build`
Expected: PASS + clean.

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "feat: update_application_content command to persist edits"
```

---

### Task 2: Vagas — editable ReviewCard

**Files:**
- Modify: `src/lib/api.ts`
- Modify: `src/screens/Jobs.tsx`

**Interfaces:**
- `api.updateApplicationContent(id, coverLetter, answersJson)`.

- [ ] **Step 1: API**

In `src/lib/api.ts` add to `api`:
```ts
  updateApplicationContent: (id: number, coverLetter: string, answersJson: string) =>
    invoke<void>("update_application_content", { id, coverLetter, answersJson }),
```

- [ ] **Step 2: Editable review card**

In `src/screens/Jobs.tsx`, replace the inline rendering of each review item with a `ReviewCard` component that holds local edit state. Add at the bottom of the file:
```tsx
function ReviewCard({
  item, onApprove, onReject,
}: {
  item: ReviewItem;
  onApprove: () => void;
  onReject: () => void;
}) {
  const initialAnswers = (() => {
    try { return JSON.parse(item.answers_json) as { question: string; answer: string }[]; }
    catch { return []; }
  })();
  const [letter, setLetter] = useState(item.cover_letter);
  const [answers, setAnswers] = useState(initialAnswers);
  const [status, setStatus] = useState<string | null>(null);

  function setAnswer(i: number, v: string) {
    setAnswers((a) => a.map((x, idx) => (idx === i ? { ...x, answer: v } : x)));
  }
  async function save() {
    setStatus(null);
    try {
      await api.updateApplicationContent(item.application_id, letter, JSON.stringify(answers));
      setStatus("Edições salvas.");
    } catch (e) { setStatus(`Erro: ${e}`); }
  }
  async function approve() {
    try {
      await api.updateApplicationContent(item.application_id, letter, JSON.stringify(answers));
      onApprove();
    } catch (e) { setStatus(`Erro: ${e}`); }
  }

  return (
    <div className="card">
      <strong>{item.job_title}</strong> — {item.company}{" "}
      <a href={item.url} onClick={(e) => openExternal(e, item.url)}>ver vaga</a>
      <label className="field" style={{ marginTop: 12 }}>
        Carta de apresentação
        <textarea rows={10} value={letter} onChange={(e) => setLetter(e.target.value)} />
      </label>
      {answers.length > 0 && (
        <div>
          <div className="hint" style={{ marginBottom: 8 }}>Respostas</div>
          {answers.map((a, i) => (
            <label className="field" key={i}>
              {a.question}
              <input value={a.answer} onChange={(e) => setAnswer(i, e.target.value)} />
            </label>
          ))}
        </div>
      )}
      <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
        <button className="btn btn-primary" onClick={approve}>Aprovar</button>
        <button className="btn btn-ghost" onClick={onReject}>Rejeitar</button>
        <button className="btn" onClick={save}>Salvar edição</button>
        {status && <span className="hint">{status}</span>}
      </div>
    </div>
  );
}
```
In the `Jobs` component, render the review section with it:
```tsx
      {review.map((r) => (
        <ReviewCard
          key={r.application_id}
          item={r}
          onApprove={() => approve(r.application_id)}
          onReject={() => reject(r.application_id)}
        />
      ))}
```
Ensure `useState` is imported and `openExternal` (already defined in this file) is in scope for `ReviewCard` (it is, same module). Keep the existing `approve`/`reject` handlers (they call the commands then `refresh()`); the old inline `<details>`/buttons for review items are removed in favor of `ReviewCard`. Leave the "Aprovadas" and "Encontradas — Scan" sections unchanged.

- [ ] **Step 3: Build**

Run: `npm run build` → tsc 0 errors + vite success.

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "feat: edit cover letter and answers in vagas before approving"
```

---

### Task 3: Manual validation

- [ ] **Step 1**

Run `npm run tauri dev`. With an application in "Aguardando aprovação": edit the cover letter text and an answer, click **Salvar edição** ("Edições salvas."), navigate away and back → edits persist. Then **Aprovar** → it moves to Aprovadas with the edited content (verify via the Aprovadas section or by re-opening). Record results. Gates the plan.

---

## Plan Self-Review

- **Spec coverage:** edit cover letter (Task 2 textarea) ✓; edit screening answers (Task 2 per-answer inputs) ✓; persist before approving (Salvar + Aprovar-saves-then-approves, Tasks 1-2) ✓; pt-BR UI / English code ✓.
- **Placeholder scan:** none.
- **Type consistency:** `update_application_content(id, coverLetter, answersJson)` ↔ Rust `(id, cover_letter, answers_json)` via Tauri camelCase mapping; `ReviewItem.answers_json` parsed to `{question,answer}[]` and re-serialized on save; `ReviewCard` uses the existing `openExternal` + `api` + `useState`.

## Hand-off

After this, resume the Plan 6 live submission validation (now able to fine-tune each letter/answer before approving): edit → Aprovar → Enviar aprovadas → confirm real submission.
