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
- `dc-runner project scaffold --project-root <path> --bundle-id <id> --bundle-version <semver> [--runner <rust|python|php>]`
- `dc-runner project scaffold --project-root <path> --bundle-id <id> --bundle-version <semver> [--runner <rust|python|php>] [--var <key=value>]... [--overwrite]`
- unknown command handling with non-zero exit code
- structured status output mode (`--json` or equivalent capability)

## MAY Surface

Runner CLIs MAY provide:

- implementation-specific helper subcommands
- additional diagnostics modes
- additional output formats beyond the structured mode
- external scaffold source override:
  - `dc-runner project scaffold --project-root <path> --bundle-url <url> --sha256 <hex> --allow-external`

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

Schema:

- `/specs/01_schema/runner_command_entrypoints_v1.yaml`

For canonical command entrypoints, `dc-runner` MUST resolve command id to
`/specs/governance/check_sets_v1.yaml` profile and enforce declared artifact
and exit-code contracts.

Entrypoints marked `visibility: top_level` are promoted to canonical help and
user-facing command surfaces.

## Spec Source Resolution

`dc-runner` MUST support runtime source selection for contract refs:

- `--spec-source bundled|workspace|auto`
- `DC_RUNNER_SPEC_SOURCE=bundled|workspace|auto`

Precedence:

1. CLI flag
2. Environment variable
3. Default (`bundled`)

Mode semantics:

- `bundled`: resolve from embedded pinned snapshot only
- `workspace`: resolve from local workspace only
- `auto`: workspace first, bundled fallback

## Scaffold Source Contract

- Canonical scaffold source is `jonruttan/data-contracts-bundles` release assets.
- Canonical mode is bundle-identity based (`bundle_id` + `bundle_version`), not
  raw URL based.
- External URL mode is opt-in only and MUST require explicit
  `--allow-external`.
- Scaffold MUST verify bundle integrity before install and emit deterministic
  scaffold artifacts under `/.artifacts/`.
- Scaffold materialization MUST be driven by
  `/specs/01_schema/scaffold_manifest_v1.yaml` bundled as
  `scaffold/scaffold_manifest_v1.yaml`.
- Canonical scaffold bundle IDs for language scaffolding are:
  - `data-contracts-lang-project-scaffold`
  - `data-contracts-lang-rust-project-scaffold`
