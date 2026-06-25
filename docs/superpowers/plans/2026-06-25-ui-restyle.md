# UI/UX Restyle — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Re-skin the entire Applybot UI to the new design system from `design_handoff_applybot/`, with zero change to behavior or backend.

**Architecture:** Approach A (approved): extend the existing no-framework CSS pattern. Replace the single `src/theme.css` with a token-driven `src/styles/` folder, add a custom titlebar (native decorations off), and update each screen's JSX/classes. All props, handlers and API calls stay identical.

**Tech Stack:** Tauri v2, React 19, TypeScript, plain CSS (no styling framework, no new deps). Window controls via `@tauri-apps/api/window` (already a dependency, `^2`).

## Global Constraints

- **No behavior changes.** Only JSX structure/classNames and CSS change. Do not touch backend (`src-tauri/src/**`), `src/lib/api.ts`, or any state/handler logic in components.
- **No new dependencies.** Use only what's in `package.json` today.
- **Mode selector stays 2 options** (Revisar / Apenas buscar). Do NOT add an "Automático" mode.
- **Default theme = light** (`data-theme="light"`). The app currently defaults to dark; this inverts it.
- **Brand accent = indigo** `--accent` (`#5b54e6` light / `#8079f7` dark), brand icon gradient indigo→cyan `#22d3ee`. Terracotta `#D97757` is fully removed.
- **Source of visual truth:** the in-repo prototype `design_handoff_applybot/Applybot.dc.html` (main app), `design_handoff_applybot/Applybot Setup.dc.html` (wizard), `design_handoff_applybot/support.js` (runtime CSS/markup), `design_handoff_applybot/README.md` (spec notes). When a pixel/spacing/shadow value is not pinned in this plan, read it from these files and match it.
- **Verification per task** (no unit tests exist for UI): `npm run build` (or `npx tsc --noEmit`) is clean, AND the screen visually matches the prototype when run via `npm run tauri dev`, in BOTH themes. Then commit.

---

## File Structure

**New files:**
- `src/styles/index.css` — aggregator; `@import`s the others. Imported once by `src/main.tsx`.
- `src/styles/tokens.css` — `:root[data-theme=...]` variables + base (`body`, headings, links, scrollbars).
- `src/styles/chrome.css` — titlebar + sidebar.
- `src/styles/components.css` — `.card`, `.btn` variants, `.pill`/status, `.field`, inputs, `.switch`.
- `src/styles/screens.css` — screen-specific and wizard-specific rules.
- `src/Titlebar.tsx` — custom window titlebar component.

**Modified files:**
- `src/main.tsx` — import `./styles/index.css` instead of `./theme.css`.
- `src/App.tsx` — layout (`titlebar` + `body[sidebar+content]`), sidebar restyle, theme switch, default theme = light.
- `src/screens/Dashboard.tsx`, `Jobs.tsx`, `Pending.tsx`, `Profile.tsx`, `Settings.tsx` — JSX/class restyle.
- `src/screens/onboarding/Onboarding.tsx`, `StepCv.tsx`, `StepPersonal.tsx`, `StepCriteria.tsx` — wizard restyle.
- `src-tauri/tauri.conf.json` — `"decorations": false` on the main window.
- `src-tauri/capabilities/default.json` — add `core:window:*` permissions.

**Deleted files:**
- `src/theme.css` — absorbed by `src/styles/`.
- `src/App.css` — legacy/dead (not imported anywhere; duplicates body reset + old terracotta `#D97757` sidebar). Removed in Task 4.

**Other pre-existing CSS to reconcile (not in original inventory):**
- `src/profile.css` (imported by `Profile.tsx`) — handled in Task 8.
- `src/onboarding.css` (imported by `Onboarding.tsx`; uses undefined `var(--success)`) — handled in Task 10.

---

## Task 1: Token foundation & CSS structure

**Files:**
- Create: `src/styles/tokens.css`, `src/styles/index.css`
- Modify: `src/main.tsx` (CSS import), `src/App.tsx` (default `data-theme` → light, if set there)
- Delete: `src/theme.css`

**Interfaces:**
- Produces: CSS custom properties available app-wide; `src/styles/index.css` as the single CSS entry point. Token names: `--backdrop --bg --titlebar --titlebar-text --sidebar --surface --surface-2 --input --border --border-strong --text --text-muted --text-faint --accent --ok --warn --info --danger`.

- [ ] **Step 1: Find where the current default theme is set.** Read `src/main.tsx` and `src/App.tsx`. Locate where `document.documentElement.setAttribute("data-theme", ...)` (or similar) initializes the theme and where `./theme.css` is imported. Note the import line and the theme-init code.

