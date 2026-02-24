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
  - id: DCCONF-RCLI-013
    title: runner cli exposes specs use command
    purpose: Optional stateful spec command for activating a cached or canonical source.
    asserts:
      imports:
      - from: asset
        names:
        - text
      checks:
      - id: assert_1
        required: false
        assert:
          std.string.contains:
          - var: text
          - dc-runner specs use
adapters:
- type: beta.check_profile_text_file_config
  actions:
  - id: svc.check_profile_text_file_config.default.1
    direction: bidirectional
    profile: default
services:
- id: svc.check_profile_text_file_config.default.1
  consumes:
  - svc.check_profile_text_file_config.default.1
```
