# applybot — Polish Design: Theming + Live Agent Activity

**Date:** 2026-06-24
**Status:** Approved (brainstorming)

## Summary

Two improvements before Plan 6 (submission), to make the app feel like a product and show the agent is working:

1. **UI/UX with light/dark theme** — a CSS-variable design system with a persisted light/dark toggle, and a cohesive restyle of all screens (sidebar, cards, buttons, inputs, status pills). No UI framework — hand-rolled CSS variables, keeping the codebase lightweight.
2. **Live agent activity feed** — the agent emits short pt-BR status lines via a new `APPLYBOT_STATUS` marker; the Dashboard shows a scrolling feed of them during a run (ephemeral, clears each run).

## Design principles

- Code/identifiers/comments in English; all UI strings in pt-BR.
- Lightweight: no new UI dependency; theming via CSS custom properties.
- The activity feed is UI-only — it never touches the DB.

## Feature 1 — Theming + restyle

- **Theme tokens** as CSS variables on `:root` and overridden under `[data-theme="dark"]` / `[data-theme="light"]`: `--bg`, `--surface`, `--surface-2`, `--text`, `--text-muted`, `--border`, `--accent` (`#D97757`), `--accent-hover`, `--danger`, `--success`, plus radius/shadow tokens.
- **Toggle** in the sidebar (☀️/🌙) flips `document.documentElement.dataset.theme`, persisted to `localStorage` (`applybot-theme`), loaded on boot (default: dark). A tiny `src/lib/theme.ts` helper handles get/set/init.
- **Restyle** every screen to use the tokens + consistent components: a branded sidebar header ("applybot"), `.card`, `.btn` / `.btn-primary` / `.btn-ghost`, styled `input`/`select`/`textarea`, heading hierarchy, and **status pills** (`.pill` with per-status color classes — e.g. awaiting_approval, approved, submitted, discarded, analyzed). Inline styles in the screens are replaced by classes. One central stylesheet (`src/theme.css`) holds tokens + shared components; screen-specific CSS stays minimal.

## Feature 2 — Live agent activity feed

- **Protocol:** a new marker `APPLYBOT_STATUS <short text>` (bare pt-BR text after the marker, not JSON). `protocol::parse_status(line) -> Option<String>` returns the text. It is NOT an `AgentEvent` and never reaches the DB sink.
- **Prompt:** add a rule instructing the agent to emit `APPLYBOT_STATUS` with a short pt-BR description before each major step (searching, opening a job, reading questions, preparing the letter, saving), and to keep them brief.
- **Runner:** in the reader thread, before marker/DB handling, check `parse_status`; on a hit, emit a Tauri event `agent://status` with the text and continue (no DB write).
- **Dashboard:** subscribe to `agent://status`, accumulate into a capped scrolling feed (e.g. last 50 lines, newest at bottom), and clear it when a new run starts (on `Iniciar`). The feed renders below the controls/counters.

## Out of scope

Submission and Auto mode (Plan 6). No persistence of the activity log (ephemeral by decision).

## Testing

- Rust: `parse_status` unit tests (recognizes the marker, ignores others, trims text).
- Frontend + theming + feed: manual validation (run the app, toggle theme, run a batch, watch the feed).
