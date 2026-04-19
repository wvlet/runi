# Unified workspace versioning for runi

## Problem

The `runi` workspace has four crates (`runi-cli`, `runi-core`, `runi-log`, `runi-test`), each with its own `version`, `edition`, and `license` duplicated in its `Cargo.toml`. All four are currently at `0.1.0` and move together — the per-crate duplication invites drift and makes bumps an N-file edit.

At this stage (pre-1.0, small workspace, no external pinning), unified versioning is the right tradeoff: simpler process, no spurious bumps, crates evolve as a set. Independent versioning can be adopted later if cadences diverge.

## Approach

1. Add `[workspace.package]` in the root `Cargo.toml` with shared `version`, `edition`, `license`.
2. Add `[workspace.dependencies]` centralizing:
   - Intra-workspace crates (`path` + `version` so publishing works).
   - Third-party deps currently duplicated across members (`nu-ansi-term`, `rstest`, etc.).
3. Rewrite each member `Cargo.toml` to inherit via `version.workspace = true` and `dep.workspace = true`.
4. Verify `cargo check --workspace` still passes.

## Non-goals

- No version bump. `0.1.0` stays.
- No crate split/merge, no feature changes.
- No publishing pipeline yet.

## Bump procedure (after this lands)

Edit `version` in `[workspace.package]` — all four member crates pick it up. Whether you also need to touch `[workspace.dependencies]` depends on the bump kind:

- **Patch bump within the same `0.x` line** (e.g. `0.1.0` → `0.1.1`): only the `[workspace.package].version` edit is needed. Cargo's default `^0.1.0` requirement still resolves, so intra-workspace consumers continue to find the new version.
- **Minor or major bump in `0.x`** (e.g. `0.1.0` → `0.2.0`, treated as breaking by Cargo) or a `1.0` transition: also update the `version = "..."` field in each intra-workspace entry under `[workspace.dependencies]` (`runi-cli`, `runi-core`, `runi-log`, `runi-test`), otherwise path-resolved members won't satisfy the old requirement. A helper like `cargo set-version` (from `cargo-edit`) or `cargo-release` handles this in one command.

When a member later needs its own cadence, override `version = "..."` in that crate's `[package]` and bump the matching `[workspace.dependencies]` entry if another workspace member depends on it.

## Files touched

- `Cargo.toml` — add `[workspace.package]` and `[workspace.dependencies]`.
- `runi-cli/Cargo.toml`, `runi-core/Cargo.toml`, `runi-log/Cargo.toml`, `runi-test/Cargo.toml` — switch to `workspace = true` inheritance.

## Verification

- `cargo check --workspace` clean.
- `cargo fmt --check` / `cargo clippy` stay green in CI.
