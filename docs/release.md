# Release

## Release Preconditions

Before tagging a release:

```sh
make verify
```

This ensures build/test integrity plus upstream compatibility validation.

## Version Bump Flow

1. Update crate/release version metadata (for example in `/spec_runner_cli/Cargo.toml`).
2. Run:

```sh
make verify
```

3. Update `/CHANGELOG.md` with user-visible release notes.
4. Commit release changes.

## Tagging Policy

Use semantic tags:

- `vX.Y.Z`

Example:

```sh
git tag v0.2.0
git push origin v0.2.0
```

## Data Contracts Coordination Note

When Data Contracts compatibility version changes:

1. update pinned upstream snapshot (`/make spec-sync`)
2. verify compatibility (`/make verify`)
3. include lock/manifest/snapshot diff in release review

Do not release runner changes that implicitly drift from pinned upstream
contracts without an explicit snapshot bump.

## Post-Release Validation

After push/tag:

1. Confirm CI green on release commit.
2. Confirm release tag points at intended commit.
3. Confirm compatibility checks still pass from a clean checkout.
