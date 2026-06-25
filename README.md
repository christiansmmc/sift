# Sift

**Sift** is a Windows desktop app that finds jobs on **LinkedIn**, writes a tailored cover
letter and screening answers for each one, and **shows them to you for approval before
anything is submitted**. You stay in control — the agent never sends an application without
your explicit "Approve".

It drives **your own Chrome browser** through Claude. There is no scraping API and no
metered API billing: Sift runs the **Claude Code CLI** under the hood, which operates Chrome
through the **Claude in Chrome** extension using your existing Claude subscription.

> The app's UI is in Brazilian Portuguese (pt-BR). This README is in English.

---

## How it works

```
┌──────────────────────────────────────────────┐
│  Sift (desktop app)                            │
│   • You configure your profile once            │
│   • You start a run and approve/reject jobs    │
└───────────────┬────────────────────────────────┘
                │ spawns
                ▼
         Claude Code CLI  ──drives──►  Chrome (Claude in Chrome)  ──►  LinkedIn
```

1. You fill in your profile (CV, personal data, what you're looking for) — once.
2. You press **Iniciar** (Start). Sift launches the Claude Code CLI in the background.
3. Claude controls **your open Chrome window** through the Claude in Chrome extension,
   searches LinkedIn, evaluates each posting against your criteria, and prepares a tailored
   cover letter + answers for the good Easy-Apply matches.
4. Prepared applications land in **Vagas** (Jobs) with status *awaiting approval*. You review
   the letter and answers, then **Approve** or **Reject**.
5. Approved applications are submitted in a separate "send" run — and only then.

Anything the agent can't safely handle (a captcha, an application that leaves LinkedIn, a
required question with no answer in your profile) becomes a **Pendência** (Pending item)
instead of a guess. The agent never invents information to fill a field.

---

## Requirements

You need three things working **before** you start a run:

1. **Google Chrome**, installed and **open**.
2. The **Claude in Chrome** extension, installed and **connected** (signed in to Claude).
   See: https://www.anthropic.com/claude-in-chrome
3. The **Claude Code CLI** (`claude`) installed and on your `PATH`, signed in with an active
   Claude subscription. Install: https://docs.claude.com/en/docs/claude-code
   Verify in a terminal:
   ```
   claude --version
   ```

You also need to be **logged into LinkedIn in that same Chrome window**. Sift assumes the
LinkedIn session is already active in the browser — it does not store your LinkedIn password.

---

## Install (Windows)

1. Go to the [**Releases**](../../releases) page.
2. Download one of the Windows installers from the latest release:
   - `sift_<version>_x64-setup.exe` — NSIS installer (recommended), **or**
   - `sift_<version>_x64_en-US.msi` — MSI installer.
3. Run the installer and launch **Sift** from the Start menu.

> The build is not code-signed, so Windows SmartScreen may warn on first launch. Click
> **More info → Run anyway**.

---

## First run — onboarding

On first launch a short, mandatory wizard collects the minimum profile. You can't start a
search until it's complete.

1. **Currículo (CV)** — upload a **PDF/DOCX**, paste the text, or let Claude extract it. The
   resume is parsed to plain text; Claude can pre-fill your likely role/seniority from it.
2. **Seus dados (Personal data)** — name (required), email, phone, location.
3. **O que você busca (What you're looking for)** — role (required), seniority, work model
   (remote/hybrid/onsite), locations, minimum salary, and any red lines.

When you're done, the Dashboard's **Iniciar** button is enabled.

---

## Daily use

### Dashboard (Painel)
- **Modo (Mode):**
  - **Revisar (prepare for approval)** — finds matches and prepares cover letters + answers
    for you to approve. This is the normal mode.
  - **Apenas buscar vagas (scan only)** — only discovers and lists jobs, prepares nothing.
- **Vagas por busca (Jobs per run)** — how many postings to process in one run.
- **Iniciar / Parar (Start / Stop)** — start or stop the agent. Make sure Chrome is open and
  connected before you start.
- **Enviar aprovadas (Send approved)** — appears when you have approved applications; this
  runs the submission step that actually applies on LinkedIn.
- Counters show **Found · Awaiting approval · Submitted · Pending**, and a live activity feed
  follows what the agent is doing.

### Vagas (Jobs)
The list of everything the agent found. For each *awaiting approval* item you can read the
generated cover letter and answers and then **Aprovar** (Approve) or rejeitar (Reject).
Approving queues it for the next "Send approved" run.

### Pendências (Pending)
Blockers that need you: captchas, applications that leave LinkedIn, or screening questions
with no answer in your profile. Resolve them (e.g. add the missing answer) and the agent can
use that answer next time.

### Perfil (Profile)
Edit your CV, personal data, and search criteria any time after onboarding.

### Configurações (Settings)
- **Cover-letter style** — short, balanced, detailed, or your own custom instructions.
- **Follow the company on apply** — off by default.
- **Agent model** — Sonnet 4.6 (fast, recommended), Opus 4.8 (best quality), or
  Haiku 4.5 (fastest). Applies to search, submission, and CV analysis.

---

## Privacy & data

- Everything (profile, CV text, jobs, applications, pending items) is stored **locally** in a
  SQLite database on your machine.
- Sift does **not** store your LinkedIn credentials — it relies on the LinkedIn session
  already open in your Chrome.
- The agent runs under your own Claude subscription via the Claude Code CLI.

---

## Troubleshooting

| Symptom | Fix |
|---|---|
| "Falha ao iniciar o agente (claude)" | The `claude` CLI isn't installed or not on `PATH`. Install Claude Code and verify `claude --version`. |
| Agent does nothing / "Browser extension is not connected" | Open Chrome and make sure the Claude in Chrome extension is connected/signed in before pressing **Iniciar**. |
| A *Pendência* says login is required | Log into LinkedIn in the same Chrome window, then start again. |
| External-application pending items | Those jobs apply on the company's own site (not LinkedIn Easy Apply); Sift won't fill them automatically. |

---

## Build from source

Prerequisites: [Node.js](https://nodejs.org), the [Rust toolchain](https://rustup.rs), and
the [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/) for Windows.

```bash
npm install
npm run tauri dev      # run in development
npm run tauri build    # produce the Windows installers in
                       # src-tauri/target/release/bundle/
```

**Stack:** Tauri v2 · React 19 · TypeScript · Rust (SQLite via rusqlite). The Rust backend
spawns the Claude Code CLI headless (`claude -p --chrome`), streams its output, and persists
results to SQLite; the React UI watches that state and reflects it in real time.

The app icon and brand assets live in [`brand/`](brand/).

---

## License

No license has been chosen yet; all rights reserved by the author until one is added.
