# CLAUDE.md

## Project Overview

**Runi** is a curated Rust foundation library collection.

## Workspace

Crates (unified version via `workspace.package.version`):

- **runi-core** — core types
- **runi-log** — structured logging
- **runi-test** — test utilities
- **runi-cli** — terminal styling and CLI launcher (with `derive` feature)
- **runi-cli-macros** — proc-macros for `runi-cli`'s `#[derive(Command)]`

## Commands

```bash
cargo build --workspace --all-targets   # Build everything
cargo test --workspace                  # Run tests
cargo fmt --all                         # Format Rust (CI checks --check)
taplo fmt                               # Format TOML (CI checks --check)
cargo clippy --workspace --all-targets -- -D warnings
mdbook serve docs                       # Local docs preview
```

## Git workflow

- Never push directly to `main`. All changes require PRs.
- Create the branch first: `git switch -c <prefix>/<topic>`.
- Save plan documents to `plans/YYYY-MM-DD-<topic>.md`.
- Use `gh` for PR management.
- Never enable auto-merge without explicit user approval. When approved: `gh pr merge --squash --auto`.

### Branch prefixes → PR labels

Branch prefixes drive automatic PR labels via `.github/labeler.yml`, which in turn drive the changelog categories in `.github/release.yml` for `gh release create --generate-notes`.

| Prefix       | Label            | Release section          |
|--------------|------------------|--------------------------|
| `breaking/`  | `breaking`       | 🔥 Breaking Changes      |
| `feature/`   | `feature`        | 🚀 Features              |
| `fix/`       | `bug`            | 🐛 Bug Fixes             |
| `chore/`     | `internal`       | 🛠 Internal Updates      |
| `test/`      | `internal`       | 🛠 Internal Updates      |
| `deps/`      | `library-update` | 🔗 Dependency Updates    |
| `docs/`      | `doc`            | 📚 Docs                  |

PRs without one of these prefixes still appear in the release notes under "Other Changes". Dependabot PRs come pre-labeled with `dependencies` and slot into the dependency section automatically.

## Release process

Tag-driven, automated by `.github/workflows/release.yml` and `release-note.yml`:

1. Bump `[workspace.package].version` (and matching `workspace.dependencies` versions) in the root `Cargo.toml`; merge the bump PR.
2. `git tag vX.Y.Z origin/main && git push origin vX.Y.Z`.
3. The release workflow publishes all five crates to crates.io in dependency order; the release-note workflow creates a GitHub release with auto-generated notes grouped by label.

`CARGO_REGISTRY_TOKEN` must be set in repo secrets.
