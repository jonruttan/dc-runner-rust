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
  - id: DCGOV-BUNDLE-CONTRACT-001
    title: bundle version contract is canonical and pinned
    purpose: Ensures canonical bundle host, explicit semver pins, and hard-fail policy are declared.
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
          - "version: 1"
      - id: assert_2
        assert:
          std.string.contains:
          - var: text
          - "repo: jonruttan/data-contracts-bundles"
      - id: assert_3
        assert:
          std.string.contains:
          - var: text
          - "bundle_id: data-contracts-lang-project-scaffold"
      - id: assert_4
        assert:
          std.string.contains:
          - var: text
          - "bundle_version: 1.0.0"
      - id: assert_5
        assert:
          std.string.contains:
          - var: text
          - "bundle_id: data-contracts-lang-rust-project-scaffold"
      - id: assert_6
        assert:
          std.string.contains:
          - var: text
          - "on_unpublished_version: hard_fail"
adapters:
- type: beta.check_profile_text_file_config
  actions:
  - id: svc.gov.runtime.bundle.version.contract.1
    direction: bidirectional
    profile: default
services:
- id: svc.gov.runtime.bundle.version.contract.1
  consumes:
  - svc.gov.runtime.bundle.version.contract.1
```