- [ ] **Step 2: Create `src/styles/tokens.css`** with the exact token sets (these values are authoritative, from `support.js` `THEMES`):

```css
:root,
:root[data-theme="light"] {
  --backdrop:#e7e8ec; --bg:#f2f3f6; --titlebar:#ffffff; --titlebar-text:#1b1e26;
  --sidebar:#ffffff; --surface:#ffffff; --surface-2:#f4f5f8; --input:#ffffff;
  --border:#e6e8ee; --border-strong:#d7dae2;
  --text:#1a1d26; --text-muted:#5a616e; --text-faint:#8b919d;
  --accent:#5b54e6; --ok:#15a05a; --warn:#c47d08; --info:#2f74e0; --danger:#d8453b;
  --accent-grad-end:#22d3ee;
  --radius:8px;
  --shadow-card:0 1px 2px rgba(16,19,30,.04);
  --shadow-pop:0 24px 60px rgba(8,10,20,.18), 0 6px 18px rgba(8,10,20,.10);
}
:root[data-theme="dark"] {
  --backdrop:#06080b; --bg:#0f141a; --titlebar:#0b0f14; --titlebar-text:#e8eaef;
  --sidebar:#11161d; --surface:#161c24; --surface-2:#1b222c; --input:#1b222c;
  --border:#252d39; --border-strong:#323b49;
  --text:#e9ebf0; --text-muted:#98a0ad; --text-faint:#697080;
  --accent:#8079f7; --ok:#3ecf8e; --warn:#f0b357; --info:#5b9cf0; --danger:#f06d63;
  --accent-grad-end:#22d3ee;
  --radius:8px;
  --shadow-card:0 1px 2px rgba(0,0,0,.30);
  --shadow-pop:0 24px 60px rgba(8,10,20,.45), 0 6px 18px rgba(8,10,20,.25);
}

* { box-sizing: border-box; }
body {
  margin: 0;
  font-family: "Geist", system-ui, -apple-system, sans-serif;
  font-size: 13.5px; line-height: 1.6;
  background: var(--bg); color: var(--text);
}
h1 { font-size: 18px; margin: 0 0 16px; }
h2 { font-size: 13px; margin: 20px 0 8px; color: var(--text-muted); text-transform: uppercase; letter-spacing: .04em; }
a { color: var(--accent); cursor: pointer; }
.hint { color: var(--text-muted); font-size: 12px; margin: 4px 0 0; }
.loading { padding: 24px; color: var(--text-muted); }

/* Themed scrollbars (WebView2/Chromium). */
* { scrollbar-width: thin; scrollbar-color: var(--border-strong) transparent; }
::-webkit-scrollbar { width: 10px; height: 10px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: var(--border-strong); border-radius: 6px; border: 2px solid var(--bg); }
::-webkit-scrollbar-thumb:hover { background: var(--text-faint); }
::-webkit-scrollbar-corner { background: transparent; }
```

> Note: shadow/radius/typography values above are sensible defaults matching the README. During Task 2–10, when a component's prototype rule specifies a different value, prefer the prototype's.

- [ ] **Step 3: Create `src/styles/index.css`** as the aggregator (the other files are created in later tasks; add their `@import`s now so the entry point is stable):

```css
@import "./tokens.css";
@import "./chrome.css";
@import "./components.css";
@import "./screens.css";
```

- [ ] **Step 4: Create empty placeholder files** so the imports resolve until later tasks fill them: create `src/styles/chrome.css`, `src/styles/components.css`, `src/styles/screens.css`, each containing only a header comment, e.g. `/* chrome: titlebar + sidebar */`.

- [ ] **Step 5: Update `src/main.tsx`** — replace the `import "./theme.css";` line with `import "./styles/index.css";`.

- [ ] **Step 6: Set default theme to light.** In the theme-init code found in Step 1, make the default `"light"` when no saved preference exists (e.g. `localStorage.getItem("theme") ?? "light"`). Keep persistence logic unchanged.

- [ ] **Step 7: Delete `src/theme.css`.**

- [ ] **Step 8: Verify build.** Run: `npx tsc --noEmit` then `npm run build`. Expected: no errors, no missing-import errors for CSS.

- [ ] **Step 9: Verify visually.** Run `npm run tauri dev`. Expected: app launches in light theme; colors use the new palette (off-white bg, indigo accents). Layout may be rough until later tasks — that's fine.

- [ ] **Step 10: Commit.**

```bash
git add src/styles src/main.tsx src/App.tsx
git rm src/theme.css
git commit -m "refactor(ui): token foundation + styles structure, default light theme"
```

