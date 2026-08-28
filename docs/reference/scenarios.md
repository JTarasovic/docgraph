+++

id = "reference:scenarios"
type = "reference"

[properties]
role = "conformance"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "plan:address-post-v0-reference-work"
predicate = "implements"
target = "reference:scenarios#s-N6Z4YKP9M0"

[[docgraph_generated.incoming]]
source = "plan:close-initial-design-gaps"
predicate = "implements"
target = "reference:scenarios#s-N6Z4YKP9M0"

[[docgraph_generated.incoming]]
source = "task:audit-initial-design-closure"
predicate = "implements"
target = "reference:scenarios#s-N6Z4YKP9M0"

[[docgraph_generated.incoming]]
source = "task:expand-initial-design-conformance"
predicate = "implements"
target = "reference:scenarios#s-9P22A3H49K"

[[docgraph_generated.incoming]]
source = "task:expand-initial-design-conformance"
predicate = "implements"
target = "reference:scenarios#s-N6Z4YKP9M0"

[[docgraph_generated.inverses]]
source = "reference:scenarios#s-9P22A3H49K"
type = "implemented_by"
target = "task:expand-initial-design-conformance"

[[docgraph_generated.inverses]]
source = "reference:scenarios#s-N6Z4YKP9M0"
type = "implemented_by"
target = "plan:address-post-v0-reference-work"

[[docgraph_generated.inverses]]
source = "reference:scenarios#s-N6Z4YKP9M0"
type = "implemented_by"
target = "plan:close-initial-design-gaps"

[[docgraph_generated.inverses]]
source = "reference:scenarios#s-N6Z4YKP9M0"
type = "implemented_by"
target = "task:audit-initial-design-closure"

[[docgraph_generated.inverses]]
source = "reference:scenarios#s-N6Z4YKP9M0"
type = "implemented_by"
target = "task:expand-initial-design-conformance"

+++
<a id="s-W2JDJP657B"></a>
# Draft Scenarios and Conformance Suite

**Status:** Draft

**Scope convention:** This document keeps the complete directional scenario set.
The v0 conformance slice in Section 20 defines the release contract. Sections or
cases identified as post-v0 remain design targets, not v0 acceptance requirements.

<a id="s-ZV6AWWNBQT"></a>
## 1. Purpose

A stable set of fixture repositories should define what “generic” means and prevent development from overfitting to one workflow.

The suite must collectively exercise:

- stable identity
- arbitrary and typed relations
- workflow transitions
- recursive traversal
- derived state
- inference
- section references
- edge properties
- search
- mutation
- validation
- Git behavior
- explainability
- dynamically generated commands

These should become executable end-to-end fixture repositories.

<a id="s-YBDS8T5BK3"></a>
## 2. Architecture Decisions

Entities: ADR, Task, Specification.

Relations: supersedes, blocked_by, implements, references.

Workflow:

```text
proposed → accepted
proposed → rejected
accepted → superseded
```

Tests: legal/illegal transitions, inverse relations, downstream impact, supersession traversal, stable identity across moves, section references across heading renames, automatic unblocking, blocker explanation.

<a id="s-K62BXQKZGB"></a>
## 3. Task Dependency Graph

```text
Task-A
 ├─ depends_on → Task-B
 ├─ depends_on → Task-C
 └─ blocked_by → ADR-42
```

Derived:

```text
actionable(Task-A) =
  Task-B complete
  AND Task-C complete
  AND ADR-42 accepted
```

Tests: recursive dependencies, cycles, derived readiness, direct/transitive blockers, `explain`, downstream changes.

<a id="s-P7TNAVGGZ9"></a>
## 4. Requirements Traceability

Entities: Requirement, Component, Test, Evidence.

```text
Requirement
    ↓ implemented_by
Component
    ↓ verified_by
Test
    ↓ produces
Evidence
```

Queries include uncovered requirements, missing verification, evidence chains, and impact of requirement changes.

<a id="s-99B9NPC7EJ"></a>
## 5. Specification Section → Implementation

Entities: Document, Section, Task, SourceArtifact.

```text
Spec#retry-policy
      ↑ implements
Task-31
      ↓ modifies
src/retry.rs
```

Tests: normalization of stable IDs for every heading, relative/logical references, Markdown backlinks, semantic edges, heading rename, dangling refs, exact source spans.

