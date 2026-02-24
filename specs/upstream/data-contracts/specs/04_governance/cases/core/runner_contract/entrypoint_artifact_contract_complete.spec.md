```yaml contract-spec
spec_version: 1
schema_ref: /specs/01_schema/schema_v1.md
harness:
  type: unit.test
  profile: check
  config:
    check:
      profile: text.file
      config: {}
contracts:
  clauses:
  - id: DCGOV-ENTRY-003
    title: entrypoint artifact contract is complete
    purpose: Ensures required command artifacts are declared in the entrypoint contract.
    asserts:
      imports:
      - from: asset
        names:
        - text
      checks:
      - id: assert_1
        assert:
          std.string.contains:
          - var: text
          - /.artifacts/governance-summary.json
      - id: assert_2
        assert:
          std.string.contains:
          - var: text
          - /.artifacts/critical-gate-summary.json
      - id: assert_3
        assert:
          std.string.contains:
          - var: text
          - /.artifacts/job-hooks/DCCONF-JOB-012.complete.json
      - id: assert_4
        assert:
          std.string.contains:
          - var: text
          - /.artifacts/job-hooks/DCCONF-JOB-013.complete.json
      - id: assert_5
        assert:
          std.string.contains:
          - var: text
          - /.artifacts/job-hooks/DCCONF-JOB-014.complete.json
      - id: assert_6
        assert:
          std.string.contains:
          - var: text
          - /.artifacts/job-hooks/DCCONF-JOB-006.complete.json
      - id: assert_7
        assert:
          std.string.contains:
          - var: text
          - /.artifacts/job-hooks/DCCONF-JOB-007.complete.json
      - id: assert_8
        assert:
          std.string.contains:
          - var: text
          - /.artifacts/governance-trace.json
      - id: assert_9
        assert:
          std.string.contains:
          - var: text
          - /.artifacts/job-hooks/DCCONF-JOB-009.complete.json
      - id: assert_10
        assert:
          std.string.contains:
          - var: text
          - /.artifacts/job-hooks/DCCONF-JOB-010.complete.json
      - id: assert_11
        assert:
          std.string.contains:
          - var: text
          - /.artifacts/job-hooks/DCCONF-JOB-011.complete.json
adapters:
- type: beta.check_profile_text_file_config
  actions:
  - id: svc.gov.entry.artifact.contract.1
    direction: bidirectional
    profile: default
services:
- id: svc.gov.entry.artifact.contract.1
  consumes:
  - svc.gov.entry.artifact.contract.1
```
