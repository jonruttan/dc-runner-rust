# Rust Job Script Cases

## DCIMPL-RUST-JOB-001

```yaml contract-spec
id: DCIMPL-RUST-JOB-001
title: governance scan bundle helper smoke
purpose: Contract job entrypoint for Rust-native helper dispatch and scalar path#id job refs.
type: contract.job
harness:
  spec_lang:
    capabilities:
    - ops.helper
    - ops.job
  jobs:
    main:
      mode: check
      helper: helper.governance.scan_bundle
      inputs:
        path: /specs
        patterns:
        - contract-spec
      outputs:
        summary: .artifacts/job-scan-summary.json
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCIMPL-RUST-JOB-001.fail.json
        format: json
        report_name: DCIMPL-RUST-JOB-001.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCIMPL-RUST-JOB-001.complete.json
        format: json
        report_name: DCIMPL-RUST-JOB-001.complete
contract:
  defaults:
    class: MUST
  imports:
  - from: artifact
    names:
    - summary_json
  steps:
  - id: assert_1
    assert:
    - ops.job.dispatch:
      - main
    - std.logic.neq:
      - std.object.get:
        - {var: summary_json}
        - scanned_files
      - null
when:
  fail:
  - ops.job.dispatch:
    - on_fail
  complete:
  - ops.job.dispatch:
    - on_complete
```

## DCIMPL-RUST-JOB-002

```yaml contract-spec
id: DCIMPL-RUST-JOB-002
title: conformance parity command via contract.job
type: contract.job
harness:
  spec_lang:
    capabilities:
    - ops.helper
    - ops.job
  jobs:
    main:
      mode: check
      helper: helper.parity.run_conformance
      inputs:
        cases: specs/conformance/cases
        php_runner: dc-runner-php/conformance_runner.php
        out: .artifacts/conformance-parity.json
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCIMPL-RUST-JOB-002.fail.json
        format: json
        report_name: DCIMPL-RUST-JOB-002.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCIMPL-RUST-JOB-002.complete.json
        format: json
        report_name: DCIMPL-RUST-JOB-002.complete
contract:
  defaults:
    class: MUST
  imports:
  - from: artifact
    names:
    - summary_json
  steps:
  - id: assert_1
    assert:
    - ops.job.dispatch:
      - main
    - std.logic.eq:
      - std.object.get:
        - {var: summary_json}
        - ok
      - true
when:
  fail:
  - ops.job.dispatch:
    - on_fail
  complete:
  - ops.job.dispatch:
    - on_complete
```

## DCIMPL-RUST-JOB-003

```yaml contract-spec
id: DCIMPL-RUST-JOB-003
title: perf smoke command via contract.job
type: contract.job
harness:
  spec_lang:
    capabilities:
    - ops.helper
    - ops.job
  jobs:
    main:
      mode: warn
      helper: helper.perf.run_smoke
      inputs:
        report_out: .artifacts/perf-smoke-report.json
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCIMPL-RUST-JOB-003.fail.json
        format: json
        report_name: DCIMPL-RUST-JOB-003.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCIMPL-RUST-JOB-003.complete.json
        format: json
        report_name: DCIMPL-RUST-JOB-003.complete
contract:
  defaults:
    class: MUST
  imports:
  - from: artifact
    names:
    - summary_json
  steps:
  - id: assert_1
    assert:
    - ops.job.dispatch:
      - main
    - std.logic.eq:
      - std.object.get:
        - {var: summary_json}
        - ok
      - true
when:
  fail:
  - ops.job.dispatch:
    - on_fail
  complete:
  - ops.job.dispatch:
    - on_complete
```

## DCIMPL-RUST-JOB-004

