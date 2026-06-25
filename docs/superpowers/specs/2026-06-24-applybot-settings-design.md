# applybot — Settings Design: Configurações tab + cover-letter style

**Date:** 2026-06-24
**Status:** Approved (brainstorming)

## Summary

A new **Configurações** tab (5th nav screen), extensible for future settings. Its first
setting controls **how the agent writes the cover letter** — a dropdown with three presets
plus a custom free-text option — replacing today's single hardcoded prompt rule.

## Storage

A key-value `settings(key TEXT PRIMARY KEY, value TEXT NOT NULL)` table in SQLite (Rust-owned,
like everything else). Simple and extensible for future configs. Keys used now:
- `cover_letter_style` ∈ `short` | `balanced` | `detailed` | `custom` (default `balanced`)
- `cover_letter_custom` (free text; used only when style is `custom`)

## Cover-letter style

The Configurações screen shows a dropdown (pt-BR labels) and, only for "Personalizada", a
textarea for the user's own instructions:
- **Curta e simples** (`short`) — 2 short paragraphs, first person, casual-but-professional, as if the candidate wrote it quickly; no clichés, no formal template.
- **Equilibrada** (`balanced`, default) — 3 short paragraphs, specific to the company, one quantified proof.
- **Detalhada** (`detailed`) — the current behavior: 4 paragraphs, formal, quantified proofs, no clichés.
- **Personalizada** (`custom`) — the user's free-text instructions, used verbatim.

## How it reaches generation

Today the cover-letter rule (Rule 4) is hardcoded in `system_prompt.md`. It becomes a
`{{COVER_LETTER_STYLE}}` placeholder. `runner::start` loads the two settings and computes the
style instruction (`prompt::cover_letter_instruction(style, custom)`), passing it to
`build_system_prompt`, which fills the placeholder. So the chosen style applies to all future
Revisar runs. Existing queued cover letters keep their text (reject + re-run to regenerate).

## Scope

- Global setting (applies to all searches). Applies to future generations only.
- Does not touch submission. The Configurações tab is built to hold more settings later.

## Testing

- Rust: `settings` store (get/set/default) unit tests; `cover_letter_instruction` returns the right text per style and uses the custom text for `custom`; `build_system_prompt` fills `{{COVER_LETTER_STYLE}}` (no leftover placeholder).
- Frontend: manual (open Configurações, change style, run a Revisar batch, confirm the letter style changed).
