# Tag-triggered cargo publish + release notes

## Problem

We need a repeatable, low-touch release process for the runi workspace crates. Today there is no automation for publishing to crates.io, and no automation for cutting a GitHub release with notes.

## Design

Two new workflows under `.github/workflows/`:

### `release.yml`

- **Triggers:** `push` of tags matching `v*`, plus `workflow_dispatch` for manual reruns. The job is gated by `if: github.ref_type == 'tag' && startsWith(github.ref_name, 'v')` so a manual dispatch from a branch is a no-op rather than an accidental publish.
- **Job:** single `publish` job on `ubuntu-latest`.
- **Steps:**
  1. `actions/checkout@v6` (no special fetch-depth needed for publishing).
  2. `dtolnay/rust-toolchain@stable`.
  3. `Swatinem/rust-cache@v2` to keep build times down.
  4. **Publish loop:** a single bash step iterating crates in dependency order. For each crate it queries `https://crates.io/api/v1/crates/<crate>/<version>`; if that version is already there the crate is skipped, otherwise it runs `cargo publish -p <crate>` followed by `sleep 20` to let the index propagate before the next dependent crate.
  5. Authenticate with `CARGO_REGISTRY_TOKEN` secret via env.
- **Publish order (dependency-driven):**
  1. `runi-test` — published first because `runi-core` and `runi-cli` use it as a dev-dependency, and cargo strips the `path` and resolves against the registry on publish.
  2. `runi-core` — depends only on external crates at runtime; dev-dep on `runi-test`.
  3. `runi-log` — independent; no internal deps.
  4. `runi-cli` — independent at runtime; dev-dep on `runi-test`.
- **Resumability:** the per-crate "already published?" check makes the workflow rerunnable. If a later crate fails after earlier crates were published, fixing the issue and re-dispatching on the same tag will skip the already-published crates and continue the chain rather than failing on the first duplicate.

### `release-note.yml`

- **Triggers:** `push` of tags matching `v*`, plus `workflow_dispatch`. Same `github.ref_type == 'tag'` guard as the publish workflow so a manual run from a branch cannot create a bogus release / floating tag.
- **Step:** `gh release view` precheck so that reruns on a tag whose release already exists are a no-op (green); otherwise `gh release create "$GITHUB_REF_NAME" --repo "$GITHUB_REPOSITORY" --generate-notes`.
- **Permissions:** `contents: write` so the GITHUB_TOKEN can create releases.

## Manual release process

1. Bump `[workspace.package].version` and the matching `workspace.dependencies` versions in the root `Cargo.toml`.
2. Open and merge the bump PR.
3. `git tag vX.Y.Z && git push origin vX.Y.Z`.
4. Both workflows fire on the tag: crates land on crates.io and a GitHub release appears with auto-generated notes.

## Secrets

- `CARGO_REGISTRY_TOKEN` — must be added to repo settings (Settings → Secrets and variables → Actions). The workflow will fail loudly if missing.

## Trade-offs / notes

- The skip-if-already-published check exists only so that the very first failure recovery isn't a manual chore — without it, a rerun on the same tag fails immediately on the first already-published crate.
- We deliberately keep the workflows small. A tag-vs-manifest preflight, sparse-index polling, etc. were considered but rejected: each adds new failure modes for a release path that runs a handful of times per year, and the recovery from the underlying mistakes (yank, retag) is a maintainer-level operation anyway.
- We deliberately *don't* run `cargo test` in `release.yml`. CI on `main` already gates that, and re-running it on tag push would slow the publish down without adding signal.
- We use `--generate-notes` rather than maintaining a hand-curated CHANGELOG.md. Adoption of git-cliff or a CHANGELOG-driven release can come later if the auto-generated notes prove insufficient.
