# Sift — Ícone do app

Marca do **Sift**: funil/peneira que filtra vagas, com o ponto central representando a vaga certa que passou pelo filtro.

## Arquivos
- `sift-icon.svg` — **master vetorial** (escala infinita, edite aqui se precisar).
- `sift.ico` — ícone multi-resolução para **Windows / Tauri** (16, 24, 32, 48, 64, 128, 256px num só arquivo).
- `png/sift-{16..1024}.png` — PNGs individuais (16, 24, 32, 48, 64, 128, 256, 512, 1024).

## Como usar no Tauri
**Jeito recomendado (gera todos os formatos automaticamente):**
```bash
npm run tauri icon sift-brand/png/sift-1024.png
```
Esse comando cria a pasta `src-tauri/icons/` completa (`.ico`, `.icns` pra Mac, e todos os PNGs que o Windows/Linux/Android precisam) a partir do PNG de 1024px.

**Jeito manual:** copie `sift.ico` para `src-tauri/icons/icon.ico` e ajuste o `tauri.conf.json`:
```json
"bundle": { "icon": ["icons/icon.ico", "icons/icon.png"] }
```

## Cores da marca
- Fundo do ícone: `#0f141a` (mesmo surface escuro do app)
- Funil (gradiente): `#6f67f7` → `#22d3ee`
- No app, o acento principal é `#5b54e6` (claro) / `#8079f7` (escuro)

## Favicon / web
Use `png/sift-32.png` e `png/sift-256.png`, ou referencie o `sift-icon.svg` direto.
