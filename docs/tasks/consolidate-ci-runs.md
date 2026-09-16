+++

id = "task:consolidate-ci-runs"
type = "task"
state = "in_progress"

[properties]
title = "Consolidate short CI runs and add required checks"

[docgraph_generated]
schema_version = 1

+++
<a id="s-1QN9Z4H9HB"></a>
# Consolidate short CI runs and add required checks

Address [#33](https://github.com/JTarasovic/docgraph/issues/33). Two sub-30-second
jobs — the `action-failures` validation check (`ci.yml`) and the `conventional-title`
PR-title check (`pr-title.yml`) — each bill a full billable minute and run as separate
jobs. Consolidate them, keep the required merge gate coherent with any renamed check
contexts, and clean up noisy step output across the workflows.

<a id="s-0629N75XTX"></a>
## Acceptance

- The fast validation-action and PR-title checks run as a single `quality-gate` job,
  billing one minute instead of two, and still run on every pull request (including
  title edits so a failing title can be fixed).
- `ci.yml` retains only the `rust` job; `pr-title.yml` is retired.
- The "Protect main" ruleset requires `rust` and `quality-gate`; the rename lands in
  lockstep so no PR is blocked on a check that never reports.
- `windows-e2e` stays advisory, with a comment recording that it is intentionally not
  required and must be added if its path filtering ever changes.
- Workflow runs produce no spurious failure annotations or garbage output.
