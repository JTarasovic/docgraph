+++

id = "reference:design"
type = "reference"

[properties]
role = "design"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "issue:search-index-includes-structured-frontmatter"
predicate = "affects"
target = "reference:design#s-FDMHXV5Q4Q"

[[docgraph_generated.incoming]]
source = "plan:address-post-v0-reference-work"
predicate = "implements"
target = "reference:design#s-DRW3RR84VS"

[[docgraph_generated.incoming]]
source = "plan:close-initial-design-gaps"
predicate = "implements"
target = "reference:design#s-DRW3RR84VS"

[[docgraph_generated.incoming]]
source = "plan:project-aware-commands"
predicate = "implements"
target = "reference:design#s-DAR1R6WHJE"

[[docgraph_generated.incoming]]
source = "task:audit-initial-design-closure"
predicate = "implements"
target = "reference:design#s-DRW3RR84VS"

[[docgraph_generated.incoming]]
source = "task:complete-safe-read-mutation-boundary"
predicate = "implements"
target = "reference:design#s-B7542FYPRY"

[[docgraph_generated.incoming]]
source = "task:generate-model-appendix"
predicate = "implements"
target = "reference:design#s-DD5NS2HR0R"

[[docgraph_generated.incoming]]
source = "task:implement-derived-index-lifecycle"
predicate = "implements"
target = "reference:design#s-7BBMBXC9RK"

[[docgraph_generated.incoming]]
source = "task:index-searchable-markdown-content"
predicate = "implements"
target = "reference:design#s-FDMHXV5Q4Q"

[[docgraph_generated.inverses]]
source = "reference:design#s-7BBMBXC9RK"
type = "implemented_by"
target = "task:implement-derived-index-lifecycle"

[[docgraph_generated.inverses]]
source = "reference:design#s-B7542FYPRY"
type = "implemented_by"
target = "task:complete-safe-read-mutation-boundary"

[[docgraph_generated.inverses]]
source = "reference:design#s-DAR1R6WHJE"
type = "implemented_by"
target = "plan:project-aware-commands"

[[docgraph_generated.inverses]]
source = "reference:design#s-DD5NS2HR0R"
type = "implemented_by"
target = "task:generate-model-appendix"

[[docgraph_generated.inverses]]
source = "reference:design#s-DRW3RR84VS"
type = "implemented_by"
target = "plan:address-post-v0-reference-work"

[[docgraph_generated.inverses]]
source = "reference:design#s-DRW3RR84VS"
type = "implemented_by"
target = "plan:close-initial-design-gaps"

[[docgraph_generated.inverses]]
source = "reference:design#s-DRW3RR84VS"
type = "implemented_by"
target = "task:audit-initial-design-closure"

[[docgraph_generated.inverses]]
source = "reference:design#s-FDMHXV5Q4Q"
type = "affected_by"
target = "issue:search-index-includes-structured-frontmatter"

[[docgraph_generated.inverses]]
source = "reference:design#s-FDMHXV5Q4Q"
type = "implemented_by"
target = "task:index-searchable-markdown-content"

[[docgraph_generated.backlinks]]
source = "reference:config-grammar#s-P73QA8YDQB"
target = "reference:design#s-DRW3RR84VS"

[[docgraph_generated.backlinks]]
source = "reference:scenarios#s-N6Z4YKP9M0"
target = "reference:design#s-DRW3RR84VS"

+++
<a id="s-FEFSK4BQTV"></a>
# Draft Design: Git-Native Document Graph and Workflow Engine

**Status:** Draft

**Scope convention:** This document describes the cohesive target architecture.
The v0 delivery scope in Section 15.1 is the release contract. Material identified
as post-v0 is directional: it should fit the architecture, but is not required for
v0 conformance.

<a id="s-1T5ZJGP27R"></a>
## 1. Purpose

Build a generic, repository-local engine for structured document state, relationships, retrieval, validation, and safe mutation by humans and software agents.

