# Runner CLI Interface Contract (v1)

Defines the portable CLI surface shared by runner implementations and consumed
by the control-plane conformance layer.

## Scope

This contract specifies only implementation-agnostic runner CLI behavior.
Runtime-specific or implementation-specific subcommands remain runner-owned.
Runner-owned CLI behavior specs are maintained in external runner spec
repositories, not in canonical schema trees.
Command behavior is resolved from spec-defined command entrypoints, not shell
wrappers.

## MUST Surface

Runner CLIs MUST provide deterministic behavior for:

- `dc-runner --help`
- `dc-runner conformance`
- `dc-runner governance`
- `dc-runner critical-gate`
- `dc-runner schema check`
- `dc-runner schema lint`
- `dc-runner schema format`
- `dc-runner docs generate`
- `dc-runner docs generate-check`
- `dc-runner docs build`
- `dc-runner docs build-check`
- `dc-runner docs lint`
- `dc-runner docs graph`
- `dc-runner bundle list`
- `dc-runner bundle info --bundle-id <id>`
- `dc-runner bundle install --project-lock <path> --out <path>`
- `dc-runner bundle install-check --project-lock <path> --out <path>`
- `dc-runner bundle bootstrap --lock <path> --out <path>`
- `dc-runner bundle bootstrap-check --lock <path> --out <path>`
- `dc-runner bundle outdated --project-lock <path> [--format json|table]`
- `dc-runner bundle upgrade --project-lock <path> [--dry-run]`
- `dc-runner bundle run --bundle-id <id> --bundle-version <semver> --entrypoint <name> [--arg <value>]...`
- `dc-runner bundle scaffold --project-root <path> --bundle-id <id> --bundle-version <semver> [--runner <rust|python|php>]`
- `dc-runner bundle scaffold --project-root <path> --bundle-id <id> --bundle-version <semver> [--runner <rust|python|php>] [--var <key=value>]... [--overwrite]`
- unknown command handling with non-zero exit code
- structured status output mode (`--json` or equivalent capability)

## MAY Surface

Runner CLIs MAY provide:

- implementation-specific helper subcommands
- additional diagnostics modes
- additional output formats beyond the structured mode
- external scaffold source override:
  - `dc-runner bundle scaffold --project-root <path> --bundle-url <url> --sha256 <hex> --allow-external`
- optional spec-management suite under `dc-runner specs` for updating and managing local
  spec cache state without a binary upgrade:
  - `dc-runner specs refresh`
  - `dc-runner specs status`
  - `dc-runner specs versions`
  - `dc-runner specs use`
  - `dc-runner specs rollback`
  - `dc-runner specs verify`
  - `dc-runner specs clean`
  - `dc-runner specs info`
  - `dc-runner specs prune`

## Capability Model

Portable required behavior is represented as required commands and output
contract keys in `/specs/01_schema/runner_cli_contract_v1.yaml`.

`schema` command contract:

- `schema check` validates schema registry and schema-case invariants
- `schema lint` runs schema portability/style checks against canonical trees
- `schema format` applies canonical schema/spec normalization edits
- schema command semantics are spec-defined via entrypoints and executable job refs

Canonical executable case payload shape for formatting and execution is
`spec_version/schema_ref/harness/contracts.clauses[].asserts.checks[]`.

Implementation-specific additions MUST be capability-gated and MUST NOT weaken
the required portable command semantics.

## Entrypoint Resolution

Command-to-profile mapping source of truth:

- `/specs/04_governance/runner_entrypoints_v1.yaml`
- `/specs/00_core/runner_version_contract_v1.yaml`

Schema:

- `/specs/01_schema/runner_command_entrypoints_v1.yaml`

For canonical command entrypoints, `dc-runner` MUST resolve command id to
`/specs/04_governance/check_sets_v1.yaml` profile and enforce declared artifact
and exit-code contracts.

