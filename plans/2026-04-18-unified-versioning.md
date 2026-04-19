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

Edit `version` in `[workspace.package]` — all four member crates pick it up. Then mirror the new version in each intra-workspace entry under `[workspace.dependencies]` (`runi-core`, `runi-log`, `runi-cli`, `runi-test`), since Cargo doesn't let those inherit. So: one canonical edit plus four mirror edits per bump, which is why a helper like `cargo set-version` / `cargo-release` is the practical path once publishing starts.

When a member later needs its own cadence, override `version = "..."` in that crate's `[package]` and bump the matching `[workspace.dependencies]` entry if another workspace member depends on it.

## Files touched

- `Cargo.toml` — add `[workspace.package]` and `[workspace.dependencies]`.
- `runi-cli/Cargo.toml`, `runi-core/Cargo.toml`, `runi-log/Cargo.toml`, `runi-test/Cargo.toml` — switch to `workspace = true` inheritance.

## Verification

- `cargo check --workspace` clean.
- `cargo fmt --check` / `cargo clippy` stay green in CI.
