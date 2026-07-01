<div align="center">

<img src="brand/png/sift-256.png" width="112" alt="Sift logo" />

# Sift

**Finds jobs on LinkedIn, drafts a tailored cover letter and screening answers for each one,
and shows them to you for approval — nothing is submitted without your "Approve".**

[![Website](https://img.shields.io/badge/website-christiansmmc.github.io%2Fsift-5b54e6?style=flat-square)](https://christiansmmc.github.io/sift/)
![Platform](https://img.shields.io/badge/platform-Windows-0f141a?style=flat-square)
![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%20v2-6f67f7?style=flat-square)
![Powered by Claude](https://img.shields.io/badge/powered%20by-Claude-22d3ee?style=flat-square)

🌐 **[christiansmmc.github.io/sift](https://christiansmmc.github.io/sift/)**

</div>

---

Sift is a Windows desktop app that drives **your own Chrome browser** through Claude. There's no
scraping API and no metered API billing: it runs the **Claude Code CLI** under the hood, which
operates Chrome via the **Claude in Chrome** extension using your existing Claude subscription.
You stay in control — the agent never sends an application without your explicit approval, and it
never invents information to fill a field.

> 🇧🇷 The app's UI is in Brazilian Portuguese (pt-BR). This README is in English.

<br>

## Quick start

1. **Install Sift** → [download an installer](#-install) or [build from source](#-build-from-source).
2. **Have the prerequisites running** → Chrome open, [Claude in Chrome](https://www.anthropic.com/claude-in-chrome) connected, [Claude Code CLI](https://docs.claude.com/en/docs/claude-code) signed in, and LinkedIn logged in. ([details](#requirements))
3. **Complete onboarding** → upload your CV and fill your profile (one time).
4. **Press Iniciar** → review each prepared application and **Approve** or **Reject**.
5. **Press Enviar aprovadas** → approved applications are submitted, and only then.

<br>

## 📦 Install

### ⬇️ Download (recommended)

1. Open the [**Releases**](../../releases) page.
2. Download the installer for your platform from the latest release:
   - **Windows** — `sift_<version>_x64-setup.exe` (NSIS, **recommended**) or `sift_<version>_x64_en-US.msi`
   - **Linux** — `sift_<version>_amd64.AppImage` (`chmod +x` it, then run) or `sift_<version>_amd64.deb`
3. Run it, then launch **Sift**.

> The build isn't code-signed, so Windows SmartScreen may warn on first launch.
> Click **More info → Run anyway**.

### 🔧 Build from source

Prerequisites: [Node.js](https://nodejs.org), the [Rust toolchain](https://rustup.rs), and the
[Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/) for Windows.

```bash
git clone https://github.com/christiansmmc/sift.git
cd sift
npm install
npm run tauri dev      # run in development
npm run tauri build    # build installers → src-tauri/target/release/bundle/
```

<br>

## Requirements

Before you start a run, these four things must be ready:

| | What | Why |
|---|---|---|
| 🌐 | **Google Chrome**, installed and **open** | Sift drives this window |
| 🧩 | **[Claude in Chrome](https://www.anthropic.com/claude-in-chrome)** extension, connected & signed in | lets Claude control the browser |
| 💻 | **[Claude Code CLI](https://docs.claude.com/en/docs/claude-code)** (`claude`) on your `PATH`, with an active subscription | runs the agent — verify with `claude --version` |
| 🔗 | **LinkedIn** logged in **in that same Chrome window** | Sift uses your existing session; it never stores your password |

<br>

## How it works

```
┌────────────────────────────────────────────────┐
│  Sift (desktop app)                              │
│   • You configure your profile once              │
│   • You start a run and approve / reject jobs    │
└───────────────┬──────────────────────────────────┘
                │ spawns
                ▼
        Claude Code CLI  ──drives──►  Chrome (Claude in Chrome)  ──►  LinkedIn
```

1. Press **Iniciar** (Start). Sift launches the Claude Code CLI in the background.
2. Claude controls your open Chrome window, searches LinkedIn, evaluates each posting against
   your criteria, and prepares a tailored cover letter + answers for good Easy-Apply matches.
3. Prepared applications land in **Vagas** (Jobs) as *awaiting approval*. You review and
   **Approve** or **Reject**.
4. Approved applications are submitted in a separate **Enviar aprovadas** (Send approved) run —
   and only then.

Anything the agent can't safely handle (a captcha, an application that leaves LinkedIn, a
required question with no answer in your profile) becomes a **Pendência** (Pending item) instead
of a guess.

<br>

## Using Sift

<details>
<summary><strong>First run — onboarding</strong></summary>

<br>

A short, mandatory wizard collects the minimum profile. You can't start a search until it's done.

1. **Currículo (CV)** — upload a **PDF/DOCX**, paste the text, or let Claude extract it. The
   resume is parsed to plain text; Claude can pre-fill your likely role/seniority from it.
2. **Seus dados (Personal data)** — name (required), email, phone, location.
3. **O que você busca (What you're looking for)** — role (required), seniority, work model
   (remote/hybrid/onsite), locations, minimum salary, and any red lines.

When you're done, the Dashboard's **Iniciar** button is enabled.

</details>

<details>
<summary><strong>Daily use — Dashboard, Jobs, Pending, Profile, Settings</strong></summary>

<br>

**Dashboard (Painel)**
- **Modo (Mode):** *Revisar* (prepare cover letters + answers for approval — the normal mode) or
  *Apenas buscar vagas* (scan only — just discover and list jobs).
- **Vagas por busca (Jobs per run)** — how many postings to process in one run.
- **Iniciar / Parar (Start / Stop)** — start or stop the agent (Chrome must be open and connected).
- **Enviar aprovadas (Send approved)** — appears when you have approved applications; runs the
  submission step that actually applies on LinkedIn.
- Counters show **Found · Awaiting approval · Submitted · Pending**, with a live activity feed.

**Vagas (Jobs)** — everything the agent found. For each *awaiting approval* item, read the
generated cover letter and answers, then **Aprovar** or **Rejeitar**. Approving queues it for the
next *Send approved* run.

**Pendências (Pending)** — blockers that need you: captchas, applications that leave LinkedIn, or
screening questions with no answer in your profile. Resolve them (e.g. add the missing answer) and
the agent can reuse that answer next time.

**Perfil (Profile)** — edit your CV, personal data, and search criteria any time.

**Configurações (Settings)** — cover-letter style (short / balanced / detailed / custom), whether
to follow the company on apply (off by default), and the agent model (Sonnet 5 — fast,
recommended; Opus 4.8 — best quality; Haiku 4.5 — fastest).

</details>

<details>
<summary><strong>Troubleshooting</strong></summary>

<br>

| Symptom | Fix |
|---|---|
| "Falha ao iniciar o agente (claude)" | The `claude` CLI isn't installed or not on `PATH`. Install Claude Code and verify `claude --version`. |
| Agent does nothing / "Browser extension is not connected" | Open Chrome and make sure the Claude in Chrome extension is connected/signed in before pressing **Iniciar**. |
| A *Pendência* says login is required | Log into LinkedIn in the same Chrome window, then start again. |
| External-application pending items | Those jobs apply on the company's own site (not LinkedIn Easy Apply); Sift won't fill them automatically. |

</details>

<br>

## Privacy & data

- Everything (profile, CV text, jobs, applications, pending items) is stored **locally** in a
  SQLite database on your machine.
- Sift does **not** store your LinkedIn credentials — it relies on the session already open in Chrome.
- The agent runs under your own Claude subscription via the Claude Code CLI.

<br>

## Tech stack

**Tauri v2 · React 19 · TypeScript · Rust** (SQLite via rusqlite). The Rust backend spawns the
Claude Code CLI headless (`claude -p --chrome`), streams its output, and persists results to
SQLite; the React UI watches that state and reflects it in real time. Brand assets live in
[`brand/`](brand/).

<br>

## License

Released under the [MIT License](LICENSE).

<br>

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for setup,
the dev workflow, and how to open a pull request.
