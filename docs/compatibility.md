# Compatibility

## Compatibility Policy

`dc-runner-rust` is the required Rust implementation lane and must remain
compatible with canonical Data Contracts runner contracts.

Canonical contracts are authored upstream in Data Contracts and consumed here
via a pinned vendored snapshot.

## Upstream Pinning Artifacts

Primary artifacts:

- `/specs/upstream/data_contracts_lock_v1.yaml`
- `/specs/upstream/data-contracts.manifest.sha256`
- `/specs/upstream/data-contracts/**`

Lock file records:

- upstream repo
- pinned tag/ref
- resolved commit
- sync timestamp
- snapshot integrity metadata

## Manual Bump Workflow

1. Choose upstream version/tag.
2. Sync snapshot:

```sh
make spec-sync TAG=<upstream-tag> SOURCE=<path-or-url>
```

3. Run full checks:

```sh
make verify
```

4. Review and commit lock + manifest + snapshot changes with any runtime updates.

## What `compat-check` Validates

`/make compat-check` enforces at least:

1. Lock/snapshot/manifest integrity coherence.
2. Presence of required upstream contract/schema/governance artifacts.
3. Required runner subcommand surface compatibility against upstream runner
   interface contract.
4. Representative exit-code semantics (`0/1/2`).
5. Rust required-lane execution policy (no direct Python runtime execution in
   adapter path).

## CI Behavior and Rationale

CI runs:

- `make spec-sync-check`
- `make compat-check`

Rationale:

- detect snapshot drift immediately
- keep compatibility checks deterministic
- preserve a reviewable pinned boundary between upstream contract evolution and
  local runtime implementation changes