```yaml contract-spec
id: DCIMPL-RUST-JOB-004
title: schema registry build via contract.job
type: contract.job
harness:
  spec_lang:
    capabilities:
    - ops.helper
    - ops.job
  jobs:
    main:
      mode: build
      helper: helper.schema.registry_report
      inputs:
        format: json
        out: .artifacts/schema_registry_report.json
        check: false
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCIMPL-RUST-JOB-004.fail.json
        format: json
        report_name: DCIMPL-RUST-JOB-004.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCIMPL-RUST-JOB-004.complete.json
        format: json
        report_name: DCIMPL-RUST-JOB-004.complete
contract:
  defaults:
    class: MUST
  imports:
  - from: artifact
    names:
    - summary_json
  steps:
  - id: assert_1
    assert:
    - ops.job.dispatch:
      - main
    - std.logic.eq:
      - std.object.get:
        - {var: summary_json}
        - ok
      - true
when:
  fail:
  - ops.job.dispatch:
    - on_fail
  complete:
  - ops.job.dispatch:
    - on_complete
```

## DCIMPL-RUST-JOB-005

```yaml contract-spec
id: DCIMPL-RUST-JOB-005
title: schema registry check via contract.job
type: contract.job
harness:
  spec_lang:
    capabilities:
    - ops.helper
    - ops.job
  jobs:
    main:
      mode: check
      helper: helper.schema.registry_report
      inputs:
        format: json
        out: .artifacts/schema_registry_report.json
        check: true
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCIMPL-RUST-JOB-005.fail.json
        format: json
        report_name: DCIMPL-RUST-JOB-005.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCIMPL-RUST-JOB-005.complete.json
        format: json
        report_name: DCIMPL-RUST-JOB-005.complete
contract:
  defaults:
    class: MUST
  imports:
  - from: artifact
    names:
    - summary_json
  steps:
  - id: assert_1
    assert:
    - ops.job.dispatch:
      - main
    - std.logic.eq:
      - std.object.get:
        - {var: summary_json}
        - ok
      - true
when:
  fail:
  - ops.job.dispatch:
    - on_fail
  complete:
  - ops.job.dispatch:
    - on_complete
```

## DCIMPL-RUST-JOB-006

```yaml contract-spec
id: DCIMPL-RUST-JOB-006
title: docs lint via contract.job
type: contract.job
harness:
  spec_lang:
    capabilities:
    - ops.helper
    - ops.job
  jobs:
    main:
      mode: lint
      helper: helper.docs.lint
      inputs: {}
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCIMPL-RUST-JOB-006.fail.json
        format: json
        report_name: DCIMPL-RUST-JOB-006.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCIMPL-RUST-JOB-006.complete.json
        format: json
        report_name: DCIMPL-RUST-JOB-006.complete
contract:
  defaults:
    class: MUST
  imports:
  - from: artifact
    names:
    - summary_json
  steps:
  - id: assert_1
    assert:
    - ops.job.dispatch:
      - main
    - std.logic.eq:
      - std.object.get:
        - {var: summary_json}
        - ok
      - true
when:
  fail:
  - ops.job.dispatch:
    - on_fail
  complete:
  - ops.job.dispatch:
    - on_complete
```

## DCIMPL-RUST-JOB-007

```yaml contract-spec
id: DCIMPL-RUST-JOB-007
title: docs generate build via contract.job
type: contract.job
harness:
  spec_lang:
    capabilities:
    - ops.helper
    - ops.job
  jobs:
    main:
      mode: build
      helper: helper.docs.generate_all
      inputs:
        action: build
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCIMPL-RUST-JOB-007.fail.json
        format: json
        report_name: DCIMPL-RUST-JOB-007.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCIMPL-RUST-JOB-007.complete.json
        format: json
        report_name: DCIMPL-RUST-JOB-007.complete
contract:
  defaults:
    class: MUST
  imports:
  - from: artifact
    names:
    - summary_json
  steps:
  - id: assert_1
    assert:
    - ops.job.dispatch:
      - main
    - std.logic.eq:
      - std.object.get:
        - {var: summary_json}
        - ok
      - true
when:
  fail:
  - ops.job.dispatch:
    - on_fail
  complete:
  - ops.job.dispatch:
    - on_complete
```

## DCIMPL-RUST-JOB-008