Markdown/frontmatter committed to Git are canonical. Search indexes, graph state, FTS, vectors, and inferred state are disposable derived data.

The system exists to prevent humans and agents from having to reconstruct project semantics with repository-wide grep/search and then inconsistently hand-edit related files.

<a id="s-YNPWHT3H8T"></a>
## 2. Core Principles

<a id="s-ZF437PWHX2"></a>
### 2.1 Repository content is canonical

The repository remains readable, reviewable, mergeable, and usable without the derived database.

The database may be deleted and rebuilt at any time.

<a id="s-09FZ0ZJJJK"></a>
### 2.2 Mechanism belongs to the tool; policy belongs to the repository

The executable understands generic primitives:

- entities
- entity types
- relations
- properties
- states
- transitions
- workflows
- inference logic
- documents
- sections
- references

It must not hard-code concepts such as ADR, task, requirement, control, or incident.

<a id="s-PDYXWR5YH8"></a>
### 2.3 Semantic mutations go through the tool

Humans and agents may edit prose directly.

Managed semantic changes should use tool operations:

- state transitions
- relationships
- stable IDs
- managed properties
- entity creation/deletion
- moves and renames
- structural changes affecting references

The tool calculates impact, validates prospective state, and applies the complete patch as a recoverable validated transaction.

<a id="s-M27J7G8FS4"></a>
### 2.4 Arbitrary relationships are first-class

Relationships normalize to:

```text
(source, predicate, target, properties)
```

Examples:

```text
task:184   blocked_by    adr:42
task:184   implements    architecture#s-83JRT4K2P6
adr:42     supersedes    adr:19
claim:17   supported_by  evidence:81
```

Relation types are data, not Rust enums.

<a id="s-MTQ9N1DBBA"></a>
### 2.5 Prefer derived state

Store lifecycle state only when necessary.

Derive concepts such as blocked, ready, actionable, covered, effective, and stale from graph relationships and repository logic.

<a id="s-X2073SHCMW"></a>
### 2.6 Semantic impact is queryable

Agents should not need to grep the repository to answer supported questions such as:

- What blocks this task?
- Why is this entity not actionable?
- What depends on this decision?
- What references this section?
- What changes if this transition occurs?
- Which requirements lack implementation or verification?

<a id="s-SE7TAGV5CQ"></a>
### 2.7 Semantic authority is explicit

Managed frontmatter, configured state, and explicit semantic relations are
authoritative. Facts derived deterministically from those inputs by repository logic
are authoritative derived state. Ordinary Markdown links are informational, while
full-text and vector matches are discovery results only.

Informational links and search results must not affect semantic validity, workflow
state, or authoritative derived facts. Repositories may promote broken internal-link
diagnostics from warnings to errors without granting those links semantic authority.
Future extraction may propose semantic relations for review but must not promote
them silently.

Complete impact means complete with respect to the authoritative managed graph and
configured logic. Meaning expressed only in prose remains outside that guarantee.

<a id="s-MN0E1TQVNG"></a>
## 3. Repository Model

A repository defines its ontology and policy:

```text
.docgraph/
  project.toml
  entities.toml
  relations.toml
  workflows.toml
  commands.toml
  logic.dl
```

The exact file layout may evolve.

`commands.toml` configures repository-defined commands over named queries and generic
mutations. It was delivered after the v0 core loop.

<a id="s-Z44JC2K1TN"></a>
### 3.1 Entity types

```toml
[entity.adr]
description = "An architectural decision record."

[entity.task]
description = "A unit of executable work."

[entity.task.property.priority]
type = "string"
required = true
values = ["low", "normal", "high"]
```

Entity and relation properties use a small repository-defined schema. v0 supports
TOML-native scalar types, homogeneous arrays, required or optional values, and
enumerated values. Adding a property definition must not require a derived-database
schema migration.

Entity property values live under a configurable, reserved `[properties]` table in
document frontmatter. Keys in that table must be declared for the entity type;
unrelated top-level frontmatter remains outside docgraph's managed schema.

