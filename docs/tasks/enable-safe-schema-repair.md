+++

id = "task:enable-safe-schema-repair"
type = "task"
state = "done"

[properties]
title = "Enable safe schema repair"

[[relations]]
type = "part_of"
target = "plan:resolve-initial-github-report-backlog"

[[relations]]
type = "implements"
target = "plan:resolve-initial-github-report-backlog#s-9S79WDF4RE"

[docgraph_generated]
schema_version = 1

+++
<a id="s-ZFKAT6EQTT"></a>
# Enable safe schema repair

Address [GitHub issue #2](https://github.com/JTarasovic/docgraph/issues/2).

<a id="s-Z6EYSSNPQ7"></a>
## Outcome

A repository with multiple values invalidated by a tightened property schema can be
repaired without removing the schema constraint and without allowing a mutation to
introduce new validation failures.

<a id="s-BK45YXKDJE"></a>
## Scope

- Define the safety rule for mutations made while the repository already contains
  relevant validation failures.
- Implement either an atomic migration operation or a repair mode that proves each
  mutation reduces the applicable error set.
- Preserve optimistic hashes, recovery journals, whole-repository validation for
  ordinary mutations, and clear failure diagnostics.
- Document the supported migration workflow and rejected unsafe cases.

<a id="s-7884MPG3ZX"></a>
## Acceptance

- The two-invalid-document reproduction can be repaired while the enum remains active.
- A repair cannot introduce a new error, worsen an existing error, or bypass unrelated
  repository invariants.
- Single-document repair, interrupted writes, dry runs, and concurrent-edit detection
  have regression coverage.
