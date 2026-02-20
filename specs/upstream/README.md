# Upstream Snapshot

This directory stores a pinned compatibility snapshot of Data Contracts specs
and a pinned runner-specific snapshot consumed by `dc-runner-rust`.

Source of truth:

- `/specs/upstream/data_contracts_lock_v1.yaml` (upstream repo/tag-or-ref/commit + integrity)
- `/specs/upstream/data-contracts.manifest.sha256` (deterministic file manifest)
- `/specs/upstream/data-contracts/` (vendored spec snapshot used for compatibility verification)
- `/specs/upstream/dc_runner_spec_lock_v1.yaml` (runner-spec repo/tag-or-ref/commit + integrity)
- `/specs/upstream/dc-runner-spec.manifest.sha256` (deterministic file manifest)
- `/specs/upstream/dc-runner-spec/` (vendored runner-specific spec snapshot)

Update flow:

```sh
cargo xtask spec sync --tag <upstream-tag> --source <path-or-url>
cargo xtask runner-spec sync --tag <runner-spec-tag> --source <path-or-url>
cargo xtask verify all
```

Integrity check:

```sh
cargo xtask spec check
cargo xtask runner-spec check
```

Related docs:

- `/docs/compatibility.md`
- `/docs/release.md`