<a id="s-BR3ASMVXZ2"></a>
### 3.2 Relationships

```toml
[relation.blocked_by]
source = ["task"]
target = ["task", "adr"]
inverse = "blocks"
acyclic = true

[relation.implements]
source = ["task", "component"]
target = ["requirement", "section"]
inverse = "implemented_by"

[relation.implements.property.scope]
type = "string"
```

Inverse relations should normally be derived.

`acyclic = true` opts a relation type into cycle validation; omitted means cycles
are allowed. Transitive closure and its downstream effects are derived by
repository logic rather than configured as relation behavior.

An explicit managed relation is unique by `(source, predicate, target)`; its
properties belong to that edge. Repeating the same triple is invalid.

<a id="s-2WZBRN0128"></a>
### 3.3 Workflows

```toml
[workflow.adr]
initial = "proposed"

[workflow.adr.states.proposed]
transitions = ["accepted", "rejected"]

[workflow.adr.states.accepted]
transitions = ["superseded"]
```

The engine exposes generic operations such as:

```text
transition(entity, target_state)
```

<a id="s-BJKN85P0MX"></a>
### 3.4 Logic

Simple structure and workflow constraints belong in config.

`logic.dl` contains repo-specific inference and the predicates used by named
queries. It does not define validation policy, transition guards, or canonical
mutation side effects.

More expressive inference should use the supported Datalog subset rather than a
bespoke logic language.

Repository logic is a restricted Souffle-compatible module, not an unrestricted
Souffle program. It may define period-terminated named rules using `:-`, recursion,
safe stratified negation, and comparisons. It may not declare relations or types,
control I/O, include files, use components, pragmas, or custom functors. Docgraph
supplies declarations, SQLite input, the entry/output relation, and a five-second
process deadline. Repository validation also passes the complete generated program
through that runtime rather than trusting only docgraph's allowlist parser.

The repository schema version covers the supported logic syntax and built-in
predicate signatures. That versioned subset is the public contract; the embedded
engine version, APIs, storage format, and unsupported engine features are
implementation details.

v0 logic is a positive allowlist: inline rules, calls to public built-ins or rules in
the same module, scalar literals and comparisons, conjunction, positive recursion,
and safe stratified negation. Public built-in names are reserved, and repository
logic cannot address implementation-private stored relations.

<a id="s-KQFW6BJNC1"></a>
## 4. Canonical Documents

Markdown remains ordinary Markdown.

Structured project state lives in frontmatter. TOML is the leading candidate because Rust supports reliable parsing and format-preserving edits.

```toml
+++

id = "adr:42"
type = "adr"
state = "proposed"

[[relations]]
type = "supersedes"
target = "adr:19"

+++
```

Repositories may map existing metadata conventions onto the normalized model, but the tool should provide a preferred convention.

`docgraph adopt <path> --id <entity> --type <type>` adds the managed identity,
preserves unrelated TOML frontmatter and prose, and assigns stable IDs to headings in
the same recoverable mutation.

`docgraph adopt --batch <manifest.toml>` performs those changes for every declared
document and validates the combined corpus once. `docgraph workflow initialize
<entity-type>` materializes a newly configured workflow's initial state across all
affected entities in the same atomic mutation.

`docgraph document create` creates a normalized managed document inside the configured
corpus. `document move` preserves entity and section identity, rewrites resolvable
path-relative Markdown links, and rejects managed references whose meaning would
change. `document delete` refuses to leave inbound managed or Markdown references.
All three expose the complete prospective file patch through `--dry-run`.

Managed facts are changed through docgraph operations. Each entity document also has
a reserved generated table containing direct incoming relations, configured
inverses, and informational backlinks. Generated frontmatter is deterministic,
disposable, and never an authoritative graph input.