---

## Task 2: Component primitives CSS

**Files:**
- Modify: `src/styles/components.css`
- Reference: `design_handoff_applybot/support.js`, `Applybot.dc.html` (rules for cards/buttons/pills/inputs/switch)

**Interfaces:**
- Produces classes consumed by all screens: `.card`, `.btn`, `.btn-primary`, `.btn-ghost`, `.btn-danger`, `.pill` + `.pill-<status>`, `.field`, base `input/select/textarea`, `.switch` (+ `.switch.on`), `.cover-view`.

- [ ] **Step 1: Open the prototype** `design_handoff_applybot/Applybot.dc.html` and `support.js`. Locate the rules for the card surface, buttons (primary/secondary/ghost), status pills, form fields, and the toggle switch. These are the visual reference for exact padding/radius/weight.

- [ ] **Step 2: Write `src/styles/components.css`** porting those rules, adapted to our class names and tokens. Baseline (adjust pixel values to match the prototype where they differ):

```css
.card {
  background: var(--surface); border: 1px solid var(--border);
  border-radius: var(--radius); padding: 16px; margin: 12px 0;
  box-shadow: var(--shadow-card);
}
.card h2:first-child, .card h3:first-child { margin-top: 0; }
.card .hint { margin-bottom: 12px; }

.btn {
  padding: 8px 14px; border-radius: var(--radius); border: 1px solid var(--border-strong);
  background: var(--surface-2); color: var(--text); cursor: pointer; font: inherit;
  font-size: 13px; transition: filter .15s, border-color .15s, background .15s;
}
.btn:hover { filter: brightness(1.06); border-color: var(--accent); }
.btn:disabled { opacity: .5; cursor: default; }
.btn-primary { background: var(--accent); border-color: var(--accent); color: #fff; }
.btn-primary:hover { filter: brightness(1.06); }
.btn-ghost { background: transparent; }
.btn-danger { color: var(--danger); border-color: var(--danger); background: transparent; }

label.field, .field { display: flex; flex-direction: column; gap: 7px; font-size: 13.5px; margin-bottom: 14px; color: var(--text); }
input, select, textarea {
  padding: 8px 10px; border: 1px solid var(--border-strong); border-radius: var(--radius);
  background: var(--input); color: var(--text); font: inherit; font-size: 13.5px;
}
input:focus, select:focus, textarea:focus { outline: none; border-color: var(--accent); }
input:disabled { background: var(--surface-2); color: var(--text-muted); }

.pill { display: inline-block; padding: 2px 10px; border-radius: 999px; font-size: 11.5px; font-weight: 600; background: var(--surface-2); color: var(--text-muted); }
.pill-awaiting_approval { background: color-mix(in srgb, var(--warn) 16%, transparent); color: var(--warn); }
.pill-approved  { background: color-mix(in srgb, var(--ok) 16%, transparent); color: var(--ok); }
.pill-submitted { background: color-mix(in srgb, var(--info) 16%, transparent); color: var(--info); }
.pill-discarded { background: color-mix(in srgb, var(--danger) 16%, transparent); color: var(--danger); }
.pill-analyzed  { background: var(--surface-2); color: var(--text-muted); }

/* Toggle switch (theme toggle + any boolean), 40×22 per README. */
.switch { width: 40px; height: 22px; border-radius: 999px; background: var(--border-strong); border: none; position: relative; cursor: pointer; transition: background .18s; padding: 0; }
.switch::after { content: ""; position: absolute; top: 3px; left: 3px; width: 16px; height: 16px; border-radius: 50%; background: #fff; transition: left .18s; box-shadow: 0 1px 2px rgba(16,19,30,.12); }
.switch.on { background: var(--accent); }
.switch.on::after { left: 21px; }

/* Read-only cover letter view. */
.cover-view { white-space: pre-wrap; background: var(--surface-2); border: 1px solid var(--border); border-radius: var(--radius); padding: 12px; max-height: 220px; overflow-y: auto; font-size: 13px; line-height: 1.5; }
```

- [ ] **Step 3: Verify build.** Run: `npx tsc --noEmit`. Expected: no errors (CSS-only change).

- [ ] **Step 4: Verify visually.** Run `npm run tauri dev`. Open any screen with cards/buttons/pills (e.g. Vagas, Config). Expected: cards, buttons, pills and inputs match the prototype styling in both themes. Toggle the theme to check dark.

- [ ] **Step 5: Commit.**

```bash
git add src/styles/components.css
git commit -m "feat(ui): component primitives (card, btn, pill, field, switch)"
```

