# Contributing to Sift

Thanks for your interest in contributing! Sift is a Tauri v2 desktop app
(React 19 + TypeScript frontend, Rust backend). This guide covers how to get
set up and how to send changes.

## Prerequisites

- [Node.js](https://nodejs.org) (v20 or newer)
- The [Rust toolchain](https://rustup.rs)
- [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS

> **Note:** Sift is primarily developed and tested on **Windows**. It drives a
> Chrome window via the Claude in Chrome extension and the Claude Code CLI — see
> the [Requirements section in the README](README.md#requirements) for the full
> runtime setup needed to actually run an end-to-end job.

## Getting started

```bash
git clone https://github.com/christiansmmc/sift.git
cd sift
npm install
npm run tauri dev      # run in development
```

## Project layout

| Path            | What lives there                                        |
|-----------------|---------------------------------------------------------|
| `src/`          | React + TypeScript frontend                             |
| `src-tauri/`    | Rust backend (Tauri commands, SQLite, agent, CV parse)  |
| `public/`, `docs/`, `brand/` | Static assets, landing page, brand assets  |

## Development workflow

1. **Fork** the repo and create a branch off `master`:
   `git checkout -b feat/my-change`
2. Make your change.
3. Verify it builds and the checks below pass locally.
4. Open a pull request against `master`.

### Checks

Before opening a PR, make sure these pass (the same checks run in CI):

```bash
npm run build                 # frontend typecheck + build
cd src-tauri && cargo build   # Rust build
cd src-tauri && cargo test    # Rust tests
```

If you touch Rust code, please also run `cargo fmt` and `cargo clippy` and
address what you reasonably can — these aren't enforced in CI yet, but keeping
new code clean is appreciated.

## Commit messages

This project uses [Conventional Commits](https://www.conventionalcommits.org/).
Write the subject and body in **English**. Examples:

```
feat: add resume-from-checkpoint to the job runner
fix: stop console window flashing on Windows
docs: clarify Chrome extension setup
```

Common types: `feat`, `fix`, `refactor`, `docs`, `chore`, `test`.

## Pull requests

- Keep PRs focused — one logical change per PR is easier to review.
- Fill out the PR template (what changed, why, how you tested it).
- Link any related issue (`Closes #123`).
- Expect review feedback; small follow-up commits are fine.

## Reporting bugs & requesting features

Use the [issue templates](https://github.com/christiansmmc/sift/issues/new/choose).
For bugs, include your OS, how you ran Sift, and the steps to reproduce.

## License

By contributing, you agree that your contributions will be licensed under the
[MIT License](LICENSE) that covers this project.
