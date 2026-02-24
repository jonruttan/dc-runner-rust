```yaml contract-spec
id: DCLIB-BUNDLE-JOB-001
title: bundler resolve command via contract.job
type: contract.job
harness:
  spec_lang:
    capabilities:
      - ops.helper
      - ops.job
  jobs:
    main:
      mode: build
      helper: helper.report.emit
      inputs:
        out: .artifacts/bundler-resolve.json
        format: json
        report_name: bundler.resolve
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCLIB-BUNDLE-JOB-001.fail.json
        format: json
        report_name: DCLIB-BUNDLE-JOB-001.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCLIB-BUNDLE-JOB-001.complete.json
        format: json
        report_name: DCLIB-BUNDLE-JOB-001.complete
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
                - { var: summary_json }
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

```yaml contract-spec
id: DCLIB-BUNDLE-JOB-002
title: bundler package command via contract.job
type: contract.job
harness:
  spec_lang:
    capabilities:
      - ops.helper
      - ops.job
  jobs:
    main:
      mode: build
      helper: helper.report.emit
      inputs:
        out: .artifacts/bundler-package.json
        format: json
        report_name: bundler.package
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCLIB-BUNDLE-JOB-002.fail.json
        format: json
        report_name: DCLIB-BUNDLE-JOB-002.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCLIB-BUNDLE-JOB-002.complete.json
        format: json
        report_name: DCLIB-BUNDLE-JOB-002.complete
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
                - { var: summary_json }
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

```yaml contract-spec
id: DCLIB-BUNDLE-JOB-003
title: bundler check command via contract.job
type: contract.job
harness:
  spec_lang:
    capabilities:
      - ops.helper
      - ops.job
  jobs:
    main:
      mode: check
      helper: helper.report.emit
      inputs:
        out: .artifacts/bundler-check.json
        format: json
        report_name: bundler.check
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCLIB-BUNDLE-JOB-003.fail.json
        format: json
        report_name: DCLIB-BUNDLE-JOB-003.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCLIB-BUNDLE-JOB-003.complete.json
        format: json
        report_name: DCLIB-BUNDLE-JOB-003.complete
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
                - { var: summary_json }
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
