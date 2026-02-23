# Release

## Canonical Release Flow

Releases are automated in two stages:

1. `/.github/workflows/release-please.yml` runs on pushes to `main` and opens/updates a Release PR.
2. Merging the Release PR creates a `vX.Y.Z` tag.
3. `/.github/workflows/release.yml` runs on that tag and publishes:
   - crates.io crate: `dc-runner-cli`
   - GitHub binaries: `dc-runner-darwin-arm64`, `dc-runner-linux-x86_64`
   - checksum files: `*.sha256`

Manual tag creation is fallback-only.

## Conventional Commit Requirements

Release versioning is driven by conventional commit signals from merged changes.

Accepted types:

- `feat`
- `fix`
- `perf`
- `refactor`
- `docs`
- `chore`
- `test`
- `build`
- `ci`

PR titles are validated in CI using this format:

```text
<type>(optional-scope): summary
```

## Publish Preconditions

Before merging release-impacting work:

```sh
cargo xtask verify all
```

## Release Workflow Contracts

`release.yml` enforces:

1. crate publish token exists (`CRATES_IO_TOKEN`)
2. tag `vX.Y.Z` matches `/dc-runner-cli/Cargo.toml` version
3. crate packages successfully before publish

If any check fails, publish stops before upload.

## Required Repository Settings

- `CRATES_IO_TOKEN` secret is configured and valid.
- GitHub Actions has permission to create PRs/tags/releases.
- Maintainers can merge Release PRs to `main`.

## Failure Handling

### Missing crates token

Release job fails early with:

- `ERROR: CRATES_IO_TOKEN is not configured for this repository.`

Fix: add or rotate `CRATES_IO_TOKEN` in repo secrets, then rerun the failed workflow.

### Tag/version mismatch

Release job fails with mismatch details.

Fix: do not hand-edit tags; merge the generated Release PR so tag/version stay aligned.

### Version already published

Crates publish step fails on duplicate version.

Fix: create a new releasable commit, let release-please open a new Release PR, merge it.

## Emergency Manual Fallback

Use only when automation is unavailable:

1. update version and changelog manually
2. run `cargo xtask verify all`
3. create and push matching tag `vX.Y.Z`
4. confirm `release.yml` succeeds

## Release Notes Install Snippet

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
