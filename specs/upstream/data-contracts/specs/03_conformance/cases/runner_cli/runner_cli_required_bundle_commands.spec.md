```yaml contract-spec
spec_version: 1
schema_ref: /specs/01_schema/schema_v1.md
harness:
  type: unit.test
  profile: check
  config:
    check:
      profile: text.file
      config:
        path: /specs/02_contracts/29_runner_cli_interface.md
contracts:
  clauses:
  - id: DCCONF-RCLI-008
    title: runner cli exposes bundle list command
    purpose: Portable CLI contract requires bundle listing surface.
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
          - dc-runner bundle list
  - id: DCCONF-RCLI-009
    title: runner cli exposes bundle inspect command
    purpose: Portable CLI contract requires bundle inspect surface.
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
          - dc-runner bundle inspect --bundle-id <id>
  - id: DCCONF-RCLI-010
    title: runner cli exposes bundle install command
    purpose: Portable CLI contract requires bundle install surface.
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
          - dc-runner bundle install --bundle-id <id> --bundle-version <semver>
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
