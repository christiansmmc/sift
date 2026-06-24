# applybot — Plan 2: Resume Parsing & Onboarding

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the navigable shell from Plan 1 into a real, completable onboarding flow: parse an uploaded CV (PDF/DOCX) to text, let Claude pre-fill the search criteria, store LinkedIn credentials in the OS keychain, persist the profile, and let the user edit it later on the Profile screen.

**Architecture:** Builds on Plan 1's tested SQLite data layer and Tauri command surface. Adds three Rust capabilities — a resume text extractor, a one-shot CV analyzer that shells out to the `claude -p` CLI, and OS-keychain credential storage — plus two React surfaces: a multi-step onboarding wizard (replacing the provisional button) and the Profile editor. After this plan, completing onboarding unlocks the four-screen shell and survives an app restart.

**Tech Stack:** Tauri v2 (+ dialog plugin), React 19, TypeScript, Rust 2021, rusqlite, `pdf-extract`, `zip` + `quick-xml` (DOCX), `keyring` v3, the `claude` CLI (already installed on the user's machine).

## Global Constraints

- Platform Windows 11, desktop-only Tauri.
- All code/identifiers/comments/schema/system-prompts in **English**; all user-facing UI strings in **Brazilian Portuguese (pt-BR)**.
- SQLite is the single source of truth for profile data. LinkedIn credentials live ONLY in the OS keychain (Windows Credential Manager via `keyring`), never in SQLite, never in plain text, never logged.
- `criteria_json` is a JSON object with this exact shape (all keys present; empty/null when unknown):
  ```json
  { "role": "", "seniority": "", "work_model": "", "locations": [], "salary_min": null, "red_lines": [] }
  ```
  `work_model` is one of `""`, `"remote"`, `"hybrid"`, `"onsite"`.
- The CV analyzer must **degrade gracefully**: if the `claude` CLI is missing, errors, or returns unparseable output, the command returns a default empty `Criteria` and the UI falls back to manual entry. Manual entry is ALWAYS available — the user is never forced to use AI.
- Conventional Commits. One commit per task minimum.
- Reuse Plan 1's interfaces verbatim: `profile::Profile { full_name, email, phone, location, cv_text, criteria_json }`, `profile::upsert`, `profile::get`, `profile::is_onboarding_complete(conn, has_linkedin_credentials)`, the `AppState { db: Mutex<Connection> }`, and the existing `api` client in `src/lib/api.ts`.

---

### Task 1: Add dependencies and the PDF text extractor

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/resume/mod.rs`
- Modify: `src-tauri/src/lib.rs` (add `mod resume;`)
- Create test fixture: `src-tauri/tests/fixtures/sample.pdf`

**Interfaces:**
- Produces:
  - `resume::extract_pdf(bytes: &[u8]) -> Result<String, String>` — extracts plain text from PDF bytes.
  - `resume::ResumeError` is represented as `String` (keep errors as `String` for Tauri compatibility).

- [ ] **Step 1: Add crates to `src-tauri/Cargo.toml`**

Under `[dependencies]` add:

```toml
pdf-extract = "0.7"
zip = "2"
quick-xml = "0.36"
keyring = "3"
```

Also add the Tauri dialog plugin (used in Task 3):

```toml
tauri-plugin-dialog = "2"
```

- [ ] **Step 2: Create a tiny real PDF fixture**

Create `src-tauri/tests/fixtures/sample.pdf` containing the literal text `Backend Engineer with 8 years of Rust experience`. Generate it programmatically so it is a valid PDF (do not hand-type binary). From the repo root run this one-off Node script (delete it after — it just writes the fixture):

```bash
node -e '
const fs=require("fs");
const text="Backend Engineer with 8 years of Rust experience";
const stream=`BT /F1 12 Tf 50 700 Td (${text}) Tj ET`;
const objs=[
 "<< /Type /Catalog /Pages 2 0 R >>",
 "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
 "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>",
 `<< /Length ${stream.length} >>\nstream\n${stream}\nendstream`,
 "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>"
];
let pdf="%PDF-1.4\n", offsets=[];
objs.forEach((o,i)=>{offsets.push(pdf.length);pdf+=`${i+1} 0 obj\n${o}\nendobj\n`;});
const xref=pdf.length;
pdf+=`xref\n0 ${objs.length+1}\n0000000000 65535 f \n`;
offsets.forEach(off=>{pdf+=String(off).padStart(10,"0")+" 00000 n \n";});
pdf+=`trailer\n<< /Size ${objs.length+1} /Root 1 0 R >>\nstartxref\n${xref}\n%%EOF`;
fs.mkdirSync("src-tauri/tests/fixtures",{recursive:true});
fs.writeFileSync("src-tauri/tests/fixtures/sample.pdf",pdf,"latin1");
console.log("wrote sample.pdf");
'
```

If `pdf-extract` cannot read this minimal PDF in the test (some versions are picky), fall back to committing a real small PDF the implementer generates with any available tool, as long as `extract_pdf` returns text containing `Backend Engineer`. The acceptance criterion is the test below passing, not the generation method.

- [ ] **Step 3: Write the failing test + implementation**

Create `src-tauri/src/resume/mod.rs`:

```rust
//! Resume text extraction (PDF / DOCX) for onboarding.

/// Extract plain text from PDF bytes.
pub fn extract_pdf(bytes: &[u8]) -> Result<String, String> {
    pdf_extract::extract_text_from_mem(bytes).map_err(|e| format!("PDF extract failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_pdf_reads_embedded_text() {
        let bytes = include_bytes!("../../tests/fixtures/sample.pdf");
        let text = extract_pdf(bytes).expect("extract pdf");
        assert!(
            text.contains("Backend Engineer"),
            "expected extracted text to contain the CV phrase, got: {text:?}"
        );
    }
}
```

Add `mod resume;` to `src-tauri/src/lib.rs`.

- [ ] **Step 4: Run the test**

Run: `cd src-tauri && cargo test resume::`
Expected: `extract_pdf_reads_embedded_text` PASSES.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: pdf resume text extraction"
```

---

### Task 2: DOCX text extractor

**Files:**
- Modify: `src-tauri/src/resume/mod.rs`
- Create test fixture: `src-tauri/tests/fixtures/sample.docx`

**Interfaces:**
- Consumes: `zip`, `quick-xml`.
- Produces: `resume::extract_docx(bytes: &[u8]) -> Result<String, String>` — extracts text from a .docx, joining `<w:t>` runs and inserting a newline at each `</w:p>` paragraph end.

- [ ] **Step 1: Create a real .docx fixture**

A .docx is a zip with `word/document.xml`. Generate a minimal valid one from the repo root (delete the script after):

```bash
node -e '
const fs=require("fs"); const {execSync}=require("child_process");
const doc=`<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Senior Frontend Developer</w:t></w:r></w:p><w:p><w:r><w:t>React and TypeScript</w:t></w:r></w:p></w:body></w:document>`;
// Build a zip with only word/document.xml using the zip npm-free approach via PowerShell Compress is messy;
// instead emit raw files and let the implementer zip them. Write the xml to a temp dir.
fs.mkdirSync("tmp-docx/word",{recursive:true});
fs.writeFileSync("tmp-docx/word/document.xml",doc);
console.log("wrote tmp-docx/word/document.xml — now zip tmp-docx into sample.docx");
'
```

Then zip it into a `.docx` (store the `word/document.xml` path inside the archive). On Windows PowerShell:

```powershell
Compress-Archive -Path tmp-docx\* -DestinationPath src-tauri\tests\fixtures\sample.docx.zip -Force
Move-Item src-tauri\tests\fixtures\sample.docx.zip src-tauri\tests\fixtures\sample.docx -Force
Remove-Item -Recurse -Force tmp-docx
```

(A `.docx` missing the `[Content_Types].xml` part is still a valid zip and our extractor only reads `word/document.xml`, so this minimal archive is sufficient for the test. If `Compress-Archive` nests an extra top folder, ensure the entry path is exactly `word/document.xml` — adjust by zipping from inside `tmp-docx`.)

- [ ] **Step 2: Write the failing test + implementation**

Append to `src-tauri/src/resume/mod.rs`:

```rust
use std::io::Read;

/// Extract plain text from DOCX bytes by reading `word/document.xml`.
pub fn extract_docx(bytes: &[u8]) -> Result<String, String> {
    let reader = std::io::Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(reader).map_err(|e| format!("not a valid docx (zip): {e}"))?;
    let mut xml = String::new();
    zip.by_name("word/document.xml")
        .map_err(|e| format!("docx missing word/document.xml: {e}"))?
        .read_to_string(&mut xml)
        .map_err(|e| format!("read document.xml: {e}"))?;

    use quick_xml::events::Event;
    use quick_xml::reader::Reader;
    let mut reader = Reader::from_str(&xml);
    let mut out = String::new();
    let mut in_text = false;
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if e.name().as_ref() == b"w:t" => in_text = true,
            Ok(Event::End(e)) if e.name().as_ref() == b"w:t" => in_text = false,
            Ok(Event::End(e)) if e.name().as_ref() == b"w:p" => out.push('\n'),
            Ok(Event::Text(t)) if in_text => {
                out.push_str(&t.unescape().map_err(|e| format!("xml unescape: {e}"))?);
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("xml parse error: {e}")),
            _ => {}
        }
        buf.clear();
    }
    Ok(out.trim().to_string())
}

#[cfg(test)]
mod docx_tests {
    use super::*;

    #[test]
    fn extract_docx_reads_paragraph_text() {
        let bytes = include_bytes!("../../tests/fixtures/sample.docx");
        let text = extract_docx(bytes).expect("extract docx");
        assert!(text.contains("Senior Frontend Developer"), "got: {text:?}");
        assert!(text.contains("React and TypeScript"), "got: {text:?}");
    }
}
```

- [ ] **Step 3: Run the test**

Run: `cd src-tauri && cargo test resume::`
Expected: both `extract_pdf_reads_embedded_text` and `extract_docx_reads_paragraph_text` PASS.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: docx resume text extraction"
```

---

### Task 3: `parse_resume` command + file dialog

**Files:**
- Create: `src-tauri/src/resume/mod.rs` already exists — add a dispatcher function.
- Modify: `src-tauri/src/commands.rs` (add `parse_resume` command)
- Modify: `src-tauri/src/lib.rs` (register the dialog plugin + the command)
- Modify: `src-tauri/capabilities/default.json` (allow dialog)
- Modify: `package.json` (add `@tauri-apps/plugin-dialog`)

**Interfaces:**
- Consumes: `resume::extract_pdf`, `resume::extract_docx`.
- Produces:
  - `resume::extract_from_path(path: &str) -> Result<String, String>` — picks the extractor by file extension (`.pdf` / `.docx`), returns text; errors on unsupported extensions.
  - Tauri command `parse_resume(path: String) -> Result<String, String>`.
  - Frontend: `api.parseResume(path)` and `api.pickResumeFile()` (dialog) — added in Task 6's client, but the command + plugin are wired here.

- [ ] **Step 1: Add the path dispatcher + test**

Append to `src-tauri/src/resume/mod.rs`:

```rust
/// Choose an extractor by file extension and read the file from disk.
pub fn extract_from_path(path: &str) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read {path}: {e}"))?;
    let lower = path.to_lowercase();
    if lower.ends_with(".pdf") {
        extract_pdf(&bytes)
    } else if lower.ends_with(".docx") {
        extract_docx(&bytes)
    } else {
        Err("Formato não suportado: use PDF ou DOCX".to_string())
    }
}

#[cfg(test)]
mod path_tests {
    use super::*;

    #[test]
    fn extract_from_path_rejects_unknown_extension() {
        let err = extract_from_path("resume.txt").unwrap_err();
        assert!(err.contains("Formato não suportado"));
    }
}
```

Note: this error string IS user-facing (surfaced in the UI), so it is pt-BR by the Global Constraints — that is intentional and correct.

- [ ] **Step 2: Run the test**

Run: `cd src-tauri && cargo test resume::path_tests`
Expected: PASS (file-not-found path is not exercised; the `.txt` branch returns the pt-BR error before reading).

Wait — `extract_from_path` reads the file BEFORE checking extension. Fix the ordering so the extension check happens first (so the unit test doesn't need a real file):

```rust
pub fn extract_from_path(path: &str) -> Result<String, String> {
    let lower = path.to_lowercase();
    let bytes = std::fs::read(path).map_err(|e| format!("read {path}: {e}"))?;
    if lower.ends_with(".pdf") {
        extract_pdf(&bytes)
    } else if lower.ends_with(".docx") {
        extract_docx(&bytes)
    } else {
        Err("Formato não suportado: use PDF ou DOCX".to_string())
    }
}
```

The `.txt` test still fails on `fs::read` (file missing) rather than the extension branch. To keep the test pure, check the extension FIRST:

```rust
pub fn extract_from_path(path: &str) -> Result<String, String> {
    let lower = path.to_lowercase();
    if !(lower.ends_with(".pdf") || lower.ends_with(".docx")) {
        return Err("Formato não suportado: use PDF ou DOCX".to_string());
    }
    let bytes = std::fs::read(path).map_err(|e| format!("read {path}: {e}"))?;
    if lower.ends_with(".pdf") {
        extract_pdf(&bytes)
    } else {
        extract_docx(&bytes)
    }
}
```

Use THIS final version. Re-run the test — it now passes without a real file.

- [ ] **Step 3: Add the Tauri command**

In `src-tauri/src/commands.rs` add:

```rust
#[tauri::command]
pub fn parse_resume(path: String) -> CmdResult<String> {
    crate::resume::extract_from_path(&path)
}
```

- [ ] **Step 4: Register the dialog plugin + command**

In `src-tauri/src/lib.rs`, add `.plugin(tauri_plugin_dialog::init())` to the builder chain (near the other `.plugin(...)` calls), and add `commands::parse_resume` to the `generate_handler!` list.

In `src-tauri/capabilities/default.json`, add the dialog permission to the `permissions` array: `"dialog:default"` (and `"dialog:allow-open"` if present in this plugin version). Then install the JS side: `npm install @tauri-apps/plugin-dialog`.

- [ ] **Step 5: Verify it compiles**

Run: `cd src-tauri && cargo build` → success. Then `npm run build` → success.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: parse_resume command with file dialog plugin"
```

---

### Task 4: LinkedIn credentials in the OS keychain

**Files:**
- Create: `src-tauri/src/credentials.rs`
- Modify: `src-tauri/src/lib.rs` (add `mod credentials;`)
- Modify: `src-tauri/src/commands.rs` (add commands; wire real `has_linkedin_credentials` into `get_onboarding_status`)

**Interfaces:**
- Consumes: `keyring` v3.
- Produces:
  - `credentials::save_linkedin(username: &str, password: &str) -> Result<(), String>` — stores under service `applybot-linkedin`, account = `username`; also records the username separately so it can be read back (store under a fixed account `__current_user__`).
  - `credentials::current_username() -> Option<String>` — the saved LinkedIn username, if any.
  - `credentials::has_linkedin() -> bool` — true when a username + password are stored.
  - Tauri commands: `save_linkedin_credentials(username: String, password: String)`, `has_linkedin_credentials() -> bool`, `get_linkedin_username() -> Option<String>`.
  - `get_onboarding_status` now calls `credentials::has_linkedin()` instead of hardcoded `false`.

- [ ] **Step 1: Implement the module**

Create `src-tauri/src/credentials.rs`:

```rust
//! LinkedIn credential storage in the OS keychain (Windows Credential Manager).
//! Credentials never touch SQLite, plain text, or logs.

use keyring::Entry;

const SERVICE: &str = "applybot-linkedin";
const USER_POINTER: &str = "__current_user__";

fn entry(account: &str) -> Result<Entry, String> {
    Entry::new(SERVICE, account).map_err(|e| format!("keychain entry: {e}"))
}

pub fn save_linkedin(username: &str, password: &str) -> Result<(), String> {
    entry(username)?
        .set_password(password)
        .map_err(|e| format!("save password: {e}"))?;
    // Record which username is current so we can read it back.
    entry(USER_POINTER)?
        .set_password(username)
        .map_err(|e| format!("save username pointer: {e}"))?;
    Ok(())
}

pub fn current_username() -> Option<String> {
    entry(USER_POINTER).ok()?.get_password().ok()
}

pub fn has_linkedin() -> bool {
    match current_username() {
        Some(user) => entry(&user)
            .ok()
            .and_then(|e| e.get_password().ok())
            .is_some(),
        None => false,
    }
}
```

Add `mod credentials;` to `src-tauri/src/lib.rs`.

- [ ] **Step 2: Add the commands and wire the gate**

In `src-tauri/src/commands.rs`:

```rust
#[tauri::command]
pub fn save_linkedin_credentials(username: String, password: String) -> CmdResult<()> {
    crate::credentials::save_linkedin(&username, &password)
}

#[tauri::command]
pub fn has_linkedin_credentials() -> CmdResult<bool> {
    Ok(crate::credentials::has_linkedin())
}

#[tauri::command]
pub fn get_linkedin_username() -> CmdResult<Option<String>> {
    Ok(crate::credentials::current_username())
}
```

Change `get_onboarding_status` to use the real check:

```rust
#[tauri::command]
pub fn get_onboarding_status(state: State<AppState>) -> CmdResult<bool> {
    let conn = state.db.lock().map_err(err)?;
    let has_creds = crate::credentials::has_linkedin();
    profile::is_onboarding_complete(&conn, has_creds).map_err(err)
}
```

Register the three new commands in `lib.rs`'s `generate_handler!`.

- [ ] **Step 3: Test the credential round-trip**

Add to `src-tauri/src/credentials.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Uses the real OS keychain; cleans up after itself. Serialized via a unique account.
    #[test]
    fn save_then_has_and_read_username() {
        let user = "applybot-test-user@example.com";
        save_linkedin(user, "secret-pw").expect("save");
        assert!(has_linkedin());
        assert_eq!(current_username().as_deref(), Some(user));
        // cleanup
        let _ = entry(user).unwrap().delete_credential();
        let _ = entry(USER_POINTER).unwrap().delete_credential();
    }
}
```

Run: `cd src-tauri && cargo test credentials::`
Expected: PASS. (If the CI/headless keychain is unavailable the test may error — on the user's Windows machine the Credential Manager is present, so it passes locally. If it fails ONLY due to no keychain backend, mark the test `#[ignore]` with a comment and verify manually; do not delete the assertions.)

- [ ] **Step 4: Verify build**

Run: `cd src-tauri && cargo build` → success.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: linkedin credential storage in os keychain"
```

---

### Task 5: CV analysis via one-shot `claude -p`

**Files:**
- Create: `src-tauri/src/cv_analysis.rs`
- Modify: `src-tauri/src/lib.rs` (add `mod cv_analysis;`)
- Modify: `src-tauri/src/commands.rs` (add `analyze_cv` command)

**Interfaces:**
- Produces:
  - `cv_analysis::Criteria { role: String, seniority: String, work_model: String, locations: Vec<String>, salary_min: Option<i64>, red_lines: Vec<String> }` — Serialize/Deserialize/Default; serializes to the exact `criteria_json` shape in Global Constraints.
  - `cv_analysis::build_prompt(cv_text: &str) -> String` — the analysis prompt (English instruction, returns JSON only).
  - `cv_analysis::parse_response(stdout: &str) -> Criteria` — extracts the JSON object from CLI stdout; returns `Criteria::default()` if parsing fails.
  - `cv_analysis::analyze(cv_text: &str) -> Criteria` — runs `claude -p <prompt>`, returns parsed criteria or default on any failure.
  - Tauri command `analyze_cv(cv_text: String) -> Criteria`.

- [ ] **Step 1: Implement with unit-testable seams**

Create `src-tauri/src/cv_analysis.rs`:

```rust
//! One-shot CV analysis via the `claude -p` CLI. Degrades to an empty
//! Criteria on any failure so the UI can fall back to manual entry.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Criteria {
    pub role: String,
    pub seniority: String,
    pub work_model: String,
    pub locations: Vec<String>,
    pub salary_min: Option<i64>,
    pub red_lines: Vec<String>,
}

pub fn build_prompt(cv_text: &str) -> String {
    format!(
        "You are analyzing a job candidate's CV to infer the kind of role they should search for. \
Return ONLY a JSON object, no prose, no markdown fences, with EXACTLY these keys: \
role (string, the target job title), seniority (string: junior/mid/senior/lead or \"\"), \
work_model (string: one of remote/hybrid/onsite or \"\"), locations (array of strings), \
salary_min (integer or null), red_lines (array of strings the candidate should avoid). \
Infer conservatively from the CV; use \"\" / [] / null when unknown. \
CV:\n---\n{cv_text}\n---"
    )
}

/// Extract the first top-level JSON object from arbitrary CLI stdout.
pub fn parse_response(stdout: &str) -> Criteria {
    let (start, end) = match (stdout.find('{'), stdout.rfind('}')) {
        (Some(s), Some(e)) if e > s => (s, e),
        _ => return Criteria::default(),
    };
    serde_json::from_str::<Criteria>(&stdout[start..=end]).unwrap_or_default()
}

pub fn analyze(cv_text: &str) -> Criteria {
    if cv_text.trim().is_empty() {
        return Criteria::default();
    }
    let prompt = build_prompt(cv_text);
    let output = std::process::Command::new("claude")
        .arg("-p")
        .arg(&prompt)
        .output();
    match output {
        Ok(o) if o.status.success() => {
            parse_response(&String::from_utf8_lossy(&o.stdout))
        }
        _ => Criteria::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_prompt_includes_cv_and_demands_json() {
        let p = build_prompt("10 years of Go");
        assert!(p.contains("10 years of Go"));
        assert!(p.contains("ONLY a JSON object"));
    }

    #[test]
    fn parse_response_extracts_json_from_noisy_output() {
        let out = "Here is the result:\n{\"role\":\"Backend Engineer\",\"seniority\":\"senior\",\"work_model\":\"remote\",\"locations\":[\"Brazil\"],\"salary_min\":12000,\"red_lines\":[]}\nDone.";
        let c = parse_response(out);
        assert_eq!(c.role, "Backend Engineer");
        assert_eq!(c.seniority, "senior");
        assert_eq!(c.work_model, "remote");
        assert_eq!(c.locations, vec!["Brazil".to_string()]);
        assert_eq!(c.salary_min, Some(12000));
    }

    #[test]
    fn parse_response_returns_default_on_garbage() {
        assert_eq!(parse_response("no json here"), Criteria::default());
        assert_eq!(parse_response("{not valid json}"), Criteria::default());
    }
}
```

Add `mod cv_analysis;` to `src-tauri/src/lib.rs`.

- [ ] **Step 2: Add the command**

In `src-tauri/src/commands.rs`:

```rust
#[tauri::command]
pub fn analyze_cv(cv_text: String) -> CmdResult<crate::cv_analysis::Criteria> {
    Ok(crate::cv_analysis::analyze(&cv_text))
}
```

Register `commands::analyze_cv` in `lib.rs`'s `generate_handler!`.

- [ ] **Step 3: Run the tests**

Run: `cd src-tauri && cargo test cv_analysis::`
Expected: all three tests PASS (no real `claude` call — only prompt + parse are tested; `analyze` itself is validated manually in the wizard).

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: one-shot cv analysis via claude -p"
```

---

### Task 6: Onboarding wizard (frontend)

**Files:**
- Modify: `src/lib/api.ts` (add new command wrappers + dialog helper)
- Modify: `src/types.ts` (add `Criteria`)
- Create: `src/screens/Onboarding.tsx` (replace stub with the real 4-step wizard)
- Create: `src/screens/onboarding/StepPersonal.tsx`, `StepCv.tsx`, `StepCriteria.tsx`, `StepLinkedin.tsx`
- Create: `src/onboarding.css`

**Interfaces:**
- Consumes: `api.parseResume`, `api.analyzeCv`, `api.saveProfile`, `api.saveLinkedinCredentials`, the dialog plugin's `open`.
- Produces: a wizard that, on finish, persists the profile + credentials and calls `onDone()` (which flips the gate). All UI strings pt-BR.

- [ ] **Step 1: Extend the API client and types**

In `src/types.ts` add:

```ts
export interface Criteria {
  role: string;
  seniority: string;
  work_model: string;
  locations: string[];
  salary_min: number | null;
  red_lines: string[];
}
```

In `src/lib/api.ts` add the imports and methods:

```ts
import { open } from "@tauri-apps/plugin-dialog";
import type { /* existing */ Criteria } from "../types";

// inside the `api` object:
  parseResume: (path: string) => invoke<string>("parse_resume", { path }),
  analyzeCv: (cvText: string) => invoke<Criteria>("analyze_cv", { cvText }),
  saveLinkedinCredentials: (username: string, password: string) =>
    invoke<void>("save_linkedin_credentials", { username, password }),
  hasLinkedinCredentials: () => invoke<boolean>("has_linkedin_credentials"),
  getLinkedinUsername: () => invoke<string | null>("get_linkedin_username"),

// and a dialog helper exported alongside `api`:
export async function pickResumeFile(): Promise<string | null> {
  const result = await open({
    multiple: false,
    filters: [{ name: "Currículo", extensions: ["pdf", "docx"] }],
  });
  return typeof result === "string" ? result : null;
}
```

(Note: `invoke` maps the Rust `cv_text` arg to JS `cvText` automatically via Tauri's camelCase convention; confirm the command arg is received — if not, pass `{ cv_text: cvText }`.)

- [ ] **Step 2: Build the wizard shell**

Create `src/screens/Onboarding.tsx`:

```tsx
import { useState } from "react";
import { api } from "../lib/api";
import type { Criteria, Profile } from "../types";
import StepPersonal from "./onboarding/StepPersonal";
import StepCv from "./onboarding/StepCv";
import StepCriteria from "./onboarding/StepCriteria";
import StepLinkedin from "./onboarding/StepLinkedin";
import "../onboarding.css";

const EMPTY_CRITERIA: Criteria = {
  role: "", seniority: "", work_model: "", locations: [], salary_min: null, red_lines: [],
};

export default function Onboarding({ onDone }: { onDone: () => void }) {
  const [step, setStep] = useState(0);
  const [personal, setPersonal] = useState({ full_name: "", email: "", phone: "", location: "" });
  const [cvText, setCvText] = useState("");
  const [criteria, setCriteria] = useState<Criteria>(EMPTY_CRITERIA);
  const [linkedin, setLinkedin] = useState({ username: "", password: "" });
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const steps = ["Seus dados", "Currículo", "O que você busca", "Login LinkedIn"];

  async function finish() {
    setSaving(true);
    setError(null);
    try {
      const profile: Profile = {
        ...personal,
        cv_text: cvText,
        criteria_json: JSON.stringify(criteria),
      };
      await api.saveProfile(profile);
      await api.saveLinkedinCredentials(linkedin.username, linkedin.password);
      onDone();
    } catch (e) {
      setError(String(e));
      setSaving(false);
    }
  }

  // Minimum to advance/finish, mirrors the Rust onboarding-complete gate.
  const canFinish =
    personal.full_name.trim() !== "" &&
    cvText.trim() !== "" &&
    criteria.role.trim() !== "" &&
    linkedin.username.trim() !== "" &&
    linkedin.password.trim() !== "";

  return (
    <div className="onb">
      <header className="onb-head">
        <h1>Configuração inicial</h1>
        <ol className="onb-steps">
          {steps.map((s, i) => (
            <li key={s} className={i === step ? "current" : i < step ? "done" : ""}>{s}</li>
          ))}
        </ol>
      </header>

      <main className="onb-body">
        {step === 0 && <StepPersonal value={personal} onChange={setPersonal} />}
        {step === 1 && <StepCv cvText={cvText} setCvText={setCvText} criteria={criteria} setCriteria={setCriteria} />}
        {step === 2 && <StepCriteria value={criteria} onChange={setCriteria} />}
        {step === 3 && <StepLinkedin value={linkedin} onChange={setLinkedin} />}
      </main>

      {error && <p className="onb-error">Erro ao salvar: {error}</p>}

      <footer className="onb-foot">
        <button disabled={step === 0 || saving} onClick={() => setStep((s) => s - 1)}>Voltar</button>
        {step < 3 ? (
          <button onClick={() => setStep((s) => s + 1)}>Próximo</button>
        ) : (
          <button disabled={!canFinish || saving} onClick={finish}>
            {saving ? "Salvando…" : "Concluir"}
          </button>
        )}
      </footer>
    </div>
  );
}
```

- [ ] **Step 3: Step 1 — personal data**

Create `src/screens/onboarding/StepPersonal.tsx`:

```tsx
type Personal = { full_name: string; email: string; phone: string; location: string };

export default function StepPersonal({
  value, onChange,
}: { value: Personal; onChange: (v: Personal) => void }) {
  const set = (k: keyof Personal) => (e: React.ChangeEvent<HTMLInputElement>) =>
    onChange({ ...value, [k]: e.target.value });
  return (
    <section className="step">
      <h2>Seus dados</h2>
      <label>Nome completo<input value={value.full_name} onChange={set("full_name")} /></label>
      <label>E-mail<input value={value.email} onChange={set("email")} /></label>
      <label>Telefone<input value={value.phone} onChange={set("phone")} /></label>
      <label>Localização<input value={value.location} onChange={set("location")} /></label>
      <p className="hint">O nome é obrigatório para concluir.</p>
    </section>
  );
}
```

- [ ] **Step 4: Step 2 — CV upload/paste + analyze**

Create `src/screens/onboarding/StepCv.tsx`:

```tsx
import { useState } from "react";
import { api, pickResumeFile } from "../../lib/api";
import type { Criteria } from "../../types";

export default function StepCv({
  cvText, setCvText, criteria, setCriteria,
}: {
  cvText: string; setCvText: (t: string) => void;
  criteria: Criteria; setCriteria: (c: Criteria) => void;
}) {
  const [busy, setBusy] = useState<"" | "parsing" | "analyzing">("");
  const [note, setNote] = useState<string | null>(null);

  async function upload() {
    setNote(null);
    const path = await pickResumeFile();
    if (!path) return;
    setBusy("parsing");
    try {
      const text = await api.parseResume(path);
      setCvText(text);
    } catch (e) {
      setNote(`Não consegui ler o arquivo: ${e}`);
    } finally {
      setBusy("");
    }
  }

  async function analyze() {
    if (!cvText.trim()) return;
    setBusy("analyzing");
    setNote(null);
    try {
      const c = await api.analyzeCv(cvText);
      if (c.role || c.seniority || c.work_model) {
        setCriteria({ ...criteria, ...c });
        setNote("Critérios pré-preenchidos a partir do currículo. Revise no próximo passo.");
      } else {
        setNote("Não consegui inferir critérios automaticamente — você pode preencher manualmente.");
      }
    } catch (e) {
      setNote(`Análise indisponível (${e}). Preencha manualmente.`);
    } finally {
      setBusy("");
    }
  }

  return (
    <section className="step">
      <h2>Currículo</h2>
      <div className="row">
        <button onClick={upload} disabled={busy !== ""}>
          {busy === "parsing" ? "Lendo…" : "Enviar PDF/DOCX"}
        </button>
        <button onClick={analyze} disabled={!cvText.trim() || busy !== ""}>
          {busy === "analyzing" ? "Analisando…" : "Analisar com Claude"}
        </button>
      </div>
      <label>Texto do currículo
        <textarea rows={12} value={cvText} onChange={(e) => setCvText(e.target.value)}
          placeholder="Cole o texto do seu currículo aqui, ou envie um PDF/DOCX acima." />
      </label>
      {note && <p className="hint">{note}</p>}
    </section>
  );
}
```

- [ ] **Step 5: Step 3 — criteria (pre-filled, editable)**

Create `src/screens/onboarding/StepCriteria.tsx`:

```tsx
import type { Criteria } from "../../types";

export default function StepCriteria({
  value, onChange,
}: { value: Criteria; onChange: (c: Criteria) => void }) {
  const set = <K extends keyof Criteria>(k: K, v: Criteria[K]) => onChange({ ...value, [k]: v });
  return (
    <section className="step">
      <h2>O que você busca</h2>
      <p className="hint">Pré-preenchido pela análise do currículo quando disponível. Ajuste à vontade.</p>
      <label>Cargo<input value={value.role} onChange={(e) => set("role", e.target.value)} /></label>
      <label>Senioridade<input value={value.seniority} onChange={(e) => set("seniority", e.target.value)} placeholder="junior / mid / senior / lead" /></label>
      <label>Modelo de trabalho
        <select value={value.work_model} onChange={(e) => set("work_model", e.target.value)}>
          <option value="">Indiferente</option>
          <option value="remote">Remoto</option>
          <option value="hybrid">Híbrido</option>
          <option value="onsite">Presencial</option>
        </select>
      </label>
      <label>Localizações (separadas por vírgula)
        <input value={value.locations.join(", ")}
          onChange={(e) => set("locations", e.target.value.split(",").map((s) => s.trim()).filter(Boolean))} />
      </label>
      <label>Salário mínimo (R$)
        <input type="number" value={value.salary_min ?? ""}
          onChange={(e) => set("salary_min", e.target.value === "" ? null : Number(e.target.value))} />
      </label>
      <label>Red-lines (o que evitar, separadas por vírgula)
        <input value={value.red_lines.join(", ")}
          onChange={(e) => set("red_lines", e.target.value.split(",").map((s) => s.trim()).filter(Boolean))} />
      </label>
      <p className="hint">O cargo é obrigatório para concluir.</p>
    </section>
  );
}
```

- [ ] **Step 6: Step 4 — LinkedIn login**

Create `src/screens/onboarding/StepLinkedin.tsx`:

```tsx
type Login = { username: string; password: string };

export default function StepLinkedin({
  value, onChange,
}: { value: Login; onChange: (v: Login) => void }) {
  return (
    <section className="step">
      <h2>Login LinkedIn</h2>
      <p className="hint">Guardado com segurança no Gerenciador de Credenciais do Windows — nunca em texto puro.</p>
      <label>E-mail / usuário
        <input value={value.username} onChange={(e) => onChange({ ...value, username: e.target.value })} />
      </label>
      <label>Senha
        <input type="password" value={value.password} onChange={(e) => onChange({ ...value, password: e.target.value })} />
      </label>
    </section>
  );
}
```

- [ ] **Step 7: Styles**

Create `src/onboarding.css`:

```css
.onb { max-width: 640px; margin: 0 auto; padding: 32px 24px; display: flex; flex-direction: column; min-height: 100vh; }
.onb-head h1 { margin: 0 0 12px; }
.onb-steps { display: flex; gap: 8px; list-style: none; padding: 0; margin: 0 0 24px; font-size: 13px; }
.onb-steps li { color: #888; padding: 4px 8px; border-radius: 6px; }
.onb-steps li.current { background: #D97757; color: #fff; }
.onb-steps li.done { color: #4caf50; }
.onb-body { flex: 1; }
.step { display: flex; flex-direction: column; gap: 12px; }
.step label { display: flex; flex-direction: column; gap: 4px; font-size: 14px; }
.step input, .step textarea, .step select { padding: 8px; border: 1px solid #ccc; border-radius: 6px; font: inherit; }
.row { display: flex; gap: 8px; }
.hint { color: #888; font-size: 13px; margin: 4px 0 0; }
.onb-error { color: #c0392b; }
.onb-foot { display: flex; justify-content: space-between; padding-top: 16px; }
.onb-foot button { padding: 8px 20px; border-radius: 6px; border: none; background: #D97757; color: #fff; cursor: pointer; }
.onb-foot button:disabled { opacity: 0.5; cursor: default; }
```

- [ ] **Step 8: Verify build + manual smoke**

Run: `npm run build` → tsc + vite succeed. Then `npm run tauri dev` and walk the wizard: type a name, paste CV text (or upload a PDF/DOCX), click "Analisar com Claude" (criteria may pre-fill), fill a role, enter LinkedIn login, click Concluir. The shell should appear. Restart the app — it should now SKIP onboarding (gate passes because profile + credentials persist).

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "feat: multi-step onboarding wizard with cv upload and analysis"
```

---

### Task 7: Profile screen (edit after onboarding)

**Files:**
- Modify: `src/screens/Profile.tsx` (replace stub with the editor)
- Create: `src/profile.css`

**Interfaces:**
- Consumes: `api.getProfile`, `api.saveProfile`, `api.analyzeCv`, `api.getLinkedinUsername`, `api.saveLinkedinCredentials`.
- Produces: a Profile editor reusing the same fields as onboarding, in form mode, with an optional "Analisar com Claude" assist. Saving persists via `save_profile`.

- [ ] **Step 1: Implement the Profile editor**

Replace `src/screens/Profile.tsx` with:

```tsx
import { useEffect, useState } from "react";
import { api } from "../lib/api";
import type { Criteria, Profile as ProfileT } from "../types";
import "../profile.css";

const EMPTY_CRITERIA: Criteria = {
  role: "", seniority: "", work_model: "", locations: [], salary_min: null, red_lines: [],
};

function parseCriteria(json: string): Criteria {
  try { return { ...EMPTY_CRITERIA, ...JSON.parse(json) }; } catch { return EMPTY_CRITERIA; }
}

export default function Profile() {
  const [profile, setProfile] = useState<ProfileT | null>(null);
  const [criteria, setCriteria] = useState<Criteria>(EMPTY_CRITERIA);
  const [linkedinUser, setLinkedinUser] = useState<string>("");
  const [status, setStatus] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    api.getProfile().then((p) => {
      setProfile(p);
      setCriteria(parseCriteria(p.criteria_json));
    });
    api.getLinkedinUsername().then((u) => setLinkedinUser(u ?? ""));
  }, []);

  if (!profile) return <section className="prof"><h1>Perfil</h1><p>Carregando…</p></section>;

  const setField = (k: keyof ProfileT, v: string) => setProfile({ ...profile, [k]: v });
  const setCrit = <K extends keyof Criteria>(k: K, v: Criteria[K]) => setCriteria({ ...criteria, [k]: v });

  async function analyze() {
    if (!profile!.cv_text.trim()) return;
    setBusy(true); setStatus(null);
    try {
      const c = await api.analyzeCv(profile!.cv_text);
      if (c.role || c.seniority || c.work_model) {
        setCriteria({ ...criteria, ...c });
        setStatus("Critérios atualizados pela análise. Revise e salve.");
      } else {
        setStatus("Não consegui inferir critérios — ajuste manualmente.");
      }
    } catch (e) {
      setStatus(`Análise indisponível (${e}).`);
    } finally { setBusy(false); }
  }

  async function save() {
    setBusy(true); setStatus(null);
    try {
      await api.saveProfile({ ...profile!, criteria_json: JSON.stringify(criteria) });
      setStatus("Perfil salvo.");
    } catch (e) {
      setStatus(`Erro ao salvar: ${e}`);
    } finally { setBusy(false); }
  }

  return (
    <section className="prof">
      <h1>Perfil</h1>

      <h2>Seus dados</h2>
      <label>Nome completo<input value={profile.full_name} onChange={(e) => setField("full_name", e.target.value)} /></label>
      <label>E-mail<input value={profile.email} onChange={(e) => setField("email", e.target.value)} /></label>
      <label>Telefone<input value={profile.phone} onChange={(e) => setField("phone", e.target.value)} /></label>
      <label>Localização<input value={profile.location} onChange={(e) => setField("location", e.target.value)} /></label>
      <label>LinkedIn<input value={linkedinUser} disabled title="Editável na reconfiguração de credenciais" /></label>

      <h2>Currículo</h2>
      <label><textarea rows={10} value={profile.cv_text} onChange={(e) => setField("cv_text", e.target.value)} /></label>
      <button onClick={analyze} disabled={busy || !profile.cv_text.trim()}>
        {busy ? "Analisando…" : "Analisar com Claude"}
      </button>

      <h2>O que você busca</h2>
      <label>Cargo<input value={criteria.role} onChange={(e) => setCrit("role", e.target.value)} /></label>
      <label>Senioridade<input value={criteria.seniority} onChange={(e) => setCrit("seniority", e.target.value)} placeholder="junior / mid / senior / lead" /></label>
      <label>Modelo de trabalho
        <select value={criteria.work_model} onChange={(e) => setCrit("work_model", e.target.value)}>
          <option value="">Indiferente</option>
          <option value="remote">Remoto</option>
          <option value="hybrid">Híbrido</option>
          <option value="onsite">Presencial</option>
        </select>
      </label>
      <label>Localizações (vírgula)
        <input value={criteria.locations.join(", ")}
          onChange={(e) => setCrit("locations", e.target.value.split(",").map((s) => s.trim()).filter(Boolean))} />
      </label>
      <label>Salário mínimo (R$)
        <input type="number" value={criteria.salary_min ?? ""}
          onChange={(e) => setCrit("salary_min", e.target.value === "" ? null : Number(e.target.value))} />
      </label>
      <label>Red-lines (vírgula)
        <input value={criteria.red_lines.join(", ")}
          onChange={(e) => setCrit("red_lines", e.target.value.split(",").map((s) => s.trim()).filter(Boolean))} />
      </label>

      <div className="prof-actions">
        <button onClick={save} disabled={busy}>Salvar</button>
        {status && <span className="prof-status">{status}</span>}
      </div>
    </section>
  );
}
```

- [ ] **Step 2: Styles**

Create `src/profile.css`:

```css
.prof { max-width: 640px; display: flex; flex-direction: column; gap: 10px; }
.prof h2 { margin: 16px 0 4px; font-size: 16px; }
.prof label { display: flex; flex-direction: column; gap: 4px; font-size: 14px; }
.prof input, .prof textarea, .prof select { padding: 8px; border: 1px solid #ccc; border-radius: 6px; font: inherit; }
.prof input:disabled { background: #f0f0f0; color: #777; }
.prof-actions { display: flex; align-items: center; gap: 12px; margin-top: 16px; }
.prof-actions button { padding: 8px 24px; border: none; border-radius: 6px; background: #D97757; color: #fff; cursor: pointer; }
.prof-status { color: #555; font-size: 14px; }
```

- [ ] **Step 3: Verify build + manual smoke**

Run: `npm run build` → succeeds. Then `npm run tauri dev`, complete onboarding, go to Perfil: fields are populated from the DB; edit the role, click Salvar, see "Perfil salvo."; navigate away and back — the change persists.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: profile editor screen"
```

---

## Plan 2 Self-Review

- **Spec coverage:** PDF parse (Task 1) ✓; DOCX parse (Task 2) ✓; upload via dialog (Task 3) ✓; CV analysis pre-fill via `claude -p` one-shot with graceful fallback (Task 5, consumed in Tasks 6-7) ✓; LinkedIn creds in OS keychain + real onboarding gate (Task 4) ✓; multi-step onboarding wizard replacing the provisional button (Task 6) ✓; manual entry always available (textarea + editable criteria, no forced AI) ✓; Profile editor form + Claude-assisted (Task 7) ✓; pt-BR UI / English code (throughout) ✓.
- **Placeholder scan:** No TBD/TODO. Task 3 deliberately shows the extractor-ordering iteration ending in a single FINAL version to use — not a placeholder. Fixture generation has a documented fallback if a crate is picky.
- **Type consistency:** `Criteria` matches across Rust (`cv_analysis.rs`), TS (`types.ts`), the wizard, and the Profile screen — same six fields, same `work_model` enum values. `criteria_json` shape matches the Global Constraints object. Command names match between `generate_handler!` (Tasks 3-5) and `api.ts` (Task 6). `is_onboarding_complete` is now fed the real `has_linkedin()` (Task 4), so the gate the wizard satisfies (name + cv + role + linkedin creds) exactly mirrors the Rust check.

## Hand-off to Plan 3

Plan 3 (agent engine) consumes Plan 2's outputs: `profile::get` (to build the system prompt), `credentials::current_username`/the stored password (to log into LinkedIn), and the `criteria_json` (search parameters). It adds the prompt builder, the `AgentRunner` that spawns `claude --chrome`, the `StateWatcher`, and the English system prompt — plus `PRAGMA journal_mode=WAL` once a second SQLite connection is introduced.
