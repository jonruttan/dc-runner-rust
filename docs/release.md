# Release

## Release Preconditions

Before tagging a release:

```sh
cargo xtask verify
```

This ensures build/test integrity plus upstream compatibility validation.

## Version Bump Flow

1. Update crate/release version metadata (for example in `/spec_runner_cli/Cargo.toml`).
2. Run:

```sh
cargo xtask verify
```

3. Update `/CHANGELOG.md` with user-visible release notes.
4. Keep `runner_adapter.sh` deprecation note until shim removal release.
5. Commit release changes.

## Tagging Policy

Use semantic tags:

- `vX.Y.Z`

Example:

```sh
git tag v0.2.0
git push origin v0.2.0
```

## Automated Multi-Platform Release

Workflow: `/.github/workflows/release.yml`

Trigger:

- push tag matching `v*`
- optional manual `workflow_dispatch`

Release targets:

- `darwin-arm64` (`aarch64-apple-darwin`)
- `darwin-x86_64` (`x86_64-apple-darwin`)
- `linux-x86_64` (`x86_64-unknown-linux-gnu`)

Published assets per target:

- `dc-runner-rust-<platform>`
- `dc-runner-rust-<platform>.sha256`

The publish job aggregates all matrix artifacts and uploads them to the GitHub
Release associated with the tag.

## Data Contracts Coordination Note

When Data Contracts compatibility version changes:

1. update pinned upstream snapshot (`cargo xtask spec-sync`)
2. verify compatibility (`cargo xtask verify`)
3. include lock/manifest/snapshot diff in release review

Do not release runner changes that implicitly drift from pinned upstream
contracts without an explicit snapshot bump.

## Post-Release Validation

After push/tag:

1. Confirm CI and Release workflows are green for the release tag.
2. Confirm release tag points at intended commit.
3. Confirm all three platform binaries and checksum files are attached.
4. Confirm compatibility checks still pass from a clean checkout.
