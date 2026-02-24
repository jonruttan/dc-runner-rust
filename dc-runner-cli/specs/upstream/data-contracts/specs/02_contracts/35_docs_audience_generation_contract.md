# Docs Audience Generation Contract (v1)

Defines deterministic generation of audience-targeted documentation surfaces from
spec-declared docs metadata.

## Canonical Audience Enum

- `operator`
- `integrator`
- `implementer`
- `maintainer`
- `governance`
- `reviewer`
- `auditor`

## Source Inputs

Generator MUST collect docs metadata from:

1. executable spec surfaces (`*.spec.md`)
2. structured docs metadata fields (`docs[]`, `harness.docs[]`, and equivalent)
3. `yaml doc-meta` fences where present

Non-canonical audience tokens are invalid for active v1 surfaces.

## Structured docs rendering

Generators and downstream renderers MUST treat nested structured `docs[]` entries as
authoritative and render each available field into sectioned content:

- `Purpose`: from `purpose` when present, otherwise from first descriptive sentence in
  `description`.
- `Inputs`: from `inputs` when present; may synthesize a deterministic fallback from
  input parameter descriptions when `inputs` is omitted.
- `Outputs`: from `returns` when present; otherwise from `outputs` when present.
- `Errors`: from `errors` when present.
- `Caveats`: from `caveats` when present.
- `Usage context`: from `usage_context` when present.
- `Preconditions`: from `preconditions` when present.
- `Side effects`: from `side_effects` when present.

If optional structured fields are omitted, generators may fallback to `description`
and preserve existing section markers, but active quality checks should fail for
entries that provide neither non-empty structured fields nor useful fallback text.

## Template-driven rendering

Audience docs generation is asset-first and template-driven:

- generator inputs MAY provide a template asset id via `docs_generate.template_asset_id`
  (indirection through `assets[]`/`artifacts[]` is required by core schema),
  and the runner MUST render using template evaluation semantics from
  `specs/02_contracts/23_docs_template_contract.md`.
- Templates are optional for implementation style, but when present they must render:
  `summary`, `description/purpose`, `inputs`, `returns`, `errors`, `usage_context`,
  and a deterministic `source location` for traceability.
- generated files that are rendered from templates are compared byte-for-byte in
  `--check` mode just like non-template-based rendering.

## Outputs

For each audience, generator MUST emit:

- `/docs/audience/<audience>/index.md`
- `/docs/audience/<audience>/reference_manifest.yaml`
- `/docs/audience/<audience>/reference_index.md`
- `/docs/audience/<audience>/reference_coverage.md`

Output structure schema:

- `/specs/01_schema/docs_audience_surface_v1.yaml`

## Deterministic Rules

1. Normalize metadata and reject unknown audience tokens.
2. Build audience-specific source path set from declared metadata only.
3. Sort source paths lexicographically.
4. Render manifest/index/coverage in stable order.
5. `--check` mode MUST hard-fail on drift from generated result.
6. Output artifacts are generated per audience from executable spec metadata and are
   deterministic regardless of runtime language.

## Runtime and Policy

- `dc-runner docs generate` and `dc-runner docs generate-check` are spec-native
  command entrypoints and must not execute project Python scripts.
- Generator CLI contracts are:
  - `dc-runner docs generate --build`
  - `dc-runner docs generate --check`
  - `dc-runner docs generate --build --surface <audience>`
- Supported operational flags are `--summary-out`, `--timing-out`, `--report-out`,
  and `--jobs`.
- Shell scripts remain wrappers only.
- Audience docs generation/check semantics are governed by executable specs and
  policy contracts.
