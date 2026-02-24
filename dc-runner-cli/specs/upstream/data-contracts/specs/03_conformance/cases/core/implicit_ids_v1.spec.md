```yaml contract-spec
spec_version: 1
schema_ref: "/specs/01_schema/schema_v1.md"
harness:
  type: unit.test
  profile: check
contracts:
  clauses:
  - id: DCCONF-IMPLICIT-ID-001
    title: omitted docs id remains valid metadata
    docs:
    - summary: contract docs id intentionally omitted
      audience: implementer
      status: active
      id: implicit_ids_v1.doc.1.1
      description: |-
        Purpose: Capture actionable details on how this contract symbol behaves for the declared audience.

        Inputs:
        - Inputs come from the declared contract and export bindings for this symbol.
        - Runtime bindings and policy constraints are applied by the harness before evaluation.

        Returns:
        - The assertion or helper return value produced by this symbol.

        Errors/Caveats:
        - Malformed inputs and shape mismatches are surfaced as validation failures.
        - Schema/runtime binding or environment mismatches are surfaced as runtime or validation failures.
        - Policy and validation failures are surfaced through contract evaluation, including policy assertions.

        Usage context:
        - Use this for implementation work, local debugging, and runner-side behavior analysis.
      inputs:
      - Contract parameters and required case inputs associated with this docs-bearing
        symbol.
      - Any runtime symbols declared through harness/config bindings for the owning
        execution path.
      returns:
      - Structured evaluation result as defined by the owning assert/export symbol.
      errors:
      - Validation failures for malformed inputs and invalid bindings.
      - Runtime environment and policy compatibility errors.
      usage_context:
      - Use this for implementation work, local debugging, and runner-side behavior
        analysis.
    - summary: contract docs id intentionally omitted (operator)
      audience: operator
      status: active
      id: implicit_ids_v1.doc.1.1.operator
      description: |-
        Purpose: Capture actionable details on how this contract symbol behaves for the declared audience.

        Inputs:
        - Inputs come from the declared contract and export bindings for this symbol.
        - Runtime bindings and policy constraints are applied by the harness before evaluation.

        Returns:
        - The assertion or helper return value produced by this symbol.

        Errors/Caveats:
        - Malformed inputs and shape mismatches are surfaced as validation failures.
        - Schema/runtime binding or environment mismatches are surfaced as runtime or validation failures.
        - Policy and validation failures are surfaced through contract evaluation, including policy assertions.

        Usage context:
        - Use this for observability, runbook readiness, and incident response.
      inputs:
      - Contract parameters and required case inputs associated with this docs-bearing
        symbol.
      - Any runtime symbols declared through harness/config bindings for the owning
        execution path.
      returns:
      - Structured evaluation result as defined by the owning assert/export symbol.
      errors:
      - Validation failures for malformed inputs and invalid bindings.
      - Runtime environment and policy compatibility errors.
      usage_context:
      - Use this for observability, runbook readiness, and incident response.
    - summary: contract docs id intentionally omitted (integrator)
      audience: integrator
      status: active
      id: implicit_ids_v1.doc.1.1.integrator
      description: |-
        Purpose: Capture actionable details on how this contract symbol behaves for the declared audience.

        Inputs:
        - Inputs come from the declared contract and export bindings for this symbol.
        - Runtime bindings and policy constraints are applied by the harness before evaluation.

        Returns:
        - The assertion or helper return value produced by this symbol.

        Errors/Caveats:
        - Malformed inputs and shape mismatches are surfaced as validation failures.
        - Schema/runtime binding or environment mismatches are surfaced as runtime or validation failures.
        - Policy and validation failures are surfaced through contract evaluation, including policy assertions.

        Usage context:
        - Use this for composing this contract in pipelines, services, and toolchains.
      inputs:
      - Contract parameters and required case inputs associated with this docs-bearing
        symbol.
      - Any runtime symbols declared through harness/config bindings for the owning
        execution path.
      returns:
      - Structured evaluation result as defined by the owning assert/export symbol.
      errors:
      - Validation failures for malformed inputs and invalid bindings.
      - Runtime environment and policy compatibility errors.
      usage_context:
      - Use this for composing this contract in pipelines, services, and toolchains.
    - summary: contract docs id intentionally omitted (maintainer)
      audience: maintainer
      status: active
      id: implicit_ids_v1.doc.1.1.maintainer
      description: |-
        Purpose: Capture actionable details on how this contract symbol behaves for the declared audience.

        Inputs:
        - Inputs come from the declared contract and export bindings for this symbol.
        - Runtime bindings and policy constraints are applied by the harness before evaluation.

        Returns:
        - The assertion or helper return value produced by this symbol.

        Errors/Caveats:
        - Malformed inputs and shape mismatches are surfaced as validation failures.
        - Schema/runtime binding or environment mismatches are surfaced as runtime or validation failures.
        - Policy and validation failures are surfaced through contract evaluation, including policy assertions.

        Usage context:
        - Use this for versioning, changelogs, and stability planning.
      inputs:
      - Contract parameters and required case inputs associated with this docs-bearing
        symbol.
      - Any runtime symbols declared through harness/config bindings for the owning
        execution path.
      returns:
      - Structured evaluation result as defined by the owning assert/export symbol.
      errors:
      - Validation failures for malformed inputs and invalid bindings.
      - Runtime environment and policy compatibility errors.
      usage_context:
      - Use this for versioning, changelogs, and stability planning.
    - summary: contract docs id intentionally omitted (governance)
      audience: governance
      status: active
      id: implicit_ids_v1.doc.1.1.governance
      description: |-
        Purpose: Capture actionable details on how this contract symbol behaves for the declared audience.

        Inputs:
        - Inputs come from the declared contract and export bindings for this symbol.
        - Runtime bindings and policy constraints are applied by the harness before evaluation.

        Returns:
        - The assertion or helper return value produced by this symbol.

        Errors/Caveats:
        - Malformed inputs and shape mismatches are surfaced as validation failures.
        - Schema/runtime binding or environment mismatches are surfaced as runtime or validation failures.
        - Policy and validation failures are surfaced through contract evaluation, including policy assertions.

        Usage context:
        - Use this for policy gating, approval review, and compliance checks.
      inputs:
      - Contract parameters and required case inputs associated with this docs-bearing
        symbol.
      - Any runtime symbols declared through harness/config bindings for the owning
        execution path.
      returns:
      - Structured evaluation result as defined by the owning assert/export symbol.
      errors:
      - Validation failures for malformed inputs and invalid bindings.
      - Runtime environment and policy compatibility errors.
      usage_context:
      - Use this for policy gating, approval review, and compliance checks.
    - summary: contract docs id intentionally omitted (reviewer)
      audience: reviewer
      status: active
      id: implicit_ids_v1.doc.1.1.reviewer
      description: |-
        Purpose: Capture actionable details on how this contract symbol behaves for the declared audience.

        Inputs:
        - Inputs come from the declared contract and export bindings for this symbol.
        - Runtime bindings and policy constraints are applied by the harness before evaluation.

        Returns:
        - The assertion or helper return value produced by this symbol.

        Errors/Caveats:
        - Malformed inputs and shape mismatches are surfaced as validation failures.
        - Schema/runtime binding or environment mismatches are surfaced as runtime or validation failures.
        - Policy and validation failures are surfaced through contract evaluation, including policy assertions.

        Usage context:
        - Use this to verify correctness, completeness, and release readiness.
      inputs:
      - Contract parameters and required case inputs associated with this docs-bearing
        symbol.
      - Any runtime symbols declared through harness/config bindings for the owning
        execution path.
      returns:
      - Structured evaluation result as defined by the owning assert/export symbol.
      errors:
      - Validation failures for malformed inputs and invalid bindings.
      - Runtime environment and policy compatibility errors.
      usage_context:
      - Use this to verify correctness, completeness, and release readiness.
    - summary: contract docs id intentionally omitted (auditor)
      audience: auditor
      status: active
      id: implicit_ids_v1.doc.1.1.auditor
      description: |-
        Purpose: Capture actionable details on how this contract symbol behaves for the declared audience.

        Inputs:
        - Inputs come from the declared contract and export bindings for this symbol.
        - Runtime bindings and policy constraints are applied by the harness before evaluation.

        Returns:
        - The assertion or helper return value produced by this symbol.

        Errors/Caveats:
        - Malformed inputs and shape mismatches are surfaced as validation failures.
        - Schema/runtime binding or environment mismatches are surfaced as runtime or validation failures.
        - Policy and validation failures are surfaced through contract evaluation, including policy assertions.

        Usage context:
        - Use this as documented evidence for audit and policy review.
      portable:
        status: pass
        category:
      inputs:
      - Contract parameters and required case inputs associated with this docs-bearing
        symbol.
      - Any runtime symbols declared through harness/config bindings for the owning
        execution path.
      returns:
      - Structured evaluation result as defined by the owning assert/export symbol.
      errors:
      - Validation failures for malformed inputs and invalid bindings.
      - Runtime environment and policy compatibility errors.
      usage_context:
      - Use this as documented evidence for audit and policy review.
    asserts:
      checks:
      - id: assert_1
        assert:
          lit: true
```

