# Refatoração de UI/UX — Applybot

**Data:** 2026-06-25
**Tipo:** Restyle visual (sem mudança de comportamento)
**Fonte do design:** `design_handoff_applybot/` (mockup `Applybot.dc.html`, wizard `Applybot Setup.dc.html`, spec `README.md`, runtime `support.js`)

## Objetivo

Aplicar o novo design system gerado pelo Claude design a todo o app, mantendo a
lógica funcional intocada. É um re-skin: cada tela conserva suas props, handlers
e chamadas de API; muda apenas o JSX/CSS. O app é Tauri v2 + React 19, hoje com
um único `src/theme.css` (~92 linhas) de classes semânticas e sem framework de
estilo.

## Escopo

**Dentro:**
- Novo sistema de tokens (cores, tipografia, sombras, raios) — temas claro e escuro.
- Titlebar custom (decorações nativas desligadas, controles de janela próprios).
- Sidebar nova (item ativo com barra de acento; toggle de tema vira switch).
- Restyle das 5 telas principais: Painel, Vagas, Pendências, Perfil, Config.
- Restyle do onboarding (wizard de 3 passos + conclusão), conforme `Applybot Setup.dc.html`.

**Fora (decidido com o usuário):**
- Modo "Automático (enviar direto)": **não** implementar. O `<select>` de modo
  mostra apenas os 2 modos atuais (Revisar / Apenas buscar). Nenhuma lógica de
  envio automático é adicionada.
- Qualquer nova funcionalidade de backend. As Configurações atuais já contêm os
  3 cards do design (estilo da carta, seguir empresa, modelo do agente) — mapeamento 1:1.

## Decisões

1. **Abordagem A (aprovada):** estender o padrão atual — CSS global por tokens +
   ajuste de JSX, sem framework novo, sem dependências de estilo. O CSS cresce e é
   dividido por responsabilidade (ver Arquitetura de CSS).
2. **Titlebar custom: sim.**
3. **Modo Automático: não** (apenas 2 modos, restyle).
4. **Onboarding: incluído** (agora tem design próprio).
5. **Tema default: claro.** Hoje o app abre em escuro; inverter para bater com o design.

## Arquitetura de CSS

Substituir `src/theme.css` por uma pasta `src/styles/` importada num agregador:

- `src/styles/tokens.css` — `:root[data-theme="light"|"dark"]` com as variáveis; base (`body`, tipografia, scrollbars).
- `src/styles/chrome.css` — titlebar e sidebar.
- `src/styles/components.css` — `.card`, `.btn`/variações, `.pill`/status, `.field`, inputs, `.switch`.
- `src/styles/screens.css` — ajustes específicos de tela e do wizard.
- `src/styles/index.css` — `@import` dos arquivos acima; é o que `main.tsx` importa.

Valores exatos de espaçamento/sombra/tipografia que não estão tabelados abaixo
são extraídos do `support.js`/`README.md` durante a implementação, com o protótipo
aberto lado a lado para conferência.

## Tokens — valores exatos (de `support.js` → `THEMES`)

Default = `light`. Tipografia: **Geist** (chrome 13.5px/400), fallback `system-ui, -apple-system, sans-serif`.

| token | claro | escuro |
|---|---|---|
| `--backdrop` | `#e7e8ec` | `#06080b` |
| `--bg` | `#f2f3f6` | `#0f141a` |
| `--titlebar` | `#ffffff` | `#0b0f14` |
| `--titlebar-text` | `#1b1e26` | `#e8eaef` |
| `--sidebar` | `#ffffff` | `#11161d` |
| `--surface` | `#ffffff` | `#161c24` |
| `--surface-2` | `#f4f5f8` | `#1b222c` |
| `--input` | `#ffffff` | `#1b222c` |
| `--border` | `#e6e8ee` | `#252d39` |
| `--border-strong` | `#d7dae2` | `#323b49` |
| `--text` | `#1a1d26` | `#e9ebf0` |
| `--text-muted` | `#5a616e` | `#98a0ad` |
| `--text-faint` | `#8b919d` | `#697080` |
| `--accent` | `#5b54e6` | `#8079f7` |
| `--ok` | `#15a05a` | `#3ecf8e` |
| `--warn` | `#c47d08` | `#f0b357` |
| `--info` | `#2f74e0` | `#5b9cf0` |
| `--danger` | `#d8453b` | `#f06d63` |

- **Accent de marca:** índigo `--accent`, com gradiente para ciano `#22d3ee` no ícone/logo.
- **Status pills** passam a derivar de `--warn` (awaiting), `--ok` (approved/submitted),
  `--info`, `--danger` (discarded), `--accent` — substituindo as cores hardcoded de hoje.

## App chrome

### Titlebar (`src/Titlebar.tsx`, novo; ~32px de altura)
- `tauri.conf.json`: janela com `"decorations": false`.
- Região arrastável via `data-tauri-drag-region`.
- Esquerda: ícone gradiente índigo→ciano + wordmark "applybot".
- Direita: botões minimizar/maximizar/fechar ligados a `getCurrentWindow()`
  (`@tauri-apps/api/window`): `minimize()`, `toggleMaximize()`, `close()`.
- Capability: adicionar as permissões `core:window:allow-minimize`,
  `core:window:allow-toggle-maximize`, `core:window:allow-close`,
  `core:window:allow-start-dragging` (ajustar à capability existente em `src-tauri/capabilities/`).

### Sidebar (em `App.tsx` / CSS)
- Item de nav ativo com barra de acento índigo à esquerda; hover suave.
- Wordmark no topo.
- Rodapé: toggle de tema como **switch 40×22** (substitui o botão atual), persistindo a escolha como hoje.

### Layout (`App.tsx`)
- Estrutura `titlebar` (topo) + `body` (`sidebar` + `content`).

## Mapa de telas (lógica intocada)

| Tela | Arquivo | O que muda |
|---|---|---|
| Painel | `src/screens/Dashboard.tsx` | Cards de contagem, controles de run, modo (2 opções), estado "Enviando", feed — novo visual. |
| Vagas | `src/screens/Jobs.tsx` | Cards/abas/pills/cartas — novo visual; mesma lógica de revisão/aprovação. |
| Pendências | `src/screens/Pending.tsx` | Cards/itens — novo visual. |
| Perfil | `src/screens/Profile.tsx` | Cards/inputs — novo visual. |
| Config | `src/screens/Settings.tsx` | 3 cards (estilo da carta, candidatura, agente) — novo visual; conteúdo idêntico. |
| Onboarding | `src/screens/Onboarding.tsx` + `StepCv`/`StepPersonal`/`StepCriteria` | Card central (~600px), stepper de 3 passos, tela de conclusão — conforme `Applybot Setup.dc.html`. |

## Verificação

- `npm run tauri dev` e comparação visual de cada tela com o mockup (protótipo aberto lado a lado).
- `tsc` / build sem erros.
- Não há testes de UI no projeto; a verificação é visual + build limpo. Conferir
  os dois temas (claro/escuro) e os controles da titlebar (minimizar/maximizar/fechar/arrastar).

## Arquivos tocados (resumo)

- Novos: `src/Titlebar.tsx`, `src/styles/{index,tokens,chrome,components,screens}.css`.
- Editados: `src/App.tsx`, `src/main.tsx` (import do CSS), as 6 telas + 3 steps,
  `src-tauri/tauri.conf.json` (decorations), capability do `src-tauri/capabilities/`.
- Removido: `src/theme.css` (absorvido por `src/styles/`).