---

## Task 3: Custom titlebar + Tauri window config

**Files:**
- Create: `src/Titlebar.tsx`
- Modify: `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`, `src/App.tsx` (mount titlebar + layout), `src/styles/chrome.css` (titlebar rules)
- Reference: `design_handoff_applybot/Applybot.dc.html` titlebar markup/styles

**Interfaces:**
- Consumes: tokens `--titlebar`, `--titlebar-text`, `--accent`, `--accent-grad-end`.
- Produces: `<Titlebar />` default export; app body wrapped below a 32px titlebar.

- [ ] **Step 1: Disable native decorations.** In `src-tauri/tauri.conf.json`, find the `app.windows[0]` object (label `"main"`) and add `"decorations": false`. Keep all other window fields unchanged.

- [ ] **Step 2: Grant window permissions.** In `src-tauri/capabilities/default.json`, add to the `permissions` array: `"core:window:allow-minimize"`, `"core:window:allow-toggle-maximize"`, `"core:window:allow-close"`, `"core:window:allow-start-dragging"`. Resulting array:

```json
"permissions": [
  "core:default",
  "opener:default",
  "dialog:default",
  "dialog:allow-open",
  "core:window:allow-minimize",
  "core:window:allow-toggle-maximize",
  "core:window:allow-close",
  "core:window:allow-start-dragging"
]
```

- [ ] **Step 3: Create `src/Titlebar.tsx`:**

```tsx
import { getCurrentWindow } from "@tauri-apps/api/window";

const win = getCurrentWindow();

export default function Titlebar() {
  return (
    <div className="titlebar" data-tauri-drag-region>
      <div className="titlebar-brand">
        <span className="titlebar-logo" aria-hidden />
        <span className="titlebar-word">applybot</span>
      </div>
      <div className="titlebar-controls">
        <button className="tb-btn" onClick={() => win.minimize()} aria-label="Minimizar">─</button>
        <button className="tb-btn" onClick={() => win.toggleMaximize()} aria-label="Maximizar">▢</button>
        <button className="tb-btn tb-close" onClick={() => win.close()} aria-label="Fechar">✕</button>
      </div>
    </div>
  );
}
```

- [ ] **Step 4: Add titlebar styles to `src/styles/chrome.css`** (match the prototype; baseline below):

```css
.titlebar { height: 32px; display: flex; align-items: center; justify-content: space-between; background: var(--titlebar); color: var(--titlebar-text); border-bottom: 1px solid var(--border); user-select: none; -webkit-user-select: none; }
.titlebar-brand { display: flex; align-items: center; gap: 8px; padding-left: 12px; pointer-events: none; }
.titlebar-logo { width: 16px; height: 16px; border-radius: 5px; background: linear-gradient(135deg, var(--accent), var(--accent-grad-end)); }
.titlebar-word { font-size: 12.5px; font-weight: 600; }
.titlebar-controls { display: flex; height: 100%; }
.tb-btn { width: 46px; height: 100%; border: none; background: transparent; color: var(--titlebar-text); cursor: pointer; font-size: 11px; display: grid; place-items: center; }
.tb-btn:hover { background: var(--surface-2); }
.tb-close:hover { background: var(--danger); color: #fff; }
```

- [ ] **Step 5: Mount the titlebar in `src/App.tsx`.** Wrap the app so the titlebar sits above the existing `.app` row. Read the current `App.tsx` return first; change the outermost structure to:

```tsx
<div className="root">
  <Titlebar />
  <div className="app">
    {/* existing sidebar + content unchanged */}
  </div>
</div>
```

Add `import Titlebar from "./Titlebar";` at the top. Add to `src/styles/chrome.css`: `.root { display: flex; flex-direction: column; height: 100vh; }` and change the existing `.app`/`.content` max-height usage so the content area fills the remaining height below the 32px bar (e.g. `.app { flex: 1; min-height: 0; } .content { max-height: none; }`).

- [ ] **Step 6: Verify build.** Run: `npx tsc --noEmit` (catches the `@tauri-apps/api/window` import + JSX). Expected: no errors.

- [ ] **Step 7: Verify visually + functionally.** Run `npm run tauri dev`. Expected: native Windows title bar is gone; custom 32px bar shows the gradient logo + "applybot" on the left and min/max/close on the right. Dragging the bar moves the window; the three buttons minimize/maximize/close work; close button hover turns red.

- [ ] **Step 8: Commit.**

```bash
git add src/Titlebar.tsx src/App.tsx src/styles/chrome.css src-tauri/tauri.conf.json src-tauri/capabilities/default.json
git commit -m "feat(ui): custom titlebar with window controls"
```

