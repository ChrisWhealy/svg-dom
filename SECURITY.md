# Security Policy

Thank you for helping keep `svg-dom` and its users safe.

## Supported versions

`svg-dom` is pre-1.0 and follows the Cargo convention that `0.y.z` releases may contain breaking changes between minor (`y`) versions.
Security fixes are issued **only against the latest published `0.1.x` release** on [crates.io](https://crates.io/crates/svg-dom); there are no long-term support branches.
If you are pinned to an older version, the fix is to upgrade.

| Version  | Supported
|---|---|
| latest `0.1.x` | :white_check_mark:
| anything older | :x:

## Reporting a vulnerability

**Please do not report security vulnerabilities through public GitHub issues, pull requests, or discussions.**
Public disclosure before a fix is available puts every user of the crate at risk.

Instead, use one of the following private channels:

1. **GitHub private vulnerability reporting (preferred).** Open a report from the [**Security** tab](https://github.com/ChrisWhealy/svg-dom/security/advisories/new) of the repository.
   This keeps the discussion private and allows a fix and an advisory to be coordinated in one place.
2. **Email.** If you cannot use GitHub, write to **chris@whealy.com** with a subject line beginning `[svg-dom security]`.

To help triage a problem quickly, please include as much of the following as you can:

- the affected version(s) and target (e.g. `wasm32-unknown-unknown` in which browser);
- a description of the issue and its impact;
- the steps, proof-of-concept, or sufficient code needed to reproduce the problem;
- any suggested remediation, if you have one.

## What to expect

- **Acknowledgement** of your report within **5 working days**.
- An initial **assessment** (severity and whether we can reproduce it) within **10 working days**.
- Regular updates on progress while we work on a fix.
- **Coordinated disclosure:** once a fix is released, we will publish a [RustSec advisory](https://rustsec.org/) and/or a GitHub Security Advisory and credit you, unless you prefer to remain anonymous.

We ask that you give us a reasonable opportunity to release a fix before any public disclosure.

## Scope

This policy covers the `svg-dom` library crate itself.
Note that:

- The crate contains **no `unsafe` code** in its library build (enforced with `#![forbid(unsafe_code)]`), and its dependency tree is audited in CI with [`cargo-deny`](https://github.com/EmbarkStudios/cargo-deny) (advisories, licenses, bans, and sources).
- Vulnerabilities in **upstream dependencies** (e.g. `wasm-bindgen`, `web-sys`, `js-sys`) should be reported to those projects; if one affects `svg-dom` users, we will respond by updating the affected dependency.
- The `demo/` gallery and `demo-server/` are **examples**, not part of the published library, and are out of scope for this policy.

Thank you for your commitment to responsible disclosure.
