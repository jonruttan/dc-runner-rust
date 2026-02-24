# Terminal prompt behavior coverage

```yaml contract-spec
spec_version: 1
schema_ref: "/specs/01_schema/schema_v1.md"
harness:
  type: unit.test
  profile: check
contracts:
  clauses:
  - id: DCCONF-TERM-001
    title: terminal.prompt contract documents plain/rich modes and raw output
    purpose: Ensures `domain.terminal.prompt` is documented with `plain|rich` mode
      and raw terminal input return shape.
    expect:
      portable:
        status: pass
    asserts:
      imports:
      - from: asset
        names:
        - text
      checks:
      - id: assert_1
        assert:
        - std.string.contains:
          - var: text
          - "domain.terminal.prompt"
      - id: assert_2
        assert:
        - std.string.contains:
          - var: text
          - "plain"
      - id: assert_3
        assert:
        - std.string.contains:
          - var: text
          - "rich"
      - id: assert_4
        assert:
        - std.string.contains:
          - var: text
          - "Returns: Raw input text as a string."
      - id: assert_5
        assert:
        - std.string.contains:
          - var: text
          - "default: plain"
  - id: DCCONF-TERM-002
    title: terminal prompt symbol is discoverable in domain index
    purpose: Ensures domain library index entries stay synchronized with terminal
      prompt.
    expect:
      portable:
        status: pass
    asserts:
      imports:
      - from: asset
        names:
        - text
      checks:
      - id: assert_1
        assert:
        - std.string.contains:
          - var: text
          - "/specs/05_libraries/domain/terminal_prompt.spec.md"
      - id: assert_2
        assert:
        - std.string.contains:
          - var: text
          - "domain.terminal.prompt"
adapters:
- type: io.fs
  defaults:
    direction: input
    profile: read.text
  actions:
  - id: svc.assert_check.text_file.1
    config:
      source_asset_id: art.svc.assert_check.text_file.1.source.1
  - id: svc.assert_check.text_file.2
    config:
      source_asset_id: art.svc.assert_check.text_file.2.source.1
services:
- id: svc.assert_check.text_file.1
  consumes:
  - svc.assert_check.text_file.1
- id: svc.assert_check.text_file.2
  consumes:
  - svc.assert_check.text_file.2
assets:
- id: art.svc.assert_check.text_file.1.source.1
  ref: "/specs/05_libraries/domain/terminal_prompt.spec.md"
- id: art.svc.assert_check.text_file.2.source.1
  ref: "/specs/05_libraries/domain/index.md"
```
