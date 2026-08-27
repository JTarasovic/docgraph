+++

id = "plan:close-initial-design-gaps"
type = "plan"
state = "completed"

[properties]
title = "Close the initial design gaps"

[[relations]]
type = "implements"
target = "reference:design#s-DRW3RR84VS"

[[relations]]
type = "implements"
target = "reference:config-grammar#s-P73QA8YDQB"

[[relations]]
type = "implements"
target = "reference:scenarios#s-N6Z4YKP9M0"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "issue:managed-metadata-guardrails"
predicate = "affects"
target = "plan:close-initial-design-gaps"

[[docgraph_generated.incoming]]
source = "issue:multi-file-adoption-normalize-first"
predicate = "affects"
target = "plan:close-initial-design-gaps"

[[docgraph_generated.incoming]]
source = "task:audit-initial-design-closure"
predicate = "implements"
target = "plan:close-initial-design-gaps#s-Q08ZGYHV8W"

[[docgraph_generated.incoming]]
source = "task:audit-initial-design-closure"
predicate = "part_of"
target = "plan:close-initial-design-gaps"

[[docgraph_generated.incoming]]
source = "task:complete-safe-read-mutation-boundary"
predicate = "implements"
target = "plan:close-initial-design-gaps#s-X15NG4P6Y4"

[[docgraph_generated.incoming]]
source = "task:complete-safe-read-mutation-boundary"
predicate = "part_of"
target = "plan:close-initial-design-gaps"

[[docgraph_generated.incoming]]
source = "task:complete-section-relation-roundtrip"
predicate = "implements"
target = "plan:close-initial-design-gaps#s-40M5GN7XPN"

[[docgraph_generated.incoming]]
source = "task:complete-section-relation-roundtrip"
predicate = "part_of"
target = "plan:close-initial-design-gaps"

[[docgraph_generated.incoming]]
source = "task:expand-initial-design-conformance"
predicate = "implements"
target = "plan:close-initial-design-gaps#s-KWVRG8GZN0"

[[docgraph_generated.incoming]]
source = "task:expand-initial-design-conformance"
predicate = "part_of"
target = "plan:close-initial-design-gaps"

[[docgraph_generated.incoming]]
source = "task:implement-derived-index-lifecycle"
predicate = "implements"
target = "plan:close-initial-design-gaps#s-0C1QAG3ZHE"

[[docgraph_generated.incoming]]
source = "task:implement-derived-index-lifecycle"
predicate = "part_of"
target = "plan:close-initial-design-gaps"

[[docgraph_generated.inverses]]
source = "plan:close-initial-design-gaps"
type = "affected_by"
target = "issue:managed-metadata-guardrails"

[[docgraph_generated.inverses]]
source = "plan:close-initial-design-gaps"
type = "affected_by"
target = "issue:multi-file-adoption-normalize-first"

[[docgraph_generated.inverses]]
source = "plan:close-initial-design-gaps"
type = "contains"
target = "task:audit-initial-design-closure"

[[docgraph_generated.inverses]]
source = "plan:close-initial-design-gaps"
type = "contains"
target = "task:complete-safe-read-mutation-boundary"

[[docgraph_generated.inverses]]
source = "plan:close-initial-design-gaps"
type = "contains"
target = "task:complete-section-relation-roundtrip"

[[docgraph_generated.inverses]]
source = "plan:close-initial-design-gaps"
type = "contains"
target = "task:expand-initial-design-conformance"

[[docgraph_generated.inverses]]
source = "plan:close-initial-design-gaps"
type = "contains"
target = "task:implement-derived-index-lifecycle"

[[docgraph_generated.inverses]]
source = "plan:close-initial-design-gaps#s-0C1QAG3ZHE"
type = "implemented_by"
target = "task:implement-derived-index-lifecycle"

[[docgraph_generated.inverses]]
source = "plan:close-initial-design-gaps#s-40M5GN7XPN"
type = "implemented_by"
target = "task:complete-section-relation-roundtrip"

[[docgraph_generated.inverses]]
source = "plan:close-initial-design-gaps#s-KWVRG8GZN0"
type = "implemented_by"
target = "task:expand-initial-design-conformance"

[[docgraph_generated.inverses]]
source = "plan:close-initial-design-gaps#s-Q08ZGYHV8W"
type = "implemented_by"
target = "task:audit-initial-design-closure"

[[docgraph_generated.inverses]]
source = "plan:close-initial-design-gaps#s-X15NG4P6Y4"
type = "implemented_by"
target = "task:complete-safe-read-mutation-boundary"

[[docgraph_generated.backlinks]]
source = "issue:managed-metadata-guardrails#s-44777ZK3XR"
target = "docs/plans/close-initial-design-gaps.md"

[[docgraph_generated.backlinks]]
source = "issue:multi-file-adoption-normalize-first#s-3V3RP8C4NH"
target = "docs/plans/close-initial-design-gaps.md"

[[docgraph_generated.backlinks]]
source = "reference:design#s-DRW3RR84VS"
target = "docs/plans/close-initial-design-gaps.md"

+++
<a id="s-EESDQJ04F8"></a>
# Close the initial design gaps

<a id="s-AEYS83XV2V"></a>
## Objective

Finish the cohesive mechanics promised by the initial reference set without pulling explicitly deferred product directions back into scope.

<a id="s-XHH0TTSDTM"></a>
## Scope

Cover persistent derived state, the read/recovery boundary, exact stable-section relation round trips, and conformance coverage. Structural document editing, provider adapters, vectors, dynamic commands, semantic diff/merge, and expanded retrieval convenience commands remain outside this plan.

<a id="s-G6Z4NGDY9V"></a>
## Steps

<a id="s-0C1QAG3ZHE"></a>
### Implement the derived-index lifecycle

[Implement the derived-index lifecycle](../tasks/implement-derived-index-lifecycle.md) with a real per-worktree SQLite graph and FTS index, deterministic rebuilds, and fingerprint-aware refresh.

<a id="s-X15NG4P6Y4"></a>
### Complete the safe read and mutation boundary

[Complete the safe read and mutation boundary](../tasks/complete-safe-read-mutation-boundary.md) so interrupted writes are recovered or refused before reads and mutations, and derived state is never silently stale.

<a id="s-40M5GN7XPN"></a>
### Complete stable-section relation round trips

[Complete stable-section relation round trips](../tasks/complete-section-relation-roundtrip.md) from authored frontmatter through CLI mutation, graph retrieval, and exact generated projections.

The safety-boundary step follows the derived-index foundation. Section-relation work may proceed independently.

<a id="s-KWVRG8GZN0"></a>
### Expand initial-design conformance

[Expand initial-design conformance](../tasks/expand-initial-design-conformance.md) across richer fixtures, transaction failures, worktrees, and runtime-backed CI.

<a id="s-Q08ZGYHV8W"></a>
### Audit initial-design closure

[Audit initial-design closure](../tasks/audit-initial-design-closure.md) against the accounted reference scope and record any remaining gap or deliberate deferral.

<a id="s-ZWY7GQ85Y0"></a>
## Completion

All four implementation increments are exercised by conformance tests and CI, the repository dogfood model passes, and the closure audit finds no unaccounted initial-design promise.