```yaml contract-spec
spec_version: 1
schema_ref: "/specs/01_schema/schema_v1.md"
harness:
  type: unit.test
  profile: check
  docs:
  - summary: harness docs owner id omitted
    audience: implementer
    status: active
    owners:
    - role: owner
contracts:
  clauses:
  - id: DCCONF-IMPLICIT-ID-002
    title: omitted docs owner id remains valid metadata
    expect:
      portable:
        status: pass
        category:
    asserts:
      checks:
      - id: assert_1
        assert:
          lit: true
```

```yaml contract-spec
spec_version: 1
schema_ref: "/specs/01_schema/schema_v1.md"
harness:
  type: unit.test
  profile: check
contracts:
  clauses:
  - id: DCCONF-IMPLICIT-ID-003
    title: missing predicate id is schema failure
    expect:
      portable:
        status: fail
        category: schema
    asserts:
      checks:
      - assert:
          lit: true
```

```yaml contract-spec
spec_version: 1
schema_ref: "/specs/01_schema/schema_v1.md"
harness:
  type: unit.test
  profile: check
contracts:
  clauses:
  - id: DCCONF-IMPLICIT-ID-004
    title: synthetic report label is invalid as reference identity
    expect:
      portable:
        status: fail
        category: schema
    bindings:
      defaults:
        service: svc.default.4
        import: pipe_identity
      rows:
      - id: bind.invalid.synthetic
        outputs:
        - to: out_json
    asserts:
      checks:
      - id: assert_1
        assert:
          lit: true
adapters:
- type: io.system
  defaults:
    direction: bidirectional
    imports:
    - names:
      - pipe_identity
    profile: exec.command
  actions:
  - id: svc.default.4
services:
- id: svc.default.4
  consumes:
  - svc.default.4
  exposes:
  - names:
    - pipe_identity
  bindings:
    pipe_identity:
      adapter_action: svc.default.4
      adapter_import: pipe_identity
artifacts:
- id: out_json
  ref: artifact://implicit_ids/out_json
  type: application/json
```
