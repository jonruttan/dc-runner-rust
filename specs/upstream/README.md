# Upstream Snapshot

This directory stores a pinned compatibility snapshot of Data Contracts specs
consumed by `dc-runner-rust`.

Source of truth:

- `/specs/upstream/data_contracts_lock_v1.yaml` (upstream repo/tag-or-ref/commit + integrity)
- `/specs/upstream/data-contracts.manifest.sha256` (deterministic file manifest)
- `/specs/upstream/data-contracts/` (vendored spec snapshot used for compatibility verification)

Update flow:

```sh
cargo xtask spec-sync --tag <upstream-tag> --source <path-or-url>
cargo xtask verify
```

Integrity check:

```sh
cargo xtask spec-sync-check
```

Related docs:

- `/docs/compatibility.md`
- `/docs/release.md`
