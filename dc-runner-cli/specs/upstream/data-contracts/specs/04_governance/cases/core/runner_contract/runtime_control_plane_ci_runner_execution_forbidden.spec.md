```yaml contract-spec
spec_version: 1
schema_ref: /specs/01_schema/schema_v1.md
harness:
  type: unit.test
  profile: check
  config:
    root: .
    ci_runtime_exec:
      files:
      - /.github/workflows/ci.yml
      - dc-runner governance critical
      forbidden_tokens:
      - ./dc-runner governance run
      - ./dc-runner
    check:
      profile: governance.scan
      config:
        check: runtime.control_plane_ci_runner_execution_forbidden
contracts:
  clauses:
  - id: DCGOV-RUNTIME-CI-001
    title: control-plane ci forbids runtime runner execution
    purpose: Ensures CI does not bypass the control plane by invoking runtime lane commands directly.
    asserts:
      imports:
      - from: asset
        names:
        - violation_count
      checks:
      - id: assert_1
        assert:
          call:
          - var: policy.assert.no_violations
          - std.object.assoc:
            - violation_count
            - var: violation_count
            - lit: {}
adapters:
- type: beta.scan
  actions:
  - id: act.gov.runtime.control.plane.ci.1
    direction: bidirectional
    profile: default
services:
- id: svc.gov.runtime.control.plane.ci.1
  consumes:
  - act.gov.runtime.control.plane.ci.1
```
