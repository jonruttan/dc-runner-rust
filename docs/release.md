# Release

## Release Preconditions

Before tagging a release:

```sh
cargo xtask verify all
```

This ensures build/test integrity plus upstream compatibility validation.

## Version Bump Flow

1. Update crate/release version metadata (for example in `/spec_runner_cli/Cargo.toml`).
2. Run:

```sh
cargo xtask verify all
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

## Automated Multi-Platform Release

Workflow: `/.github/workflows/release.yml`

Trigger:

- push tag matching `v*`
- optional manual `workflow_dispatch`
- crate publish runs only when tag version matches crate version

Release targets:

- `darwin-arm64` (`aarch64-apple-darwin`)
- `linux-x86_64` (`x86_64-unknown-linux-gnu`)

Published assets per target:

- `dc-runner-<platform>`
- `dc-runner-<platform>.sha256`

The publish job aggregates all matrix artifacts and uploads them to the GitHub
Release associated with the tag.

## Crates.io Publishing

Crate: `dc_runner_cli`

Release workflow includes a crate publish job that:

1. validates `vX.Y.Z` tag equals `version` in `/spec_runner_cli/Cargo.toml`
2. runs `cargo package -p dc_runner_cli --allow-dirty`
3. runs `cargo publish -p dc_runner_cli --locked`

Required repository secret:

- `CRATES_IO_TOKEN`

If tag/version mismatch occurs, crate publish fails with a clear error and no
publish attempt is made.

## Data Contracts Coordination Note

When Data Contracts compatibility version changes:

1. update pinned upstream snapshot (`cargo xtask spec sync`)
2. update pinned runner-specific snapshot (`cargo xtask runner-spec sync`)
3. verify compatibility (`cargo xtask verify all`)
4. include lock/manifest/snapshot diffs in release review

Do not release runner changes that implicitly drift from pinned upstream
contracts without an explicit snapshot bump.

## Post-Release Validation

After push/tag:

1. Confirm CI and Release workflows are green for the release tag.
2. Confirm release tag points at intended commit.
3. Confirm all three platform binaries and checksum files are attached.
4. Confirm compatibility checks still pass from a clean checkout.

## Release Notes Install Snippet

Use this in release notes:

```sh
PLATFORM="darwin-arm64" # or linux-x86_64
VERSION="vX.Y.Z"
BASE="https://github.com/jonruttan/dc-runner-rust/releases/download/${VERSION}"
curl -fL "${BASE}/dc-runner-${PLATFORM}" -o dc-runner
curl -fL "${BASE}/dc-runner-${PLATFORM}.sha256" -o dc-runner.sha256
shasum -a 256 -c dc-runner.sha256
chmod +x dc-runner
sudo mv dc-runner /usr/local/bin/dc-runner
```
