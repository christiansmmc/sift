# applybot — Plan (Polish): Theming + Live Agent Activity

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give the app a real visual identity (light/dark theme + cohesive components) and a live activity feed on the Dashboard so the user sees what the agent is doing during a run.

**Architecture:** A hand-rolled CSS-variable design system (`src/theme.css`) drives both themes via `[data-theme]` on `<html>`; a tiny `src/lib/theme.ts` persists the choice. Screens drop inline styles for shared component classes (`.card`, `.btn`, `.pill`, etc.). For activity: a new `APPLYBOT_STATUS <text>` marker is parsed in the runner and emitted as a UI-only `agent://status` Tauri event (never persisted); the Dashboard renders a scrolling feed that clears each run.

**Tech Stack:** React 19 + TS, CSS custom properties (no UI lib), Rust (protocol + runner), Tauri events.

## Global Constraints

- Platform Windows 11, desktop-only Tauri.
- Code/identifiers/comments in **English**; ALL UI strings in **pt-BR**.
- No new dependencies. Theming via CSS variables only.
- The activity feed is **UI-only** — `APPLYBOT_STATUS` never reaches the DB sink and is never persisted.
- Default theme: **dark**. Accent color `#D97757` in both themes.
- Reuse existing screens/commands; do not change agent behavior beyond emitting status lines.
- Conventional Commits.

---

### Task 1: Theme system — tokens, helper, toggle

**Files:**
- Create: `src/theme.css`
- Create: `src/lib/theme.ts`
- Modify: `src/main.tsx` (import `theme.css`, init theme before render)
- Modify: `src/App.tsx` (sidebar header + theme toggle button)

**Interfaces:**
- `theme.ts`: `getTheme(): "light" | "dark"`, `setTheme(t)`, `toggleTheme(): "light" | "dark"`, `initTheme()` (applies persisted/default to `document.documentElement.dataset.theme`).

- [ ] **Step 1: Design tokens + base components (`src/theme.css`)**

```css
:root,
:root[data-theme="dark"] {
  --bg: #1a1a1a;
  --surface: #242424;
  --surface-2: #2e2e2e;
  --text: #ececec;
  --text-muted: #9a9a9a;
  --border: #3a3a3a;
  --accent: #D97757;
  --accent-hover: #c4623f;
  --danger: #e5534b;
  --success: #4caf50;
  --radius: 8px;
  --shadow: 0 1px 3px rgba(0,0,0,0.4);
}
:root[data-theme="light"] {
  --bg: #f6f6f5;
  --surface: #ffffff;
  --surface-2: #f0efee;
  --text: #1f1f1f;
  --text-muted: #6b6b6b;
  --border: #e2e0dd;
  --accent: #D97757;
  --accent-hover: #c4623f;
  --danger: #c0392b;
  --success: #2e7d32;
  --radius: 8px;
  --shadow: 0 1px 3px rgba(0,0,0,0.08);
}

* { box-sizing: border-box; }
body { margin: 0; font-family: system-ui, -apple-system, sans-serif; background: var(--bg); color: var(--text); }
h1 { font-size: 22px; margin: 0 0 16px; }
h2 { font-size: 15px; margin: 20px 0 8px; color: var(--text-muted); text-transform: uppercase; letter-spacing: .04em; }
a { color: var(--accent); cursor: pointer; }

.card { background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius); padding: 16px; margin: 12px 0; box-shadow: var(--shadow); }

.btn { padding: 8px 16px; border-radius: var(--radius); border: 1px solid var(--border); background: var(--surface-2); color: var(--text); cursor: pointer; font: inherit; }
.btn:hover { border-color: var(--accent); }
.btn:disabled { opacity: .5; cursor: default; }
.btn-primary { background: var(--accent); border-color: var(--accent); color: #fff; }
.btn-primary:hover { background: var(--accent-hover); border-color: var(--accent-hover); }
.btn-ghost { background: transparent; }

label.field { display: flex; flex-direction: column; gap: 4px; font-size: 14px; margin-bottom: 10px; color: var(--text); }
input, select, textarea { padding: 8px 10px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface); color: var(--text); font: inherit; }
input:disabled { background: var(--surface-2); color: var(--text-muted); }

.pill { display: inline-block; padding: 2px 10px; border-radius: 999px; font-size: 12px; font-weight: 600; background: var(--surface-2); color: var(--text-muted); }
.pill-awaiting_approval { background: #4a3a1a; color: #f0c060; }
.pill-approved { background: #1f3d2a; color: #6fcf8f; }
.pill-submitted { background: #1e3a4a; color: #6fb3df; }
.pill-discarded { background: #3a2222; color: #e08585; }
.pill-analyzed { background: var(--surface-2); color: var(--text-muted); }
:root[data-theme="light"] .pill-awaiting_approval { background: #fdf0d0; color: #8a6400; }
:root[data-theme="light"] .pill-approved { background: #d8f0e0; color: #1f7a३f; }
:root[data-theme="light"] .pill-submitted { background: #d6ecf8; color: #1a5f8a; }
:root[data-theme="light"] .pill-discarded { background: #f8d6d6; color: #9a2a2a; }

.hint { color: var(--text-muted); font-size: 13px; margin: 4px 0 0; }

.app { display: flex; min-height: 100vh; }
.sidebar { width: 200px; background: var(--surface); border-right: 1px solid var(--border); display: flex; flex-direction: column; padding: 16px 12px; gap: 4px; }
.sidebar .brand { font-weight: 700; font-size: 18px; color: var(--text); padding: 4px 12px 16px; }
.sidebar .brand span { color: var(--accent); }
.sidebar nav { display: flex; flex-direction: column; gap: 2px; flex: 1; }
.sidebar button.navlink { background: transparent; color: var(--text-muted); border: none; text-align: left; padding: 10px 12px; border-radius: var(--radius); cursor: pointer; font-size: 14px; }
.sidebar button.navlink:hover { background: var(--surface-2); color: var(--text); }
.sidebar button.navlink.active { background: var(--accent); color: #fff; }
.sidebar .theme-toggle { background: transparent; border: 1px solid var(--border); color: var(--text-muted); border-radius: var(--radius); padding: 8px; cursor: pointer; font-size: 14px; }
.content { flex: 1; padding: 28px 32px; overflow-y: auto; max-height: 100vh; }
.loading { padding: 24px; color: var(--text-muted); }
```

