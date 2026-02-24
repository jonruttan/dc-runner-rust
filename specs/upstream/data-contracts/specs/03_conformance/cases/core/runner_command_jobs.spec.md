```yaml contract-spec
id: DCCONF-JOB-006
title: docs lint command via contract.job
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
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCCONF-JOB-006.fail.json
        format: json
        report_name: DCCONF-JOB-006.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCCONF-JOB-006.complete.json
        format: json
        report_name: DCCONF-JOB-006.complete
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
id: DCCONF-JOB-007
title: docs generate command via contract.job
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
        out: .artifacts/job-hooks/DCCONF-JOB-007.fail.json
        format: json
        report_name: DCCONF-JOB-007.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCCONF-JOB-007.complete.json
        format: json
        report_name: DCCONF-JOB-007.complete
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
id: DCCONF-JOB-008
title: docs generate-check command via contract.job
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
        out: .artifacts/job-hooks/DCCONF-JOB-008.fail.json
        format: json
        report_name: DCCONF-JOB-008.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCCONF-JOB-008.complete.json
        format: json
        report_name: DCCONF-JOB-008.complete
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
id: DCCONF-JOB-009
title: docs build command via contract.job
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
        out: .artifacts/job-hooks/DCCONF-JOB-009.fail.json
        format: json
        report_name: DCCONF-JOB-009.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCCONF-JOB-009.complete.json
        format: json
        report_name: DCCONF-JOB-009.complete
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
id: DCCONF-JOB-010
title: docs build-check command via contract.job
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
        out: .artifacts/job-hooks/DCCONF-JOB-010.fail.json
        format: json
        report_name: DCCONF-JOB-010.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCCONF-JOB-010.complete.json
        format: json
        report_name: DCCONF-JOB-010.complete
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
id: DCCONF-JOB-011
title: docs graph command via contract.job
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
        out: .artifacts/job-hooks/DCCONF-JOB-011.fail.json
        format: json
        report_name: DCCONF-JOB-011.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCCONF-JOB-011.complete.json
        format: json
        report_name: DCCONF-JOB-011.complete
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
id: DCCONF-JOB-012
title: schema check command via contract.job
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
        out: .artifacts/job-hooks/DCCONF-JOB-012.fail.json
        format: json
        report_name: DCCONF-JOB-012.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCCONF-JOB-012.complete.json
        format: json
        report_name: DCCONF-JOB-012.complete
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
id: DCCONF-JOB-013
title: schema lint command via contract.job
type: contract.job
harness:
  spec_lang:
    capabilities:
      - ops.helper
      - ops.job
  jobs:
    main:
      mode: check
      helper: helper.schema.normalize_runner
      inputs:
        mode: check
        path: /specs
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCCONF-JOB-013.fail.json
        format: json
        report_name: DCCONF-JOB-013.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCCONF-JOB-013.complete.json
        format: json
        report_name: DCCONF-JOB-013.complete
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
id: DCCONF-JOB-014
title: schema format command via contract.job
type: contract.job
harness:
  spec_lang:
    capabilities:
      - ops.helper
      - ops.job
  jobs:
    main:
      mode: fix
      helper: helper.schema.normalize_runner
      inputs:
        mode: fix
        path: /specs
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCCONF-JOB-014.fail.json
        format: json
        report_name: DCCONF-JOB-014.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCCONF-JOB-014.complete.json
        format: json
        report_name: DCCONF-JOB-014.complete
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
id: DCCONF-JOB-015
title: bundle list command via contract.job
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
        out: .artifacts/job-hooks/DCCONF-JOB-015.main.json
        format: json
        report_name: bundle.list
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCCONF-JOB-015.fail.json
        format: json
        report_name: DCCONF-JOB-015.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/bundle-list.json
        format: json
        report_name: DCCONF-JOB-015.complete
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
id: DCCONF-JOB-016
title: bundle install command via contract.job
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
        out: .artifacts/job-hooks/DCCONF-JOB-016.main.json
        format: json
        report_name: bundle.install
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCCONF-JOB-016.fail.json
        format: json
        report_name: DCCONF-JOB-016.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/bundle-install.json
        format: json
        report_name: DCCONF-JOB-016.complete
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
id: DCCONF-JOB-017
title: bundle info command via contract.job
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
        out: .artifacts/job-hooks/DCCONF-JOB-017.main.json
        format: json
        report_name: bundle.info
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCCONF-JOB-017.fail.json
        format: json
        report_name: DCCONF-JOB-017.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/bundle-info.json
        format: json
        report_name: DCCONF-JOB-017.complete
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
id: DCCONF-JOB-019
title: bundle install-check command via contract.job
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
        out: .artifacts/job-hooks/DCCONF-JOB-019.main.json
        format: json
        report_name: bundle.install-check
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCCONF-JOB-019.fail.json
        format: json
        report_name: DCCONF-JOB-019.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/bundle-install-check.json
        format: json
        report_name: DCCONF-JOB-019.complete
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
id: DCCONF-JOB-020
title: bundle bootstrap command via contract.job
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
        out: .artifacts/job-hooks/DCCONF-JOB-020.main.json
        format: json
        report_name: bundle.bootstrap
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCCONF-JOB-020.fail.json
        format: json
        report_name: DCCONF-JOB-020.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/bundle-bootstrap.json
        format: json
        report_name: DCCONF-JOB-020.complete
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
id: DCCONF-JOB-021
title: bundle bootstrap-check command via contract.job
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
        out: .artifacts/job-hooks/DCCONF-JOB-021.main.json
        format: json
        report_name: bundle.bootstrap-check
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCCONF-JOB-021.fail.json
        format: json
        report_name: DCCONF-JOB-021.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/bundle-bootstrap-check.json
        format: json
        report_name: DCCONF-JOB-021.complete
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
id: DCCONF-JOB-022
title: bundle outdated command via contract.job
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
        out: .artifacts/job-hooks/DCCONF-JOB-022.main.json
        format: json
        report_name: bundle.outdated
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCCONF-JOB-022.fail.json
        format: json
        report_name: DCCONF-JOB-022.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/bundle-outdated.json
        format: json
        report_name: DCCONF-JOB-022.complete
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
id: DCCONF-JOB-023
title: bundle upgrade command via contract.job
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
        out: .artifacts/job-hooks/DCCONF-JOB-023.main.json
        format: json
        report_name: bundle.upgrade
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCCONF-JOB-023.fail.json
        format: json
        report_name: DCCONF-JOB-023.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/bundle-upgrade.json
        format: json
        report_name: DCCONF-JOB-023.complete
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
id: DCCONF-JOB-024
title: bundle run command via contract.job
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
        out: .artifacts/job-hooks/DCCONF-JOB-024.main.json
        format: json
        report_name: bundle.run
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCCONF-JOB-024.fail.json
        format: json
        report_name: DCCONF-JOB-024.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/bundle-run.json
        format: json
        report_name: DCCONF-JOB-024.complete
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
id: DCCONF-JOB-025
title: bundle scaffold command via contract.job
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
        out: .artifacts/job-hooks/DCCONF-JOB-025.main.json
        format: json
        report_name: bundle.scaffold
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCCONF-JOB-025.fail.json
        format: json
        report_name: DCCONF-JOB-025.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/bundle-scaffold.json
        format: json
        report_name: DCCONF-JOB-025.complete
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
id: DCCONF-JOB-018
title: quality lint command via contract.job
type: contract.job
harness:
  spec_lang:
    capabilities:
      - ops.helper
      - ops.job
  jobs:
    main:
      mode: lint
      helper: helper.schema.lint
      inputs:
        mode: strict
        path: /specs
    on_fail:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCCONF-JOB-018.fail.json
        format: json
        report_name: DCCONF-JOB-018.fail
    on_complete:
      helper: helper.report.emit
      mode: report
      inputs:
        out: .artifacts/job-hooks/DCCONF-JOB-018.complete.json
        format: json
        report_name: DCCONF-JOB-018.complete
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
