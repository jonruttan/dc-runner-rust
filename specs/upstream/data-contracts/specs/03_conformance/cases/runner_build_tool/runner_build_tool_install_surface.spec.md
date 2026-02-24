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
  - id: DCCONF-BTOOL-007
    title: runner CLI exposes canonical scaffold command surface
    purpose: Runner CLI must expose bundle scaffold command surface for bundle-driven project setup workflows.
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
          - dc-runner bundle scaffold --project-root <path> --bundle-id <id> --bundle-version <semver>
      - id: assert_2
        assert:
          std.string.contains:
          - var: text
          - dc-runner bundle scaffold --project-root <path> --bundle-url <url> --sha256 <hex> --allow-external
adapters:
- type: beta.check_profile_text_file_config_path_specs_02_contracts_29_runner_cli_interface
  actions:
  - id: act.conf.runner.build.tool.instal.1
    direction: bidirectional
    profile: default
services:
- id: svc.conf.runner.build.tool.instal.1
  consumes:
  - act.conf.runner.build.tool.instal.1
```