Change validation compares the current corpus to a Git base and rejects managed
changes that cannot be expressed as supported operations. It validates outcomes,
not CLI provenance, so prose remains unrestricted and equivalent safe edits remain
valid.

<a id="s-0TZNSBS3HS"></a>
## 5. Stable Identity

Identity is independent of filenames, titles, and heading text.

A file may exist at:

```text
docs/design/retry.md
```

while its graph identity is:

```text
spec:retry
```

Moving the file must not change identity.

<a id="s-37AYRK3RZR"></a>
### 5.1 Sections

Every heading event emitted by `pulldown-cmark` in an indexed document has a stable
opaque ID written immediately before it. This includes headings inside lists and
block quotes; text resembling a heading inside code or raw HTML is not a heading.

```markdown
<a id="s-83JRT4K2P6"></a>
## Retry Semantics
```

Section IDs should be:

- tool-generated
- short
- opaque
- repo-local
- collision checked
- stable across heading renames

The anchor is written as explicit HTML so ordinary fragment links resolve in
web-rendered Markdown. It appears immediately before the heading it identifies.

Read-only indexing never edits canonical files. `docgraph normalize` adds IDs to
all headings that do not yet have them as a recoverable semantic mutation, and
validation reports missing IDs. Repositories run normalization when first adopting
docgraph and after adding headings manually. For nested headings, normalization
preserves the surrounding container markers and indentation.

Heading renames and section moves preserve the adjacent anchor. When a section is
split, the existing ID remains with its heading and each new heading receives a new
ID. Merging or deleting a section retires an ID; durable inbound references must be
removed or retargeted in the same safe mutation.

`docgraph section split` inserts a same-level heading at an exact source line and
assigns its new stable ID. `section merge` removes the heading and ID of an
immediately following sibling while preserving its body and descendants. `section
delete` removes the selected section subtree. Merge and delete refuse to retire IDs
still used by managed relations or surviving Markdown links. All three use the
ordinary prospective, journaled mutation protocol and support `--dry-run`.

A short Crockford Base32 random token is a likely implementation.

<a id="s-J8Z29XS8AS"></a>
## 6. Cross-Document and External References

The indexer processes semantic relationships, ordinary Markdown links, and recognized repository-host shorthand.

<a id="s-01SAQYYWW7"></a>
### 6.1 Semantic relationships

```toml
[[relations]]
type = "implements"
target = "../architecture.md#s-83JRT4K2P6"
```

<a id="s-Z4JNFVJWJ6"></a>
### 6.2 Ordinary Markdown links

```markdown
See [retry semantics](../architecture.md#s-83JRT4K2P6).
```

Semantic relationships preserve their declared predicate. Ordinary Markdown links
become informational `links_to` edges used for backlinks and retrieval, not as input
to authoritative repository logic.

If a Markdown link occurs within a section, that section is the source node.

<a id="s-9NCGWT4529"></a>
### 6.3 Repository-host shorthand

Agents and humans commonly use GitHub, GitLab, and similar shorthand even where the Markdown renderer itself would not autolink it.

The engine may recognize provider-specific forms such as:

```text
#123
owner/repo#123
!47
group/project!47
foo/bar@a5c3785
```

These pass through a provider-specific normalization layer before entering the graph.

Example:

```text
#123
  ↓
github:issue:github.com/owner/repo:123
```

or:

```text
!47
  ↓
gitlab:merge_request:gitlab.com/group/project:47
```

Provider adapters are syntax adapters only. GitHub, GitLab, or other provider concepts must not leak into the generic graph or workflow model.

External references may exist as graph nodes without fetching remote metadata.
Adapters share an offline normalization interface. A future, separate external-entity
source capability may enrich those nodes from a forge as disposable derived data;
remote issue or change content does not become canonical repository Markdown.

<a id="s-WCDD32CNPK"></a>
### 6.3.1 Derived external reference data

A future external-entity source may resolve a canonical external identity into a
provider-neutral record containing its kind, title, body, state, author, timestamps,
URL, and provider-defined attributes. Implementations may expose read, search, and
mutation capabilities independently; reference normalization must not depend on any
of them.

