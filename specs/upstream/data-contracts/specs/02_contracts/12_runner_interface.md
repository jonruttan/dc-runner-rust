# Runner Interface Contract (v1)

Defines the implementation-agnostic boundary between runner repositories and the
`data-contracts` control plane.

## Scope

This repository does not execute runner implementation lanes in canonical CI.
Runtime execution ownership lives in runner repositories:

- `dc-runner-rust`
- `dc-runner-python`
- `dc-runner-php`
- `data-contracts-runner` (shared runner behavior + shared runner contracts)
- `data-contracts-library` (shared reusable libraries + overlays)

`data-contracts` owns:

- contract/schema definitions for status exchange
- contract/schema definitions for bundle package management
- compatibility matrix normalization rules
- governance rules for freshness and visibility
- docs and reference integrity
- runner-ingestible pack manifests under `/specs/00_core/packs/`
- portable runner CLI contract definitions
- shared governance/check semantics expressed as executable `.spec.md` cases and
  reusable policy libraries

## Stable Boundary

The public command boundary is the installed binary:

- `dc-runner`

Command semantics are source-controlled in:

- `/specs/04_governance/runner_entrypoints_v1.yaml`
- `/specs/00_core/runner_version_contract_v1.yaml`

Runtime source selection is bundled-first:

- default: `bundled` embedded snapshot
- override: `--spec-source` / `DC_RUNNER_SPEC_SOURCE`

Entrypoints with `visibility: top_level` are canonical user command surfaces
and must appear in runner help output.

Shared governance semantics MUST be sourced from spec surfaces (`specs/04_governance/**`
`specs/05_libraries/policy/**`) and executed by runners. Shell scripts are not
canonical command entrypoints.

## CI Compatibility Contract

Canonical CI compatibility is version-contract-driven:

- CI MUST read required runner package metadata from
  `/specs/00_core/runner_version_contract_v1.yaml`.
- CI MUST install exactly `runner.crate` at `required_version` from crates.io.
- CI MUST run a single compatibility preflight before downstream jobs and fail
  when required entrypoints are missing.
- Git SHA installs (`cargo install --git ... --rev ...`) are forbidden in
  canonical CI.

Reusable runner executable spec suites MUST use canonical v1 case shape:

- `spec_version: 1`
- `schema_ref: /specs/01_schema/schema_v1.md`
- `harness` mapping
- `contracts.clauses[]` with `asserts.checks[]`

Boundary contract:

- canonical `data-contracts` docs/specs must not reference internal runner trees
  using internal runner tree path tokens.
- external reusable-runner surfaces must be referenced via explicit repository paths, for
  example `/dc-runner-<impl>/specs/impl/<impl>/**`,
  `/data-contracts-runner/specs/07_runner_behavior/runner/**`, and
  `/data-contracts-runner/specs/07_runner_behavior/contract_sets/shared/**`.

## Status Exchange Boundary

Runner repositories publish release assets that conform to:

- `/specs/01_schema/runner_status_report_v1.yaml`

The control plane ingests and normalizes status through:

- `dc-runner governance run`
- `/specs/01_schema/runner_status_matrix_v1.yaml`
- `/specs/02_contracts/25_compatibility_matrix.md`
- `/specs/02_contracts/27_runner_status_exchange.md`

Portable CLI surface is defined by:

- `/specs/02_contracts/29_runner_cli_interface.md`
- `/specs/01_schema/runner_cli_contract_v1.yaml`

Rust-first required-lane schema suite implementation ownership:

- `dc-runner-rust` MUST implement `schema check`, `schema lint`, and
  `schema format` for v1 schema/spec validation and key-order normalization.

Bundle package management surface is defined by:

- `/specs/02_contracts/33_bundle_package_management.md`
- `/specs/02_contracts/34_runner_implementation_spec_bundles.md`
- `/specs/01_schema/bundle_manifest_v1.yaml`
- `/specs/01_schema/resolved_bundle_lock_v1.yaml`
- `/specs/01_schema/runner_bundle_lock_v1.yaml`
- `/specs/01_schema/project_bundle_lock_v1.yaml`
- `/specs/01_schema/implementation_bundle_overlay_v1.yaml`
- `/specs/01_schema/implementation_bundle_build_lock_v1.yaml`

Canonical bundle manifest and package librarian repository:

- `https://github.com/jonruttan/data-contracts-bundles`

Reusable shared runner behavior repository:

- `https://github.com/jonruttan/data-contracts-runner`

Reusable overlay/library repository:

- `https://github.com/jonruttan/data-contracts-library`

## Policy Semantics

- `lane_class: required` indicates blocking policy effect when the required lane
  report fails or is unavailable.
- `lane_class: compatibility_non_blocking` indicates non-blocking runtime
  compatibility semantics.
- Compatibility telemetry freshness is enforced at 72 hours in this repository.

## Forbidden Couplings

`data-contracts` MUST NOT:

- vendor local runner implementation code
- execute implementation-specific runtime suites in canonical CI
- treat compatibility-lane execution as merge-blocking runtime ownership in this
  repository
