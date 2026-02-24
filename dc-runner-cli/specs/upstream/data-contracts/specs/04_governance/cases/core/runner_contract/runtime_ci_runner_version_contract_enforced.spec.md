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
      - /specs/00_core/runner_version_contract_v1.yaml
      required_tokens:
      - runner-preflight:
      - specs/00_core/runner_version_contract_v1.yaml
      - cargo install "${{ steps.contract.outputs.crate_name }}" --locked --version "${{ steps.contract.outputs.required_version }}" --force
      forbidden_tokens:
      - cargo install --git
      - --rev
    check:
      profile: governance.scan
      config:
        check: runtime.ci_runner_version_contract_enforced
contracts:
  clauses:
  - id: DCGOV-RUNTIME-CI-VC-001
    title: ci runner install resolves via version contract
    purpose: Ensures CI installs runner from the required version contract and forbids git SHA install paths.
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
  - id: act.gov.runtime.ci.version.contract.1
    direction: bidirectional
    profile: default
services:
- id: svc.gov.runtime.ci.version.contract.1
  consumes:
  - act.gov.runtime.ci.version.contract.1
```
