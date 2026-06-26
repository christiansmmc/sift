# Security Policy

## Supported versions

Sift is in early development (pre-1.0). Only the latest release on `master`
receives security fixes.

## Reporting a vulnerability

Please **do not** open a public issue for security problems.

Report vulnerabilities privately via one of:

- GitHub's [private vulnerability reporting](https://github.com/christiansmmc/sift/security/advisories/new)
  (Security → Report a vulnerability)
- Email: **csequeira153@gmail.com**

Include steps to reproduce, affected version, and impact if known. You can
expect an initial response within a few days. Please give a reasonable window
to address the issue before any public disclosure.

## Scope notes

Sift runs locally and drives your own browser session via the Claude in Chrome
extension and the Claude Code CLI. It stores data in a local SQLite database and
does not transmit your credentials. Reports about how Sift handles local data,
the spawned CLI process, or the automation it performs are in scope.
