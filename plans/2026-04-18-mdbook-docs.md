# 2026-04-18 — Documentation site with mdBook

## Goal

Bootstrap a user-facing documentation site for the `runi` workspace using
[mdBook](https://rust-lang.github.io/mdBook/). The site will live under
`docs/` and be published to GitHub Pages on every push to `main`.

## Why mdBook (not VitePress)

Runi is an early-stage collection of Rust infrastructure crates
(`runi-core`, `runi-log`, `runi-cli`, `runi-test`). At this stage the
docs are primarily a user guide that complements the auto-published
`docs.rs` API reference. We want:

- Zero JS toolchain (no `node_modules`, no lockfile churn)
- Rust-native tooling the workspace already depends on
- Fast CI: a single cargo-installed binary builds the site
- Simple stable format — the same tool the Rust Book itself uses

We can revisit VitePress later if the project grows a marketing site or
needs interactive components; for now mdBook keeps the toolchain tight.

## Scope

This first PR sets up the skeleton only. Follow-up PRs fill in crate
guides and examples.

### In scope

- `docs/book.toml` — mdBook configuration
- `docs/src/SUMMARY.md` — table of contents
- Initial chapters:
  - `introduction.md` — what Runi is, who it is for
  - `getting-started.md` — install via Cargo, quick example
  - `crates/overview.md` — map of the workspace crates
  - one short page per crate pointing to docs.rs and a minimal snippet
- `.github/workflows/docs.yml` — build + deploy to GitHub Pages
- Ignore generated `docs/book/` output in `.gitignore`

### Out of scope (follow-up PRs)

- Deep per-crate guides, tutorials, cookbook recipes
- Custom theme / branding
- Search tuning, analytics
- Versioned docs
- Compile-checked examples (see "Known limitations" below)

## Layout

```
docs/
  book.toml
  src/
    SUMMARY.md
    introduction.md
    getting-started.md
    crates/
      overview.md
      runi-core.md
      runi-log.md
      runi-cli.md
      runi-test.md
```

## CI / deploy

GitHub Actions workflow `docs.yml`:

- Triggers: push to `main` that touches `docs/**`, plus `workflow_dispatch`
- Steps: checkout → install mdBook via `cargo install --locked` (cached)
  → `mdbook build docs` → upload `docs/book` as Pages artifact → deploy
- Uses `actions/deploy-pages@v4` with `pages: write`, `id-token: write`

Pages must be enabled on the repo (source = GitHub Actions) — noted in
the PR description so the maintainer can flip it on.

## Verification

- `mdbook build docs` produces `docs/book/index.html` with no warnings
- `mdbook test docs` passes (catches broken links in code fences)
- `cargo fmt --all -- --check` and `cargo clippy --workspace` still pass
  (unchanged — no Rust code touched)

## Known limitations

- **Code snippets are `ignore`d, not compile-checked.** Making
  `mdbook test` compile snippets that use the workspace crates requires
  passing `--extern NAME=path` to rustdoc for each crate (a `-L` search
  path alone is not enough). The clean options are:
  1. generate `--extern` args dynamically and pass via
     `RUSTDOCFLAGS` / a wrapper, or
  2. move the canonical examples into a real Rust file under one of the
     crates and use mdBook's `{{#rustdoc_include}}` to pull them in.
  Option 2 is preferred long-term because it keeps examples compile-
  tested through `cargo test --workspace`. Tracked as a follow-up PR.

## Required-CI gating

Our existing `test.yml` uses `dorny/paths-filter` so `test`, `fmt`, and
`clippy` are skipped on doc-only PRs. A **skipped** job does not satisfy
a required status check, so branch protection must require a single
aggregator job instead. This PR adds `ci-required` to `test.yml`:

- Always runs (`if: always()`)
- Depends on all other jobs
- Fails only if a dependency failed or was cancelled; skipped counts as
  success

After merge, change branch protection to require only `ci-required`.

## Risks / open questions

- **mdBook version pin.** Local `rustc 1.85.1` can't install mdBook
  0.5.x (requires 1.88+). Pin the CI install to `^0.4` for now; bump
  when the workspace MSRV moves.
- **Pages setup.** First deploy requires the repo owner to enable Pages
  with "GitHub Actions" as the source — surface this in the PR body.
