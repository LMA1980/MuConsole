# MuConsole Development Guidelines

> [!IMPORTANT]
> **Scope of `AGENTS.md`**: This file is reserved strictly for **immediate, universal behavioral rules and hard invariants** that govern every agent interaction. Do NOT place occasional notes, component designs, framework preferences, or operational runbooks here; those belong in repository-specific documentation (`README.md`, `docs/`).

## 1. Branching & Pull Request Workflow (feat/\*\*)

- **Feature Branch Invariant**: All changes intended for `main` are developed on a `feat/**` branch and merged via Pull Request. Never commit or push directly to `main`.
- **Signed Commits**: `commit.gpgsign` stays enabled. Never disable, override or bypass commit signing.
- **Pre-PR Gate Order**: `(cd code && cargo fmt --all -- --check)` → `./scripts/format_docs.sh --check` → `(cd code && cargo clippy --workspace --all-targets -- -D warnings)` → `(cd code && cargo test --workspace)` → `(cd code && cargo check --target wasm32-unknown-unknown -p mu-console-ui -p mu-console-data -p mu-console-protocol -p mu-console-theme)`.

## 2. Licensing & Font Binaries

- **Two-Tier Licences**: `mu-console-protocol`, `mu-console-theme`, and `mu-console-layout` are `LGPL-3.0-only WITH LGPL-3.0-linking-exception` and must never depend on an AGPL crate. `mu-console-ui` and `mu-console-data` are `AGPL-3.0-only`.
- **No Font Binaries in Git**: Never commit `*.woff2`, `*.ttf`, `*.otf` or SVGs with embedded fonts (concept renders embed base64 HeavyData). Fonts come only from the pinned, checksum-verified `scripts/fetch-fonts.sh`.
- **Public Repository Hygiene**: Never commit internal hostnames, private IP addresses or internal DNS names. Use generic placeholders (`node-a`, `worker-riscv64`, `192.0.2.x`).

## 3. Layout & Rendering Invariants

- **Zero Page Scroll**: `html, body { overflow: hidden; height: 100dvh }`. Overflow inside a panel uses panel-local paging; only the log viewer and repo picker may scroll (virtualized).
- **Solver-Owned Breakpoints**: Breakpoints and frames come from `mu-console-layout`, never from CSS media queries. Any layout change must keep the native no-overflow tests green for every target viewport and both CRT fonts.
- **Fixed CRT Column Grid**: CRT font size is derived as `column_px / advance_em`; switching fonts never reflows the grid. Below the floor (12 px desktop, 13 px phone) text truncates with `…`.
- **Deterministic Generators**: Theme generators are pure functions of their inputs; identical inputs must produce byte-identical SVG.

## 4. Rust Conventions

- Use `usize` for indexing, lengths and anything tied to memory; use `u32` for protocol versions, IDs, counters and other domain values.
- Mobile form inputs use `font-size: 16px`; WASM/JS/CSS assets use content-hash `?v=` busting with `no-cache, must-revalidate`, and the HTML entry point is `no-cache, no-store, must-revalidate`.