(Fix any stray non-ASCII the moment it appears: the `pill-approved` light color must be `#1f7a3f` — ensure plain hex digits.)

- [ ] **Step 2: Theme helper (`src/lib/theme.ts`)**

```ts
export type Theme = "light" | "dark";
const KEY = "applybot-theme";

export function getTheme(): Theme {
  return (localStorage.getItem(KEY) as Theme) ?? "dark";
}
export function setTheme(t: Theme) {
  localStorage.setItem(KEY, t);
  document.documentElement.dataset.theme = t;
}
export function toggleTheme(): Theme {
  const next: Theme = getTheme() === "dark" ? "light" : "dark";
  setTheme(next);
  return next;
}
export function initTheme() {
  setTheme(getTheme());
}
```

- [ ] **Step 3: Wire into boot + sidebar**

In `src/main.tsx`, add `import "./theme.css";` and call `initTheme()` (import from `./lib/theme`) before `ReactDOM...render`. Keep the existing `App.css` import only if still needed; the shared styles now live in `theme.css` (you may remove `App.css` usage if its rules are superseded — but don't delete onboarding.css/profile.css yet).

In `src/App.tsx`: render a branded sidebar and a theme toggle. Replace the `<nav className="sidebar">` block with:
```tsx
import { getTheme, toggleTheme, type Theme } from "./lib/theme";
// inside App():
  const [theme, setThemeState] = useState<Theme>(getTheme());
  // ...
      <nav className="sidebar">
        <div className="brand">apply<span>bot</span></div>
        <nav>
          {NAV.map((n) => (
            <button key={n.key} className={`navlink ${screen === n.key ? "active" : ""}`} onClick={() => setScreen(n.key)}>
              {n.label}
            </button>
          ))}
        </nav>
        <button className="theme-toggle" onClick={() => setThemeState(toggleTheme())}>
          {theme === "dark" ? "☀️  Tema claro" : "🌙  Tema escuro"}
        </button>
      </nav>
```
(Keep the outer `<div className="app">` and `<main className="content">`.)

- [ ] **Step 4: Build + verify**

Run: `npm run build` → tsc + vite success.

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "feat: light/dark theme system with css tokens and sidebar toggle"
```

---

### Task 2: Restyle Dashboard + Vagas with the design system

**Files:**
- Modify: `src/screens/Dashboard.tsx`
- Modify: `src/screens/Jobs.tsx`

**Interfaces:** none new — applies `theme.css` classes.

- [ ] **Step 1: Dashboard**

Replace inline styles with classes. Controls row → a `.card` containing the Modo `<select>`, batch input (use `<label className="field">`), and an `Iniciar`/`Parar` `<button className="btn btn-primary">`, plus the running indicator. Counters → a row of small `.card`s (a simple stat: big number + muted label) instead of a `<ul>`. Use existing data (`counts.found/awaiting_approval/submitted/pending`). Errors keep `style={{color:"var(--danger)"}}` or a `.hint` with danger color.

- [ ] **Step 2: Vagas — cards + status context**

In `src/screens/Jobs.tsx`, wrap each review item in `.card`; the job title bold, company in `--text-muted`; the "ver vaga" link stays (uses `openExternal`); the `<details>` for cover letter/answers kept but styled (the `<pre>` uses `background: var(--surface-2)`, padding, radius). Buttons → `Aprovar` = `.btn .btn-primary`, `Rejeitar` = `.btn .btn-ghost`. The "Encontradas — Scan" list → `.card`s or a clean list with links. Section headers use `<h2>`.

- [ ] **Step 3: Build + commit**

Run: `npm run build` → success.
```bash
git add -A && git commit -m "style: restyle dashboard and vagas with design system"
```

---

### Task 3: Restyle Perfil, Pendências, Onboarding

**Files:**
- Modify: `src/screens/Profile.tsx`
- Modify: `src/screens/Pending.tsx`
- Modify: `src/screens/onboarding/*.tsx`, `src/screens/Onboarding.tsx`, `src/onboarding.css`, `src/profile.css`

**Interfaces:** none new.

- [ ] **Step 1: Perfil + Pendências**

Profile: convert `<label>` to `className="field"`, group sections under `<h2>`, the Save button to `.btn .btn-primary`, "Analisar com Claude" to `.btn`. Status message uses `.hint`. Reduce `profile.css` to only what's not covered by `theme.css`.

Pending: each pending → `.card`; the category label rendered as a `.pill` (use `labelFor`); per-question inputs use `.field`; buttons `.btn .btn-primary` / `.btn-ghost`.

- [ ] **Step 2: Onboarding**

Apply tokens to `onboarding.css` (use `var(--surface)/--border/--accent/--text` instead of hardcoded colors), step inputs use `.field`, footer buttons use `.btn`/`.btn-primary`. Keep the stepper; restyle with tokens. The step components' inputs inherit the global input styles.

- [ ] **Step 3: Build + commit**

Run: `npm run build` → success.
```bash
git add -A && git commit -m "style: restyle profile, pendencias, onboarding with design system"
```

---

### Task 4: `APPLYBOT_STATUS` marker + runner emits `agent://status`

**Files:**
- Modify: `src-tauri/src/agent/protocol.rs` (`parse_status`)
- Modify: `src-tauri/src/agent/runner.rs` (handle status in reader)
- Modify: `src-tauri/src/agent/system_prompt.md` (status rule)

**Interfaces:**
- `protocol::parse_status(line: &str) -> Option<String>` — returns the trimmed text after `APPLYBOT_STATUS `, else `None`. Not an `AgentEvent`.

- [ ] **Step 1: Parser + test**

In `src-tauri/src/agent/protocol.rs` add:
```rust
pub const STATUS: &str = "APPLYBOT_STATUS";

/// A short human-readable progress line (UI only — never persisted).
pub fn parse_status(line: &str) -> Option<String> {
    line.trim().strip_prefix(STATUS).map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}
```
Add tests:
```rust
    #[test]
    fn parses_status_text() {
        assert_eq!(parse_status("APPLYBOT_STATUS Lendo: Java Engineer @ Acme").as_deref(), Some("Lendo: Java Engineer @ Acme"));
        assert_eq!(parse_status("APPLYBOT_STATUS"), None);
        assert_eq!(parse_status("just chatter"), None);
    }
```

- [ ] **Step 2: Runner emits status**

In `src-tauri/src/agent/runner.rs`, in the reader thread loop, BEFORE the `process_line_with` call for each `text_line`, handle status:
```rust
                if let Some(status) = super::protocol::parse_status(&text_line) {
                    use tauri::Emitter;
                    let _ = app.clone().emit("agent://status", status);
                    continue;
                }
```
(Place it right after the `dbg_log("TEXT: ...")` line and before building `app2`/`process_line_with`. `continue` skips DB handling for status lines.)

- [ ] **Step 3: Prompt rule**

In `src-tauri/src/agent/system_prompt.md`, add to the reporting section (near the markers) a line:
```markdown
- Before each major step, print a SHORT progress line in Brazilian Portuguese:
  APPLYBOT_STATUS <very short pt-BR description, e.g. "Buscando vagas de backend", "Lendo: Java Engineer @ Acme", "Preparando carta", "Vaga salva">
  Keep these brief and frequent enough that the user sees steady progress. They are status only — keep using the JOB/PENDING/DONE markers for actual results.
```

- [ ] **Step 4: Tests + build**

Run: `cd src-tauri && cargo test agent::protocol && cargo build`
Expected: PASS + clean build.

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "feat: APPLYBOT_STATUS marker streamed to the UI as agent://status"
```

---

### Task 5: Dashboard live activity feed

**Files:**
- Modify: `src/lib/api.ts` (`onAgentStatus` helper)
- Modify: `src/screens/Dashboard.tsx`

**Interfaces:**
- `onAgentStatus(cb: (text: string) => void)` — subscribes to `agent://status`.

- [ ] **Step 1: API helper**

In `src/lib/api.ts` add (next to `onAgentEvent`):
```ts
export function onAgentStatus(cb: (text: string) => void) {
  return listen<string>("agent://status", (e) => cb(e.payload));
}
```

- [ ] **Step 2: Feed in the Dashboard**

In `src/screens/Dashboard.tsx`:
- Add `const [feed, setFeed] = useState<string[]>([]);`
- In the existing `useEffect`, also subscribe: `const uns = onAgentStatus((t) => setFeed((f) => [...f, t].slice(-50)));` and unsubscribe in cleanup alongside the existing listener.
- In `start()`, clear the feed: `setFeed([]);` before calling `startSearchBatch`.
- Render the feed in a `.card` below the counters, only when non-empty:
```tsx
      {feed.length > 0 && (
        <div className="card">
          <h2>Atividade</h2>
          <div style={{ maxHeight: 200, overflowY: "auto", display: "flex", flexDirection: "column", gap: 4, fontSize: 13 }}>
            {feed.map((line, i) => (
              <div key={i} style={{ color: "var(--text-muted)" }}>
                <span style={{ color: "var(--accent)" }}>›</span> {line}
              </div>
            ))}
          </div>
        </div>
      )}
```

- [ ] **Step 3: Build + commit**

Run: `npm run build` → success.
```bash
git add -A && git commit -m "feat: live agent activity feed on the dashboard"
```

---

### Task 6: Manual validation

- [ ] **Step 1: Theme + feed end-to-end**

Run `npm run tauri dev`. Verify:
1. App opens themed (dark by default); every screen looks cohesive (cards, buttons, pills, inputs). Toggle ☀️/🌙 — all screens switch cleanly; reopen the app → the chosen theme persists.
2. **Painel** → Iniciar (Scan, batch 2). The **Atividade** feed fills with short pt-BR lines as the agent works ("Buscando…", "Lendo: …", "Vaga salva"). A new run clears the feed first.
3. Pills show correct colors per status on Vagas.

Record what looked off (contrast, spacing, any status not styled) for a quick follow-up. This step gates the plan.

---

## Plan Self-Review

- **Spec coverage:** light/dark tokens + persisted toggle (Task 1) ✓; cohesive restyle of all screens with cards/buttons/inputs/pills (Tasks 2-3) ✓; `APPLYBOT_STATUS` marker, UI-only, never persisted (Task 4) ✓; prompt instructs short pt-BR status lines (Task 4) ✓; Dashboard live feed that clears each run (Task 5) ✓; pt-BR UI / English code ✓; no new deps ✓.
- **Placeholder scan:** No TBD. Restyle tasks give concrete class-application instructions against the `theme.css` classes defined in Task 1 (the components are fully specified there).
- **Type consistency:** `Theme` type shared from `theme.ts`; `onAgentStatus` mirrors `onAgentEvent`. `parse_status` returns `Option<String>`, handled in the reader before `process_line_with` (no DB path). Status events use a distinct channel `agent://status` (vs `agent://event`), so the existing counter-refresh listener is unaffected.

## Hand-off

After this, resume **Plan 6** (submission + Auto mode). The activity feed will also make the submission run more legible.