Fetched records belong in the per-worktree derived store with provider, identity,
fetch time, and freshness metadata. They may contribute to `get`, `context`, search,
and vector retrieval, but must be labeled as derived so callers can distinguish them
from repository-authored facts. Provider state must not satisfy repository workflows
or validation rules unless repository logic explicitly maps it.

Missing credentials, unavailable networks, unsupported capabilities, stale cache
entries, and deleted remote objects must degrade to the canonical external identity.
The repository graph remains valid and usable offline, and deleting the derived
store loses no authored information.

<a id="s-ZDNCXK183C"></a>
### 6.4 Deterministic resolution

Reference resolution must be deterministic:

1. current-document section
2. relative repository path
3. canonical entity or entity-section reference
4. recognized provider-specific shorthand
5. external URI
6. unresolved reference

Provider context may be inferred from Git remotes, but repositories must be able to configure it explicitly for mirrors, multiple remotes, and self-hosted services.

Ambiguous references must not be guessed.

For example, a naked hexadecimal token should only be treated as a commit reference when sufficiently qualified or when the local Git repository confirms that it resolves.

The indexer must never silently fuzzy-match broken references.

<a id="s-PR1MNF240G"></a>
## 7. Internal Model

Conceptually:

```rust
Entity {
    id,
    entity_type,
    source,
    properties,
}

Section {
    id,
    document_id,
    parent,
    heading,
    source_span,
    content_hash,
}

Relation {
    source,
    predicate,
    target,
    properties,
    origin,
    source_span,
}
```

Relation origin distinguishes explicit frontmatter relations, informational Markdown
links, and inferred relations. Structured query output preserves this origin so a
caller can distinguish authoritative graph facts from informational edges.

The repository-logic `relation` interface exposes only explicit managed relations
and deterministic configured derivatives such as inverses. Informational edges
remain available through generic retrieval rather than logic predicates.

Source spans should be preserved for diagnostics and precise mutation.

<a id="s-7BBMBXC9RK"></a>
## 8. Indexing

```text
repository files
    ↓
frontmatter parser
    ↓
Markdown parser
    ↓
reference resolver
    ↓
normalized graph
    ↓
validation + inference
    ↓
derived database
```

The index contains:

- entities
- sections
- relations
- workflow state
- inferred facts
- source locations
- FTS
- vector index (follow-on)
- index metadata

v0 persists normalized graph facts, exact source locations, metadata, and FTS in a
per-worktree SQLite index. Index-backed operations fingerprint canonical inputs; a missing,
stale, old-format, or corrupt disposable index is rebuilt, while search reuses a
fresh FTS index across commands. Follow-on vector data lives in the same disposable
index.

Benchmarks up to 2,100 documents did not justify cross-command parse caching or
persistent inferred-fact materialization. Graph-only reads avoid the SQLite index,
and generated-frontmatter facts are prepared once per graph. Named queries evaluate
configured logic against the current canonical graph. Vector refresh reuses embeddings
whose content hash and provider identity are unchanged and embeds only changed chunks.

The index records enough metadata to detect staleness after checkout, merge, parser changes, schema changes, or embedding changes.

<a id="s-R9K70THM8E"></a>
### 8.1 Worktrees, Checkouts, and Merges

Each Git worktree has its own derived database, mutation lock, recovery journal, and
repository fingerprint. A checkout or merge that changes canonical inputs makes the
previous fingerprint stale; the next operation refreshes the index or refuses to use
it.

Git remains responsible for text merging. Docgraph validates the complete resulting
worktree, including uncommitted changes. Two branches may each be valid while their
combination is not, so CI must run `docgraph validate` against the merged result.
Semantic conflicts that Git cannot see, such as a newly formed forbidden cycle, are
reported as ordinary validation diagnostics. v0 does not install a custom merge
driver or attempt semantic three-way merging.