<a id="s-M4NVGRJE96"></a>
## 6. Human-Agent Proposal and Approval

```text
Agent
  ↓ proposes
ADR-42 [proposed]
  ↓ blocks
Task-184
```

Tests: proposal state, approval/rejection, dry-run impact, derived updates, Git-reviewable mutation, future review of proposed graph mutations.

<a id="s-K780D8XY84"></a>
## 7. Incident and Remediation

Entities: Incident, Component, Task, Decision, RequirementGap.

Relations: caused_by, revealed, remediated_by, resulted_in, related_to.

This stresses evolving ontology and heterogeneous relationships not known to the executable.

<a id="s-PD8SW07HEH"></a>
## 8. Compliance and Assurance

Entities: Requirement, Control, Procedure, Evidence, Test.

Derived example:

```text
effective(Control) =
  implementation exists
  AND current evidence exists
  AND required tests pass
```

Tests relation properties, temporal facts, completeness, inference, and explanation.

<a id="s-6CAQR9GBED"></a>
## 9. Research and Design Rationale

Entities: Claim, Evidence, Source, Decision.

Relations: supported_by, contradicted_by, derived_from, motivates.

This stresses edge properties, competing evidence, section-level citations, and graph + FTS + vector retrieval.

<a id="s-CHY0WX7VZ9"></a>
## 10. Release Planning

Entities: Release, Milestone, Task, Decision, Requirement.

Queries: release blockers, transitive required work, decision dependencies, minimal blocking frontier, readiness explanation.

<a id="s-B3R38XW473"></a>
## 11. Documentation Maintenance

Entities: Document, Section, Concept.

Tests: Markdown links, backlinks, non-failing broken-link diagnostics, superseded
references, semantic vs casual links, stable structural references, and text
retrieval. Semantic retrieval is a post-v0 extension.

<a id="s-35SY325PT6"></a>
## 12. Historical Research

A non-software fixture prevents accidental software-development assumptions.

Entities: Person, Event, Place, Claim, Source, Document.

Relations: participated_in, occurred_at, claims, supported_by, contradicts, cites, possibly_same_as.

Supporting this fixture must require no domain-specific Rust code.

<a id="s-Z037AH2GE5"></a>
## 13. Synthetic Genericity Torture Test

An intentionally meaningless ontology prevents both developers and agents from relying on semantic intuition.

Entities:

```text
Florp
Nizzle
Quux
Borp
Zibble
Wumpus
```

Relations:

```text
florps
snargs
quuxes
wibbles
grommits
```

Workflows:

```text
Florp:
  fuzzy → crispy → dormant
          ↘ vaporized

Borp:
  red → blue
  red → plaid
  plaid → blue
```

Derived predicate:

```text
glorpable(Florp) when:
  every entity it snargs is blue
  AND it has an incoming wibbles relation
  AND no transitive florps path reaches a dormant Florp
```

The v0 fixture should deliberately include arbitrary relation types, inverse relations,
logic-defined transitive inference, edge properties, multiple workflows, both
permitted cycles and rejected cycles for an `acyclic = true` relation, section
targets, generated IDs, derived state, dangling refs, and named queries. Its post-v0
extension adds dynamically generated commands.

v0 example:

```bash
docgraph query glorpable --arg florp=florp:1
```

Post-v0 ergonomic equivalent:

```bash
docgraph florp glorpable florp:1
```

No Florp-specific code may exist in the binary.

<a id="s-PH9DH662QH"></a>
## 14. Repository-Host Reference Conformance (post-v0)

Provider shorthand should be exercised in both the documentation-maintenance fixture and synthetic fixture.

<a id="s-877RYGYVW4"></a>
### GitHub-like cases

```text
#123
owner/repo#123
GH-123
owner/repo@a5c3785
```

Cover repo-local shorthand, cross-repository refs, qualified commits, ordinary URLs, and offline normalization.

<a id="s-K0ZPST2XDR"></a>
### GitLab-like cases

```text
#123
!47
group/project#123
group/project!47
```

Include self-hosted GitLab configuration.

<a id="s-XQB7X1ZP6T"></a>
### Ambiguity

Verify:

