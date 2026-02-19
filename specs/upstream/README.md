# Upstream Snapshot

This directory stores a pinned compatibility snapshot of Data Contracts specs
consumed by `dc-runner-rust`.

Source of truth:

- `data_contracts_lock_v1.yaml` (upstream repo/tag-or-ref/commit + integrity)
- `data-contracts.manifest.sha256` (deterministic file manifest)
- `data-contracts/` (vendored spec snapshot used for compatibility verification)

Update flow:

```sh
make spec-sync TAG=<upstream-tag> SOURCE=<path-or-url>
make verify
```

Integrity check:

```sh
make spec-sync-check
```
