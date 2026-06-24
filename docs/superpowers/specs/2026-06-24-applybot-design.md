# applybot — Design Document

**Date:** 2026-06-24
**Status:** Approved (brainstorming)
**Author:** Christian Sequeira (with Claude)

---

## 1. Summary

**applybot** is a Windows desktop app that finds job postings on LinkedIn, generates a
tailored CV + cover letter for each one, and **shows them to the user for approval before
submitting**. The user opens the app, configures their profile once (forced onboarding), and
the agent does the work.

This is a clean-room rebuild inspired by an existing project (`claudia-rh`) that the user
found confusing and poorly implemented. **No code is reused.** All new code is written in
**English** (identifiers, comments, schema, system prompts). The **user-facing UI is in
natural Brazilian Portuguese (pt-BR)** — fixing the original's "weird European Portuguese"
problem.

### Design principles

- **Simple and whole** — does the core job well, no unnecessary features.
- **Onboarding-first** — open → configure once → only then use. Searching is blocked until
  the minimum profile is filled.
- **User in control** — review-before-send. The agent never submits an application without
  explicit approval.
- **No clutter** — no exposed terminal, no feedback charts. Four clean screens.

### Explicitly out of scope (cut from the original)

- Exposed PTY terminal screen.
- Feedback tab with trend charts.
- Multiple job sources at launch (LinkedIn only for MVP; source list is architected to be
  extensible later).
- Fully autonomous auto-submit (may be considered later behind a setting; not in MVP).

---

## 2. Architecture (Approach A — reuse the proven engine, rebuild the bad parts)

The original's automation mechanism is proven and is the hard part, so we keep the *pattern*
(not the code): Rust spawns the Claude Code CLI (`claude --chrome`) with a system prompt; the
agent drives Chrome via the Claude in Chrome extension and writes progress to a local SQLite
DB; the UI watches SQLite and reflects state in real time.

```
┌─────────────────────────────────────────────────────┐
│                  applybot (Tauri v2)                 │
│                                                      │
│   React 19 + TypeScript  (UI in pt-BR)               │
│   ├─ Onboarding wizard (first run, mandatory)        │
│   └─ 4 screens: Dashboard · Jobs · Pending · Profile │
│   ──────────────────────────────────────────────     │
│   Rust (Tauri v2)                                    │
│   ├─ AgentRunner   → spawns `claude --chrome` (PTY)  │
│   ├─ SQLite store  (jobs, applications, pending…)    │
│   ├─ ResumeParser  (PDF/DOCX → plain text)           │
│   ├─ StateWatcher  (SQLite → Tauri events to UI)     │
│   └─ Credentials   (Windows Credential Manager)      │
└─────────────────────────────────────────────────────┘
        Claude Code CLI + Claude in Chrome → LinkedIn
```

- The internal PTY/agent process is **never shown** to the user. The UI only shows clean
  screens driven by DB state.
- The agent uses the user's Claude subscription (via the CLI), not the metered API.
- Stack: Tauri v2, React 19, TypeScript, Rust (2021 edition), SQLite via rusqlite, keyring v3.
  Windows 11.

### Components and responsibilities

| Component | What it does | Depends on |
|---|---|---|
| `AgentRunner` (Rust) | Spawns/stops `claude --chrome`, injects the English system prompt, watches stdout for signal lines, restarts on checkpoint. | PTY, prompt builder |
| `prompt builder` (Rust) | Assembles the system prompt from profile + search criteria + recent memory. English. | SQLite/profile files |
| `SQLite store` (Rust) | Single source of truth for jobs, applications, pending actions, sessions, profile. | rusqlite |
| `StateWatcher` (Rust) | Watches the DB for changes and emits Tauri events to the frontend. | SQLite |
| `ResumeParser` (Rust) | Extracts plain text from an uploaded PDF/DOCX so it can be analyzed/stored. | a PDF + a DOCX crate |
| `Credentials` (Rust) | Stores LinkedIn login in Windows Credential Manager — never plain text. | keyring |
| Frontend (React/TS) | Onboarding wizard + 4 screens. Reads DB via Tauri commands, listens to StateWatcher events. | Tauri commands |

---

## 3. Screens and onboarding

### Onboarding (first run — mandatory, blocks usage until complete)

A short wizard. Searching cannot start until the minimum is filled.