---

## Task 4: Sidebar restyle + theme switch

**Files:**
- Modify: `src/App.tsx` (sidebar markup + theme toggle as switch), `src/styles/chrome.css` (sidebar rules)
- Delete: `src/App.css` (legacy, not imported — dead terracotta sidebar rules)
- Reference: `design_handoff_applybot/Applybot.dc.html` sidebar markup/styles

**Interfaces:**
- Consumes: theme state/handler already in `App.tsx`; nav state already in `App.tsx`.
- Produces: restyled `.sidebar` with `.navlink.active` accent bar and a `.switch` theme toggle.

- [ ] **Step 1: Read current `src/App.tsx` sidebar block** — the brand, the `nav` with `button.navlink` items, and the existing `.theme-toggle` button. Note the active-tab state variable and the theme toggle handler.

- [ ] **Step 2: Replace the theme-toggle button with a switch.** Keep the same onClick handler; change markup to:

```tsx
<button
  className={`switch ${theme === "light" ? "" : "on"}`}
  onClick={toggleTheme}
  aria-label="Alternar tema"
/>
```

(Use the actual theme variable/handler names from Step 1. The switch is "on" in dark mode — adjust the condition to taste so the visual matches the prototype.) Wrap it with a small label row if the prototype shows a "Tema" label.

- [ ] **Step 3: Add sidebar styles to `src/styles/chrome.css`** (match the prototype; baseline below):

```css
.sidebar { width: 208px; background: var(--sidebar); border-right: 1px solid var(--border); display: flex; flex-direction: column; padding: 14px 12px; gap: 4px; }
.sidebar .brand { font-weight: 700; font-size: 15px; color: var(--text); padding: 4px 12px 14px; }
.sidebar .brand span { color: var(--accent); }
.sidebar nav { display: flex; flex-direction: column; gap: 2px; flex: 1; }
.sidebar button.navlink { position: relative; background: transparent; color: var(--text-muted); border: none; text-align: left; padding: 9px 12px; border-radius: var(--radius); cursor: pointer; font-size: 13.5px; }
.sidebar button.navlink:hover { background: var(--surface-2); color: var(--text); }
.sidebar button.navlink.active { background: var(--surface-2); color: var(--text); font-weight: 600; }
.sidebar button.navlink.active::before { content: ""; position: absolute; left: -12px; top: 7px; bottom: 7px; width: 3px; border-radius: 0 3px 3px 0; background: var(--accent); }
.sidebar .theme-row { display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; color: var(--text-muted); font-size: 12.5px; }
```

(If the prototype shows a count badge on a nav item — e.g. Pendências — port that too, reading the markup.)

- [ ] **Step 4: Delete the dead `src/App.css`.** Confirm it is not imported anywhere (`git grep -n "App.css" src` returns nothing), then `git rm src/App.css`. It holds only legacy `body`/`.sidebar` rules with the old terracotta `#D97757` and is overridden/unused.

- [ ] **Step 5: Verify build.** Run: `npx tsc --noEmit`. Expected: no errors.

- [ ] **Step 6: Verify visually.** Run `npm run tauri dev`. Expected: sidebar matches the prototype — active item has the indigo accent bar, hover states work, and the theme toggle is a 40×22 switch that flips the theme. Check both themes.

- [ ] **Step 7: Commit.**

```bash
git add src/App.tsx src/styles/chrome.css
git rm src/App.css
git commit -m "feat(ui): sidebar restyle + theme switch; remove dead App.css"
```

---

## Task 5: Painel (Dashboard) restyle

**Files:**
- Modify: `src/screens/Dashboard.tsx`, `src/styles/screens.css` (if screen-specific rules needed)
- Reference: prototype Painel screen in `Applybot.dc.html` / `support.js`

**Interfaces:**
- Consumes: existing props of `Dashboard` (counts, running, runKind, mode, batch, feed, error, setMode, setBatch, onStart, onStop, approvedCount, onSubmitApproved). Do not change the props or handlers.

- [ ] **Step 1: Read current `src/screens/Dashboard.tsx`** and the prototype's Painel layout. Map each existing element (count cards, run controls, mode `<select>`, batch input, start/stop, "Enviando" state, feed list) to the prototype's markup.

- [ ] **Step 2: Restyle the JSX** to the prototype's structure using the primitives from Task 2 (`.card`, `.btn`, `.pill`) and any new screen-specific classes added to `screens.css`. **Keep the mode `<select>` at exactly 2 options** (the existing Revisar / Apenas buscar values) — do not add Automático. Keep all state bindings and handlers byte-for-byte.

