# Runner Certification Rust Job Cases

## DCIMPL-RUST-JOB-CERT-001

```yaml contract-spec
id: DCIMPL-RUST-JOB-CERT-001
title: runner certification command contract smoke
purpose: Codifies the rust runner-certify command surface and expected artifact envelope.
type: contract.check
harness:
  check:
    profile: text.file
    config: {}
contract:
  defaults:
    class: MUST
  imports:
  - from: artifact
    names:
    - text
  steps:
  - id: assert_1
    assert:
    - std.string.contains:
      - {var: text}
      - runner-certify
    - std.string.contains:
      - {var: text}
      - runner certification
```