```yaml contract-spec
id: DCIMPL-RUST-JOB-008
title: docs generate check via contract.job
type: contract.job
harness:
  spec_lang:
    capabilities:
    - ops.helper
    - ops.job
  jobs:
    main:
      mode: check
      helper: helper.docs.generate_all
      inputs:
        action: check
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCIMPL-RUST-JOB-008.fail.json
        format: json
        report_name: DCIMPL-RUST-JOB-008.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCIMPL-RUST-JOB-008.complete.json
        format: json
        report_name: DCIMPL-RUST-JOB-008.complete
contract:
  defaults:
    class: MUST
  imports:
  - from: artifact
    names:
    - summary_json
  steps:
  - id: assert_1
    assert:
    - ops.job.dispatch:
      - main
    - std.logic.eq:
      - std.object.get:
        - {var: summary_json}
        - ok
      - true
when:
  fail:
  - ops.job.dispatch:
    - on_fail
  complete:
  - ops.job.dispatch:
    - on_complete
```

## DCIMPL-RUST-JOB-009

```yaml contract-spec
id: DCIMPL-RUST-JOB-009
title: docs build reference book via contract.job
type: contract.job
harness:
  spec_lang:
    capabilities:
    - ops.helper
    - ops.job
  jobs:
    main:
      mode: build
      helper: helper.docs.generate_all
      inputs:
        action: build
        surface: reference_book
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCIMPL-RUST-JOB-009.fail.json
        format: json
        report_name: DCIMPL-RUST-JOB-009.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCIMPL-RUST-JOB-009.complete.json
        format: json
        report_name: DCIMPL-RUST-JOB-009.complete
contract:
  defaults:
    class: MUST
  imports:
  - from: artifact
    names:
    - summary_json
  steps:
  - id: assert_1
    assert:
    - ops.job.dispatch:
      - main
    - std.logic.eq:
      - std.object.get:
        - {var: summary_json}
        - ok
      - true
when:
  fail:
  - ops.job.dispatch:
    - on_fail
  complete:
  - ops.job.dispatch:
    - on_complete
```

## DCIMPL-RUST-JOB-010

```yaml contract-spec
id: DCIMPL-RUST-JOB-010
title: docs build check reference book via contract.job
type: contract.job
harness:
  spec_lang:
    capabilities:
    - ops.helper
    - ops.job
  jobs:
    main:
      mode: check
      helper: helper.docs.generate_all
      inputs:
        action: check
        surface: reference_book
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCIMPL-RUST-JOB-010.fail.json
        format: json
        report_name: DCIMPL-RUST-JOB-010.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCIMPL-RUST-JOB-010.complete.json
        format: json
        report_name: DCIMPL-RUST-JOB-010.complete
contract:
  defaults:
    class: MUST
  imports:
  - from: artifact
    names:
    - summary_json
  steps:
  - id: assert_1
    assert:
    - ops.job.dispatch:
      - main
    - std.logic.eq:
      - std.object.get:
        - {var: summary_json}
        - ok
      - true
when:
  fail:
  - ops.job.dispatch:
    - on_fail
  complete:
  - ops.job.dispatch:
    - on_complete
```

## DCIMPL-RUST-JOB-011

```yaml contract-spec
id: DCIMPL-RUST-JOB-011
title: docs graph export via contract.job
type: contract.job
harness:
  spec_lang:
    capabilities:
    - ops.helper
    - ops.job
  jobs:
    main:
      mode: build
      helper: helper.docs.generate_all
      inputs:
        action: build
        surface: docs_graph
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCIMPL-RUST-JOB-011.fail.json
        format: json
        report_name: DCIMPL-RUST-JOB-011.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCIMPL-RUST-JOB-011.complete.json
        format: json
        report_name: DCIMPL-RUST-JOB-011.complete
contract:
  defaults:
    class: MUST
  imports:
  - from: artifact
    names:
    - summary_json
  steps:
  - id: assert_1
    assert:
    - ops.job.dispatch:
      - main
    - std.logic.eq:
      - std.object.get:
        - {var: summary_json}
        - ok
      - true
when:
  fail:
  - ops.job.dispatch:
    - on_fail
  complete:
  - ops.job.dispatch:
    - on_complete
```
