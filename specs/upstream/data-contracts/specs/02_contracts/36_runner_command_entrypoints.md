# Runner Command Entrypoints Contract (v1)

Defines the spec-owned mapping from canonical runner commands to governance
execution profiles.

## Canonical Command Surface

Canonical user-facing commands are:

- `dc-runner governance`
- `dc-runner critical-gate`
- `dc-runner docs generate`
- `dc-runner docs generate-check`
- `dc-runner docs build`
- `dc-runner docs build-check`
- `dc-runner docs lint`
- `dc-runner docs graph`
- `dc-runner governance-broad-native`

The mapping source of truth is:

- `/specs/04_governance/runner_entrypoints_v1.yaml`

Schema contract:

- `/specs/01_schema/runner_command_entrypoints_v1.yaml`

## Model

Each command entry defines:

- `id`
- `cli`
- `profile`
- `visibility` (`top_level|hidden`)
- `group` (optional group label for command discovery/help)
- `source` (`core|bundler`)
- `artifacts[]`
- `exit_codes.allowed[]`

The runner resolves command id to profile and executes the profile from:

- `/specs/governance/check_sets_v1.yaml`

Entrypoint resolution MUST respect runner source mode:

- `bundled` default
- `workspace` explicit local override
- `auto` workspace-first fallback

## Hard Requirements

1. `cli` MUST begin with `dc-runner`.
2. `profile` MUST resolve to a defined profile in check sets.
3. Required `artifacts[]` MUST exist on successful/failed command completion.
4. Unknown command ids MUST fail with actionable output listing available ids.
5. Shell scripts are not command-authority sources.
6. Required schema suite entrypoints (`schema-check`, `schema-lint`,
   `schema-format`) MUST be present with `visibility: top_level`.