- [ ] **Step 3: Add any Painel-specific CSS** to `src/styles/screens.css` (e.g. count-card grid). Match the prototype.

- [ ] **Step 4: Verify build.** Run: `npx tsc --noEmit`. Expected: no errors.

- [ ] **Step 5: Verify visually + functionally.** Run `npm run tauri dev`, open Painel. Expected: matches the prototype; starting/stopping a run still works, the "Enviando" state still appears during submit, the feed updates. Check both themes.

- [ ] **Step 6: Commit.**

```bash
git add src/screens/Dashboard.tsx src/styles/screens.css
git commit -m "feat(ui): Painel (dashboard) restyle"
```

---

## Task 6: Vagas (Jobs) restyle

**Files:**
- Modify: `src/screens/Jobs.tsx`, `src/styles/screens.css`
- Reference: prototype Vagas screen

**Interfaces:**
- Consumes: existing `Jobs` props/state (review queue, approve/discard/edit handlers, cover-letter view/edit). Do not change behavior.

- [ ] **Step 1: Read current `src/screens/Jobs.tsx`** and the prototype's Vagas layout (job cards, status pills, cover-letter read-only `.cover-view` ↔ edit `textarea`, approve/discard/edit actions, any tabs/sections).

- [ ] **Step 2: Restyle the JSX** to the prototype using Task 2 primitives. Preserve the read-only↔edit toggle for the cover letter and all action handlers exactly.

- [ ] **Step 3: Add Vagas-specific CSS** to `src/styles/screens.css` as needed. Match the prototype.

- [ ] **Step 4: Verify build.** Run: `npx tsc --noEmit`. Expected: no errors.

- [ ] **Step 5: Verify visually + functionally.** Run `npm run tauri dev`, open Vagas. Expected: matches the prototype; approve/discard/edit and the cover-letter view/edit still work. Check both themes.

- [ ] **Step 6: Commit.**

```bash
git add src/screens/Jobs.tsx src/styles/screens.css
git commit -m "feat(ui): Vagas (jobs) restyle"
```

---

## Task 7: Pendências (Pending) restyle

**Files:**
- Modify: `src/screens/Pending.tsx`, `src/styles/screens.css`
- Reference: prototype Pendências screen

**Interfaces:**
- Consumes: existing `Pending` props/state (open pending actions list, resolve handler). Do not change behavior.

- [ ] **Step 1: Read current `src/screens/Pending.tsx`** and the prototype's Pendências layout (pending-action cards, resolve action, empty state).

- [ ] **Step 2: Restyle the JSX** to the prototype using Task 2 primitives. Preserve the resolve handler and any count wiring.

- [ ] **Step 3: Add Pendências-specific CSS** to `src/styles/screens.css` as needed.

- [ ] **Step 4: Verify build.** Run: `npx tsc --noEmit`. Expected: no errors.

- [ ] **Step 5: Verify visually + functionally.** Run `npm run tauri dev`, open Pendências. Expected: matches the prototype; resolving a pending item still works and updates the count. Check both themes.

- [ ] **Step 6: Commit.**

```bash
git add src/screens/Pending.tsx src/styles/screens.css
git commit -m "feat(ui): Pendências (pending) restyle"
```

---

## Task 8: Perfil (Profile) restyle

**Files:**
- Modify: `src/screens/Profile.tsx`, `src/styles/screens.css`, `src/profile.css` (existing, imported by `Profile.tsx` — holds `.prof`/`.prof-actions` layout)
- Reference: prototype Perfil screen

**Interfaces:**
- Consumes: existing `Profile` props/state (profile fields, save handler, CV/answers display). Do not change behavior.

- [ ] **Step 1: Read current `src/screens/Profile.tsx`**, its `src/profile.css` (`.prof`, `.prof-actions`), and the prototype's Perfil layout (profile cards, fields, saved answers read-only display, save action).

- [ ] **Step 2: Restyle the JSX** to the prototype using Task 2 primitives (`.card`, `.field`, inputs, `.btn-primary`). Preserve all field bindings and the save handler.

- [ ] **Step 3: Reconcile `src/profile.css`.** Move its `.prof`/`.prof-actions` rules into `src/styles/screens.css` (so all screen CSS lives together), update the import in `Profile.tsx` (remove `import "../profile.css"`), and `git rm src/profile.css`. Add any other Perfil-specific CSS to `screens.css`. Match the prototype.

- [ ] **Step 4: Verify build.** Run: `npx tsc --noEmit`. Expected: no errors.