<a id="s-FDMHXV5Q4Q"></a>
## 9. Search and Retrieval

The retrieval surface supports structured graph retrieval (`get`, `neighbors`,
`incoming`, `outgoing`, `traverse`, `path`, and `context`), full-text search, and
vector search. Repositories may expose named explanation queries for important
derived predicates.

The CLI exposes `get`, `neighbors`, `incoming`, `outgoing`, `traverse`, `path`, and
`context`. Structured output preserves edge direction and origin. Traversal is
directional and depth-bounded; context assembles detailed nodes and the relations
among them. Informational Markdown edges remain opt-in.

Embedding generation uses a provider-neutral subprocess protocol rather than bundling
a model into the binary. `semantic-search` labels vector and full-text fallback
results explicitly; search matches remain non-authoritative discovery results.

Full-text and vector chunks project searchable text from Markdown bodies. They retain
headings, prose, link labels, inline code, and fenced-code contents while excluding
managed frontmatter, stable-anchor markup, link destinations, and formatting syntax.
Structured metadata remains available through graph retrieval and queries. Content
hashes cover the projection so metadata-only changes reuse existing embeddings.

<a id="s-DAR1R6WHJE"></a>
## 10. Repo-Aware CLI

The engine exposes generic primitives through the v0 CLI. Repositories may layer
project-specific commands over those primitives.

Generic operations remain available:

```bash
docgraph transition adr:42 accepted
docgraph relate task:184 implements architecture#s-83JRT4K2P6
docgraph query task_blockers --arg task=task:184
```

A named query declares an ordered, typed list of input and output arguments that
maps directly to its repository-logic predicate. Docgraph validates the predicate
arity, binds inputs by name, type-checks results, and exposes declared output columns
through a stable structured JSON envelope.

Repositories may define higher-level commands:

```toml
[command."adr.accept"]
operation = "transition"
entity_type = "adr"
target_state = "accepted"

[command."task.blockers"]
entity_type = "task"
query = "task_blockers"
```

Producing:

```bash
docgraph adr accept adr:42
docgraph task blockers task:184
docgraph task ready
```

Project-aware `--help` exposes repository-defined commands and descriptions.
Dot-separated command names form nested command paths. Query commands expose named
query inputs as long options; an input with a configured default is optional. When
`entity_type` is set, the first query input is instead the positional source entity.

A top-level query command can answer project-wide questions without inventing an
entity namespace, for example `docgraph next [--plan <plan>]`.

Humans and agents should normally use named operations rather than write Datalog directly.

<a id="s-Y0PBVH52Q7"></a>
## 11. Agent Integration

Agent guidance is part of the product interface.

<a id="s-64KP745XR0"></a>
### 11.1 Progressive-disclosure skill package

```text
skills/docgraph/
  SKILL.md
  config-authorship.md
  commands.md
  querying.md
  mutations.md
  workflows.md
  relationships.md
  document-authoring.md
  troubleshooting.md
  repository-maintenance.md
```

`SKILL.md` contains only:

- what docgraph manages
- non-negotiable behavioral rules
- how to inspect the current repository model
- pointers to the relevant sibling guide

Task guides should explain recommended procedures, not merely list command syntax.

For mutations, the preferred flow is:

```text
inspect
→ dry-run
→ mutate
→ validate
```

<a id="s-Q30QTKRZQ6"></a>
### 11.2 Generated repository instructions

v0 generates and maintains small tested blocks in configured instruction targets,
defaulting to `AGENTS.md` and `CLAUDE.md`.

The instructions should tell agents:

- this repository uses docgraph
- managed semantic state must not be hand-edited
- semantic impact should not be inferred with grep
- where the docgraph skill lives
- how to inspect the repository model
- to validate after relevant edits

The owned region is delimited by exact versioned markers:

```markdown
<!-- docgraph:agent-instructions:v1:begin -->
...
<!-- docgraph:agent-instructions:end -->
```