Entrypoints marked `visibility: top_level` are promoted to canonical help and
user-facing command surfaces.

## CI Install Contract

Canonical CI environments MUST install `dc-runner-cli` at the exact
`required_version` declared in `/specs/00_core/runner_version_contract_v1.yaml`
and MUST fail if that version is not published.

Canonical CI MUST NOT use git revision install paths (`cargo install --git ...
--rev ...`) for runner resolution.

## Spec Source Resolution

`dc-runner` MUST support runtime source selection for contract refs:

- `--spec-source bundled|workspace|auto`
- `DC_RUNNER_SPEC_SOURCE=bundled|workspace|auto`

Precedence:

1. CLI flag
2. Environment variable
3. Default (`bundled`)

Mode semantics:

- `bundled`: resolve from the highest verified local `core` bundle cache entry when available; fall back to embedded pinned snapshot if no valid `core` cache exists
- `workspace`: resolve from local workspace only
- `auto`: workspace first, then bundled (`core` cache preferred when bundle mode is explicitly selected), then embedded fallback

## Spec State Command Surface

Runners MAY provide a stateful spec lifecycle for operator workflows:

- `dc-runner specs refresh [--source remote|bundled|workspace] [--version <semver|latest>] [--force] [--check-only] [--skip-signature]`
  - default: `--source remote`, `--version latest`
  - metadata-only activation policy: refresh updates cache metadata by default
  - explicit activation via `dc-runner specs use`
- `dc-runner specs status`
  - reports active source/version and last check/refresh metadata
- `dc-runner specs versions`
  - shows installed cache versions with status columns
- `dc-runner specs use <version-or-source> --source version|bundled|workspace`
  - validates target before switch
- `dc-runner specs rollback [--to <version|bundled>]`
  - fallback to last known-good when omitted
- `dc-runner specs verify [--source auto|active|bundled|workspace|cache:<version>]`
  - emits recovery hint when integrity checks fail
- `dc-runner specs clean [--keep <N>] [--dry-run] [--yes]`
  - safe by default and never deletes active/known-good targets
- `dc-runner specs info [<version>]`
  - surfaces metadata and module capabilities for the selected cache entry
- `dc-runner specs prune --expired`
  - applies retention-policy-driven cleanup

Default runtime behavior remains unchanged for explicit `workspace` mode and non-stateful operations.
`bundled` mode now treats a verified local `core` bundle cache as authoritative, with the
embedded snapshot used as a bootstrap/untrusted-fallback path when cache execution is not possible.

## Bundle Scaffold Source Contract

- Canonical scaffold source is `jonruttan/data-contracts-bundles` release assets.
- Canonical mode is bundle-identity based (`bundle_id` + `bundle_version`) and
  MUST be pinned by `/specs/00_core/bundle_version_contract_v1.yaml`.
- External URL mode is non-canonical emergency fallback only and MUST require
  explicit `--allow-external`.
- Scaffold MUST verify bundle integrity before install and emit deterministic
  scaffold artifacts under `/.artifacts/`.
- Scaffold materialization MUST be driven by
  `/specs/01_schema/scaffold_manifest_v1.yaml` bundled as
  `scaffold/scaffold_manifest_v1.yaml`.
- Canonical scaffold bundle IDs for language scaffolding are:
  - `data-contracts-lang-project-scaffold`
  - `data-contracts-lang-rust-project-scaffold`

## No-Install Bundle Run Contract

- `bundle run` canonical mode is bundle-identity based (`bundle_id` + `bundle_version`)
  and MUST resolve from `jonruttan/data-contracts-bundles` release assets.
- `bundle run` MUST fetch the target bundle into a temporary directory.
- `bundle run` MUST verify bundle integrity before execution.
- `bundle run` MUST execute only the requested bundle `entrypoint`.
- `bundle run` MUST delete temporary bundle materialization on both success and failure.
- `bundle run` MAY persist only declared run artifacts/log outputs.