1. **Personal data** — name, contact, location.
2. **CV** — upload **PDF/DOCX**, *or* paste text, *or* let Claude extract it. The uploaded
   file is parsed to plain text and analyzed.
3. **What you're looking for** — role, seniority, work model (remote/hybrid/onsite), salary
   range, red-lines. **Pre-populated from the CV analysis in step 2** (Claude infers likely
   role/seniority/work model from the resume); the user reviews and adjusts rather than
   filling from scratch.
4. **LinkedIn login** — credentials stored in Windows Credential Manager.

Completion gate: onboarding is "done" only when personal data + a CV + minimum search
criteria + LinkedIn credentials exist. Until then the Dashboard's "Start" action is disabled
and explains what's missing.

### The four screens

- **Dashboard (Painel)** — current state (running / stopped), a Start/Stop button, and simple
  counters: found, awaiting approval, submitted, pending. Nothing else.
- **Jobs (Vagas)** — list of jobs the agent found, each with a status (analyzed, awaiting
  approval, submitted, skipped, discarded). This is where the user **approves or rejects**
  each application and views the generated CV/cover letter before it is sent.
- **Pending (Pendências)** — blockers that need the user: captcha, an unanswered form
  question, salary outside range, sensitive data. The user resolves and releases.
- **Profile (Perfil)** — edit personal data / CV / criteria after onboarding. **Form mode or
  Claude-assisted mode** (assisted is the default, but manual form entry is always available —
  the user is never forced to chat with an agent).

---

## 4. Application flow

```
agent finds a job → evaluates match against the profile
   ├─ no match        → mark "skipped"   (no interruption)
   ├─ match           → generate CV + cover letter → status "awaiting approval"
   │                     ├─ user APPROVES (Jobs screen) → agent submits → "submitted"
   │                     └─ user REJECTS               → "discarded"
   └─ blocked (captcha / unanswered field / salary out of range / sensitive data)
                       → create a "pending action"  (shown on Pending screen)
```

The agent **never submits without explicit user approval**. Pause/stop rules (captcha,
salary, red-lines, sensitive data, open questions with no profile-backed answer) become
pending actions instead of guesses — the agent never invents information to fill a field.

---

## 5. Data model (SQLite — all English)

- `jobs` — discovered postings: title, company, url, source, status, match summary,
  discovered_at.
- `applications` — one per submitted/approved application: job_id, generated-files folder
  path, cv_path, cover_letter_path, submitted_at, status.
- `pending_actions` — blockers: job_id, category, description, resolved flag, created_at.
- `profile` — personal data, CV text, and search criteria. **SQLite is the single source of
  truth**; the prompt builder reads from here and injects it into the agent's system prompt
  (no separate YAML profile files like the original).
- `sessions` — run history: started_at, ended_at, counts, end reason.

Statuses are English enums (e.g. `discovered`, `analyzed`, `awaiting_approval`, `submitted`,
`skipped`, `discarded`, `pending_review`).

---

## 6. Error handling

- **Chrome extension drop** ("Browser extension is not connected" / "Receiving end does not
  exist") — the agent first tries to reconnect itself (`/chrome` → Reconnect). Only if
  reconnection fails repeatedly does it become a visible pending action describing a
  connection failure (not a decision about the job).
- **Nothing fails silently** — any blocker the agent can't safely handle becomes a visible
  pending action.
- **Agent process crash** — AgentRunner detects exit and reflects "stopped" on the Dashboard;
  state is persisted in SQLite so a restart resumes from DB, not from context.

---

## 7. Testing strategy

- **TDD** for the parts testable in isolation:
  - `ResumeParser` — PDF/DOCX → text (fixture files, known expected output).
  - prompt builder — given profile + criteria, produces the expected system prompt.
  - SQLite store — CRUD and status-transition logic.
  - status/flow logic — match → awaiting_approval → submitted, etc.
- **Manual validation** for the browser automation itself (cannot be tested reliably in CI):
  run against LinkedIn and observe behavior.

---

## 8. Open items for the implementation plan

- Choose Rust crates for PDF and DOCX text extraction.
- Define the exact English system-prompt text (port the *good* rules from the original —
  pause rules, honesty rule — rewritten in English and pt-BR-market-neutral; drop the
  Danish/European specifics).
- Decide the source-list config shape so adding Gupy/Catho/etc. later is a config change, not
  a rewrite.
- Define the Tauri command surface between frontend and Rust.