- [ ] **Step 5: Verify visually + functionally.** Run `npm run tauri dev`, open Perfil. Expected: matches the prototype; editing and saving profile fields still works. Check both themes.

- [ ] **Step 6: Commit.**

```bash
git add src/screens/Profile.tsx src/styles/screens.css
git commit -m "feat(ui): Perfil (profile) restyle"
```

---

## Task 9: Config (Settings) restyle

**Files:**
- Modify: `src/screens/Settings.tsx`, `src/styles/screens.css`
- Reference: prototype Config screen

**Interfaces:**
- Consumes: existing `Settings` state/handlers — `style`/`custom` (cover_letter_style/custom), `follow` (follow_company), `model` (agent_model), `save()`, `handleModelChange`. Do not change behavior. The three cards already exist and map 1:1.

- [ ] **Step 1: Read current `src/screens/Settings.tsx`** (already known: 3 cards — "Estilo da carta de apresentação", "Candidatura", "Agente") and the prototype's Config layout. Note the prototype may render the follow-company checkbox as a `.switch` and the model/style as styled selects.

- [ ] **Step 2: Restyle the JSX** to the prototype using Task 2 primitives. If the prototype shows the follow-company boolean as a switch, swap the `<input type="checkbox">` for a `.switch` button bound to the same `setFollow`/`api.setSetting("follow_company", ...)` logic. Keep `style`/`custom`/`model` selects and `save()` exactly. Keep the `style === "custom"` conditional textarea.

- [ ] **Step 3: Add Config-specific CSS** to `src/styles/screens.css` as needed.

- [ ] **Step 4: Verify build.** Run: `npx tsc --noEmit`. Expected: no errors.

- [ ] **Step 5: Verify visually + functionally.** Run `npm run tauri dev`, open Config. Expected: matches the prototype; saving cover-letter style/custom persists, follow-company toggles and persists, model select persists. Check both themes.

- [ ] **Step 6: Commit.**

```bash
git add src/screens/Settings.tsx src/styles/screens.css
git commit -m "feat(ui): Config (settings) restyle"
```

---

## Task 10: Onboarding wizard restyle

**Files:**
- Modify: `src/screens/onboarding/Onboarding.tsx`, `StepCv.tsx`, `StepPersonal.tsx`, `StepCriteria.tsx`, `src/styles/screens.css`, `src/onboarding.css` (existing, imported by `Onboarding.tsx`; holds `.onb-*` wizard layout and uses the undefined `var(--success)`)
- Reference: `design_handoff_applybot/Applybot Setup.dc.html` (the wizard design) + `support.js`

> Note current onboarding files live at `src/screens/Onboarding.tsx` and `src/screens/onboarding/Step*.tsx` (verify exact paths when reading). The import in `Onboarding.tsx` is `import "../onboarding.css"`.

**Interfaces:**
- Consumes: existing onboarding state machine (current step, next/back, per-step data + completion handler that writes the profile). Do not change the flow logic or what gets persisted.

- [ ] **Step 1: Read current onboarding files** (`Onboarding.tsx` wrapper + the 3 step components) and the wizard prototype `Applybot Setup.dc.html`. Map: centered card (~600px, `padding: 28px 40px 22px`), the 3-step stepper (Currículo → Seus dados → O que você busca), and the completion screen.

- [ ] **Step 2: Restyle the wrapper** `Onboarding.tsx` — centered card layout + stepper header indicating step 1/2/3 and the done state. Keep the step-advance/back state logic unchanged.

- [ ] **Step 3: Restyle each step** (`StepCv`, `StepPersonal`, `StepCriteria`) to the prototype using Task 2 primitives, preserving every field binding, validation, and the data each step contributes. Match the prototype's CV upload affordance (PDF/DOCX), the personal fields (name etc.), and the criteria fields.

- [ ] **Step 4: Add wizard-specific CSS** to `src/styles/screens.css` (centered card, stepper, done screen). Match the prototype. **Reconcile `src/onboarding.css`:** move its `.onb-*` rules into `screens.css` updating stale tokens (notably `var(--success)` → `var(--ok)`), remove `import "../onboarding.css"` from `Onboarding.tsx`, and `git rm src/onboarding.css`. After this task, `git grep "var(--success)" src` must return nothing.

- [ ] **Step 5: Verify build.** Run: `npx tsc --noEmit`. Expected: no errors.

