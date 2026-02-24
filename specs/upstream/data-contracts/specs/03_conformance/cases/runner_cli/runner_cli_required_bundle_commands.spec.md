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
    title: runner cli exposes bundle info command
    purpose: Portable CLI contract requires bundle info surface.
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
          - dc-runner bundle info --bundle-id <id>
  - id: DCCONF-RCLI-010
    title: runner cli exposes lock-driven bundle install command
    purpose: Portable CLI contract requires lock-driven bundle install surface.
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
          - dc-runner bundle install --project-lock <path> --out <path>
  - id: DCCONF-RCLI-011
    title: runner cli exposes bundle install-check command
    purpose: Portable CLI contract requires install-check surface.
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
          - dc-runner bundle install-check --project-lock <path> --out <path>
  - id: DCCONF-RCLI-012
    title: runner cli exposes bundle bootstrap command
    purpose: Portable CLI contract requires bootstrap surface.
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
          - dc-runner bundle bootstrap --lock <path> --out <path>
  - id: DCCONF-RCLI-013
    title: runner cli exposes bundle bootstrap-check command
    purpose: Portable CLI contract requires bootstrap-check surface.
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
          - dc-runner bundle bootstrap-check --lock <path> --out <path>
  - id: DCCONF-RCLI-014
    title: runner cli exposes bundle outdated command
    purpose: Portable CLI contract requires outdated surface.
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
          - dc-runner bundle outdated --project-lock <path> [--format json|table]
  - id: DCCONF-RCLI-015
    title: runner cli exposes bundle upgrade command
    purpose: Portable CLI contract requires upgrade surface.
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
          - dc-runner bundle upgrade --project-lock <path> [--dry-run]
  - id: DCCONF-RCLI-016
    title: runner cli exposes bundle run command
    purpose: Portable CLI contract requires no-install bundle execution surface.
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
          - dc-runner bundle run --bundle-id <id> --bundle-version <semver> --entrypoint <name> [--arg <value>]...
  - id: DCCONF-RCLI-017
    title: runner cli exposes bundle scaffold command
    purpose: Portable CLI contract requires canonical bundle scaffold surface.
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
          - dc-runner bundle scaffold --project-root <path> --bundle-id <id> --bundle-version <semver> [--runner <rust|python|php>] [--var <key=value>]... [--overwrite]
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