`docgraph instructions sync` creates or updates only that region and supports
`--dry-run`; `docgraph instructions check` detects missing, stale, or malformed
blocks without writing. Content outside a valid marker pair is preserved byte-for-byte.
Malformed or ambiguous markers cause refusal rather than guessed repair. Manual edits
inside the managed block are replaced only by an explicit `sync`, which uses the same
concurrent-change protection as other safe mutations.

<a id="s-DD5NS2HR0R"></a>
### 11.3 Tool skill vs repository appendix

The portable skill explains how docgraph works.

A tiny repository appendix at the end of each generated instruction block describes
configured entity types, relations, workflows, named queries, and common generic
operations. `docgraph instructions sync` and `check` manage it with the rest of the
block; it is not a separate generated file.

Detailed ontology remains dynamically queryable through `docgraph describe`.

<a id="s-Y4QFB1ZND8"></a>
### 11.4 Tested documentation

Skill examples and generated instructions should be tested against fixture repositories wherever practical.

The skill/config/CLI contract should be versioned so stale checked-in guidance can be detected and refreshed.

<a id="s-B7542FYPRY"></a>
## 12. Safe Mutation

Semantic mutation behaves as a recoverable validated transaction:

```text
load state and record the canonical-input fingerprint and affected-file states/hashes
    ↓
validate request
    ↓
calculate complete patch
    ↓
validate prospective repository
    ↓
acquire per-worktree mutation lock
    ↓
verify affected-file hashes are unchanged
    ↓
compare the current canonical-input fingerprint
    ↓
if other canonical inputs changed, reload, reapply, and revalidate (bounded retry)
    ↓
record recovery journal
    ↓
replace affected files through temporary files
    ↓
refresh index
```

If prospective validation fails, canonical files remain unchanged.

The lock serializes docgraph mutations within a worktree. Hash verification prevents
the tool from overwriting affected files changed since inspection. The repository
fingerprint covers only canonical graph inputs, so changes elsewhere in the worktree
do not delay a mutation. If another canonical input changes, docgraph reloads the
complete graph, reapplies the patch, and validates the new candidate. It continues
when that candidate remains valid and otherwise aborts without writing. Retries are
bounded so continuous edits cannot starve the operation indefinitely.

The recovery journal records the original and intended state of every affected path,
including path absence for creates and deletes,
and the canonical-input fingerprint. Recovery classifies each affected file as
original, intended, or unknown. It may automatically roll forward only when no file
is unknown and the intended result validates against the current complete graph. If
a file matches neither journaled state, recovery stops without overwriting it and
reports the files requiring manual resolution. An interrupted mutation is handled
before another mutation or query proceeds. The journal and lock are per-worktree
derived state and are not committed.

The derived database is updated only after the canonical files are replaced. It
records a repository fingerprint, and a query must refresh or reject an index whose
fingerprint does not match the current canonical graph inputs. A failure while
refreshing the index does not roll back canonical files; the disposable index is
rebuilt on the next use.

Substantial mutations should support dry-run impact analysis.

<a id="s-ZEJSC5W295"></a>
## 13. Direct File Editing

The tool does not prohibit direct editing.

Instead:

- prose remains freely editable
- managed frontmatter changes use docgraph operations
- generated frontmatter is refreshed by `docgraph frontmatter sync`
- invalid repository state is rejected by validation
- referenced structural edits should eventually have safe tool operations

Direct managed-field edits are unsupported. Validation enforces resulting invariants,
not editor provenance.

<a id="s-DHW5KPNDJV"></a>
## 14. Development and Repository Engineering

Initial stack:

- Rust stable
- `rustfmt`
- `clippy`
- `cargo test`
- `cargo nextest`
- `cargo deny`
- `cargo audit`
- `cargo machete`
- `cargo llvm-cov` where useful

Likely crate boundaries:

```text
crates/
  docgraph-core/
  docgraph-markdown/
  docgraph-logic/
  docgraph-cli/

fixtures/
docs/
```