- [ ] **Step 6: Verify visually + functionally.** To see onboarding, back up and remove the DB so the wizard reappears (app closed first):
  - `Rename-Item "$env:APPDATA\com.applybot.app\applybot.db" "applybot.db.bak"`
  - Run `npm run tauri dev`, walk the 3 steps to completion. Expected: matches the wizard prototype in both themes; completing it writes the profile and lands in the main app.
  - Restore: with the app closed, `Remove-Item "$env:APPDATA\com.applybot.app\applybot.db"; Rename-Item "$env:APPDATA\com.applybot.app\applybot.db.bak" "applybot.db"`.

- [ ] **Step 7: Commit.**

```bash
git add src/screens/onboarding src/styles/screens.css
git commit -m "feat(ui): onboarding wizard restyle"
```

---

## Task 11: Polish & fidelity pass + nav count badge

Gathers the deferred Minor findings from the per-task reviews plus one user-approved functional add (the nav count badge). Do this after Tasks 5–10.

**Files:**
- Modify: `src/styles/chrome.css` (titlebar fidelity + badge style + accent-bar overflow), `src/Titlebar.tsx` (logo inner dot), `src/App.tsx` (nav badge + nested-nav fix), `src/styles/components.css` (two minor cleanups)
- Reference: `design_handoff_applybot/Applybot.dc.html` / `support.js`

**Interfaces:**
- Consumes: the `counts` object already passed into `App.tsx`/`Dashboard` for the open-pending count used by the Pendências screen. Use that same field for the badge.

- [ ] **Step 1: Titlebar fidelity.** In `src/styles/chrome.css`, change `.titlebar` height `32px` → `40px`. Set the wordmark to `font-weight: 500; opacity: .72; letter-spacing: .2px` to match the prototype. In `src/Titlebar.tsx`/chrome.css, add the prototype's inner white dot inside the gradient logo (small centered white dot/diamond). Change `.tb-btn:hover` background from `var(--surface-2)` to `color-mix(in srgb, var(--text) 7%, transparent)` to match the prototype.

- [ ] **Step 2: Accent-bar robustness.** Add `overflow: visible;` explicitly to `.sidebar` so the `.navlink.active::before` accent bar can never be clipped.

- [ ] **Step 3: Nested-nav fix.** In `src/App.tsx`, the sidebar uses `<nav className="sidebar">` wrapping an inner `<nav>`. Change the inner `<nav>` to a `<div>` (keep classes/structure otherwise) so there's a single nav landmark.

- [ ] **Step 4: Nav count badge.** In `src/App.tsx`, add a count badge to the relevant nav item (Pendências) showing the open-pending count from the existing `counts` data. Render the badge only when the count > 0. Add a `.nav-badge` style to `src/styles/chrome.css` matching the prototype (small pill, Geist Mono, `min-width:17px`, height ~17px, `color-mix` background). Do not change any count-fetching logic — read the value already available.

- [ ] **Step 5: Component minor cleanups.** In `src/styles/components.css`: remove the dead `border-color: var(--accent)` in `.btn-primary` (it's reset by `border: none`); narrow `.tab-btn`'s `transition: all .12s` to `transition: background .12s, color .12s, box-shadow .12s`.

- [ ] **Step 6: Verify build.** Run: `npx tsc --noEmit` and `npm run build`. Expected: clean.

- [ ] **Step 7: Verify visually.** Run `npm run tauri dev`. Expected: titlebar is 40px with the dotted logo and lighter wordmark; nav shows the Pendências badge when there are open pendências; everything else unchanged. Check both themes.

- [ ] **Step 8: Commit.**

```bash
git add src/styles/chrome.css src/styles/components.css src/Titlebar.tsx src/App.tsx
git commit -m "polish(ui): titlebar fidelity, nav count badge, minor cleanups"
```

---

## Final verification (after Task 11)

- [ ] Run `npx tsc --noEmit` and `npm run build` — both clean.
- [ ] Run `npm run tauri dev` and walk every screen in BOTH themes, comparing against the prototype: titlebar controls, sidebar active state + theme switch, Painel run flow, Vagas approve/discard/edit, Pendências resolve, Perfil save, Config persistence, onboarding wizard.
- [ ] Confirm terracotta `#D97757` no longer appears anywhere: `git grep -i "D97757" src` returns nothing.
- [ ] Confirm `src/theme.css` is deleted and nothing imports it: `git grep "theme.css" src` returns nothing.
- [ ] Confirm legacy stylesheets are gone: `src/App.css`, `src/profile.css`, `src/onboarding.css` no longer exist and nothing imports them (`git grep -nE "App\.css|profile\.css|onboarding\.css" src` returns nothing).
- [ ] Confirm no stale tokens remain: `git grep -nE "var\(--(success|accent-hover)\)" src` returns nothing (the new system uses `--ok`, `--accent`).