- naked hexadecimal prose is not blindly interpreted as a commit
- a local commit candidate may resolve through Git without network access
- canonical docgraph refs take precedence where necessary
- malformed shorthand yields a diagnostic rather than a guessed target

<a id="s-NPF2JACCZS"></a>
### Repository configuration

Cover:

- provider inferred from `origin`
- explicit override
- multiple remotes
- fork plus upstream
- mirror
- self-hosted provider
- no remote

<a id="s-WHY9H5Q3BM"></a>
### Graph behavior

Provider references normalize into ordinary graph targets:

```text
task:184
   references
github:issue:owner/repo:123
```

No GitHub/GitLab-specific entity type may be required in engine code.

Synthetic case:

```text
florp:1
   grommits
github:issue:owner/repo:123
```

This must work without Florp-specific or GitHub-workflow-specific Rust code.

<a id="s-9P22A3H49K"></a>
## 15. Cross-Cutting Mutation Tests

Target cases are listed here. Section 20.2 accounts for the implemented v0 subset,
the remaining initial-design gaps, and explicitly deferred structural scenarios.

- file rename/move/delete/duplicate ID
- heading rename, section move/split/merge/delete
- initial and incremental normalization of ATX and Setext headings without stable IDs
- container-preserving normalization of headings inside lists and block quotes
- reproducible golden patches using an injected deterministic ID generator
- section merge/delete requiring removal or retargeting of durable inbound references
- relation add/remove, unknown informational relation, invalid endpoints, edge properties, cycle
- relation mutation refreshes generated incoming and inverse entries on affected documents
- prose-link changes refresh deterministic generated backlinks
- generated frontmatter is ignored as input and preserves surrounding authored content
- frontmatter check rejects missing, malformed, stale, or unsupported generated tables
- required, optional, enumerated, and homogeneous-array property validation
- undeclared entity/relation property, optional-property typo, and duplicate explicit relation triple
- legal/illegal workflow transition
- transition followed by derived fact and query-result changes
- direct semantic edit producing illegal state
- workflow-config change
- independently valid branches whose merged graph is invalid
- textual merge conflict reported with source diagnostics
- conflicting state changes and move plus new reference
- stale index after checkout and rebuild after merge
- concurrent mutation rejected after an affected-file hash changes
- concurrent change to another canonical input reloads and revalidates the complete graph
- concurrent non-canonical worktree change does not delay a mutation
- interrupted multi-file mutation recovered from its per-worktree journal
- recovery refuses to overwrite an affected file matching neither journaled state
- stale index rejected rather than queried after files change but refresh fails
- separate derived index, lock, journal, and fingerprint for simultaneous worktrees
- CI validation of the complete merged worktree
- repository logic accepts every construct in the v0 allowlist and rejects constructs outside it
- repository logic cannot redefine public built-ins or access implementation-private predicates
- repository logic runs in a docgraph-owned helper process with a five-second kill deadline
- supported logic syntax and built-in predicates remain stable for a schema version across internal engine upgrades

<a id="s-G9XSW3SPS6"></a>
## 16. Retrieval Tests

Each v0 fixture should exercise exact retrieval, graph traversal, FTS, named queries,
and context assembly. Post-v0 vector extensions add semantic search coverage.

Agents should not reconstruct semantic context through filesystem search.

Tests must distinguish explicit semantic relations, deterministic inferred facts,
informational Markdown links, and search results in structured output. Informational
links and search matches must not change semantic validity, workflow state, or
authoritative derived facts. Repository logic must not receive informational links
through its authoritative `relation` predicate.

Named-query tests must cover ordered input/output argument binding, missing and
ill-typed inputs, predicate-arity mismatch, ill-typed predicate results, declared
output-column order, and the stable JSON result envelope.

<a id="s-SRJWV4WZ2T"></a>
## 17. Dynamic CLI Tests

Fixture configuration defines named commands and proves the binary generates them without domain-specific code.

Tests: hierarchy, arguments, help text, read-only queries, transitions, relation mutations, JSON output, invalid command config.

<a id="s-4NJ8MJP7SH"></a>
## 18. Agent-Guidance Tests

Verify:

- root skill remains small
- sibling guides exist for supported task classes
- generated `AGENTS.md` / `CLAUDE.md` block is current
- managed instruction block updates idempotently
- `instructions check` fails for missing, stale, or malformed blocks without writing
- `instructions sync --dry-run` shows the exact prospective patch
- sync preserves user-authored bytes outside the managed block
- duplicate, nested, reversed, and unpaired markers are refused
- concurrent edits to an instruction target are not overwritten
- repo-specific appendix reflects configured ontology
- skill examples remain valid against fixtures

<a id="s-5GX1TKTSCE"></a>
## 19. Validation and Explainability

Validation must fail on unresolved managed references, duplicate or missing section
IDs, malformed frontmatter, invalid relation endpoints, invalid workflow states,
cycles in relation types configured with `acyclic = true`, missing required
properties, stale indexes, and incompatible schema versions. Broken ordinary
internal links are warnings by default and errors when configured; external URL
availability is not part of offline validation.

Repositories may define named explanation queries for important derived predicates.
These queries return the supporting or blocking facts directly; the engine does not
promise automatic provenance for arbitrary repository logic.

Bad:

```text
task:184 actionable = false
```

Good:

```text
Task-184 is not actionable because:
  ADR-42 is proposed
  Task-172 is incomplete
```

<a id="s-640PX8YW69"></a>
## 20. Fixture Structure

```text
fixtures/
  adr/
  tasks/
  requirements/
  section-references/
  human-agent/
  incidents/
  compliance/
  research/
  releases/
  docs/
  historical-research/
  synthetic/
```

Each fixture should exercise:

```text
files
→ parse
→ resolve
→ normalize
→ index
→ infer
→ query
→ mutate
→ validate prospective state
→ write
→ reindex
```

Golden tests verify exact patches, structured output, graph facts, and diagnostics.

<a id="s-2BW6BW30JA"></a>
### 20.1 v0 Conformance Slice

v0 requires the ADR, historical-research, and synthetic fixtures. Together they must
exercise the complete parse, normalize, index, infer, query, mutate, validate, write,
reindex, and generated-frontmatter sync/check loop plus the agent-guidance tests. The
remaining fixtures and the tests for vectors, repository-host shorthand, and semantic
diff remain the post-v0 conformance roadmap.

<a id="s-N6Z4YKP9M0"></a>
### 20.2 Current Coverage and Remaining Gaps

The fixtures and cross-cutting tests now prove parsing, normalization, typed
configuration and properties, multiple workflows, stable sections, exact section
relations, inverse projections, allowed and rejected cycles, persistent FTS,
restricted recursive logic, derived readiness, typed named queries, safe mutation
and recovery, worktree isolation, generated frontmatter, and generated agent
guidance. Runtime-backed logic tests run in Linux and Windows CI from checksummed
packages; the Windows suite consumes the pinned companion release rather than
rebuilding Soufflé.

Covered follow-on cases include:

- a richer synthetic ontology with multiple workflows, inverse and cyclic relations, section endpoints, and derived readiness/transitive queries
- persistent-index refresh/staleness and read-before-recovery scenarios
- concurrent edit, interrupted multi-file recovery, unknown recovery state, and separate-worktree cases
- exact section-source mutation and section-preserving generated-projection cases
- runtime-backed logic execution in CI rather than only parser/type tests
- nested repository-defined queries and mutations, project-aware help, command configuration validation, and command introspection
- recoverable document creation, identity-preserving moves with relative-link rewriting, deletion, and inbound-reference refusal
- recoverable stable-section split, adjacent-sibling merge, subtree deletion, and durable-reference refusal
- text and JSON review of granular entity, workflow, property, section, and relation changes between Git states

Provider shorthand; vectors; directional traversal and expanded context;
cross-command parse caching; and
persistent inferred-fact materialization remain post-v0 scenarios. Semantic merge
machinery is deliberately omitted unless concrete failures show that Git,
whole-corpus validation, and semantic review are insufficient. The authoritative accounting is
[Design section 15.2](./design.md#s-DRW3RR84VS).

<a id="s-7BT3682S4Q"></a>
## 21. Primary Conformance Requirement

For any semantic mutation represented by the repository model:

**An agent must be able to determine the complete document-graph impact through the tool without grepping the repository.**

The guarantee covers authoritative managed semantics and deterministic facts derived
from them, not undeclared meaning in prose.

A new workflow or ontology should require repository configuration and tests, not domain-specific engine code.