CI should at minimum enforce:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
tests
fixture/conformance tests
```

Golden tests should verify exact Markdown patches, structured command output, graph
facts, and diagnostics. Tests inject a deterministic ID generator so expected patches
remain stable; production uses collision-checked opaque random IDs.

Given the selected IDs, mutation and formatting must be deterministic and idempotent.

<a id="s-0JC6P5MZMR"></a>
## 15. Technology Direction

Initial implementation:

- Rust
- one installable package containing the Rust CLI and an opaque logic-runtime companion
- `clap`
- `pulldown-cmark`
- TOML / `toml_edit`
- `serde`
- Git-aware traversal plus `ignore`
- BLAKE3
- a restricted Datalog backend
- SQLite persistence
- SQLite FTS
- SQLite vector search
- external embedding-provider abstraction

The logic backend remains behind an internal adapter. Repository configuration and
logic must not depend on its storage schema or host-language API.

Avoid initially unless justified:

- async runtime
- daemon
- filesystem watcher
- web server
- plugin ABI
- WASM extensions
- separate search/vector databases
- ORM
- hard-coded domain workflows

<a id="s-WZFMWK8K4N"></a>
### 15.1 v0 Delivery Scope

v0 must prove the complete managed-semantic loop:

- repository configuration and typed ontology
- Markdown/frontmatter parsing and stable-ID normalization for every heading
- entity, section, relation, and source-span indexing
- deterministic local references and ordinary Markdown links
- repository validation
- graph retrieval, traversal, FTS, restricted repository logic, and named queries
- dry-run and recoverable transition and relation mutations
- deterministic generated frontmatter with sync/check commands
- structured introspection, including JSON output
- the portable docgraph skill and task guides
- idempotent generated `AGENTS.md` and `CLAUDE.md` managed blocks
- a generated repository appendix describing the configured model and common operations
- ADR, historical-research, and synthetic conformance fixtures

Agent guidance is part of the v0 product interface. An agent must be able to discover
that docgraph manages repository semantics, load the correct guidance, inspect the
repository model, and perform the validated mutation flow without reconstructing the
workflow from prose.

Vector retrieval, embedding providers, repository-host shorthand adapters, generated
nested CLI commands, automated section split/merge operations, and semantic diff
tooling were outside the v0 delivery boundary. Repository-defined commands, command
introspection, stable-section lifecycle operations, semantic change review, provider
adapters, and vector retrieval have since been delivered as follow-on work.

<a id="s-DRW3RR84VS"></a>
### 15.2 Initial Design Follow-On

This work is tracked by [Close the initial design gaps](../plans/close-initial-design-gaps.md).

The narrow v0 success criterion was proven first. The four broader implementation
gaps tracked by the follow-on plan are now closed:

- the per-worktree SQLite graph/FTS index replaces the marker and enforces freshness
- reads recover interrupted mutations before loading the graph
- stable-section relation mutation and projections retain exact endpoints
- conformance covers generic workflows, recursion, cycles, recovery, worktrees, and the packaged runtime in CI

Directional traversal, expanded context, repository-host shorthand adapters, and
vector retrieval are delivered. Representative benchmarks found no need for
cross-command parse caching or persistent inferred-fact materialization; simpler
repeated work in graph-only reads and generated projections was removed instead.

Semantic merge machinery is deliberately omitted. Git remains the merge mechanism and
docgraph validates the complete result; that decision should be revisited only with
concrete merge failures that validation and semantic review do not address.

<a id="s-HESSVR7FJT"></a>
## 16. Primary Acceptance Principle

For any semantic mutation represented by the repository model:

**An agent must be able to determine the complete document-graph impact through the tool without grepping the repository.**

This guarantee applies to authoritative managed semantics and deterministic facts
derived from them. It does not claim to recover undeclared meaning from prose.

The repository defines policy and vocabulary. The Rust executable provides generic mechanism.
