+++

id = "plan:deliver-external-entity-sources"
type = "plan"
state = "completed"

[properties]
title = "Deliver external entity sources"

[[relations]]
type = "implements"
target = "reference:design#s-WCDD32CNPK"

[[relations]]
type = "implements"
target = "reference:config-grammar#s-6RGFEKP4CT"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:add-github-external-entity-source"
predicate = "implements"
target = "plan:deliver-external-entity-sources#s-0WCAQFSYKB"

[[docgraph_generated.incoming]]
source = "task:add-github-external-entity-source"
predicate = "part_of"
target = "plan:deliver-external-entity-sources"

[[docgraph_generated.incoming]]
source = "task:define-external-entity-source-contract"
predicate = "implements"
target = "plan:deliver-external-entity-sources#s-6DE1F7THEQ"

[[docgraph_generated.incoming]]
source = "task:define-external-entity-source-contract"
predicate = "part_of"
target = "plan:deliver-external-entity-sources"

[[docgraph_generated.incoming]]
source = "task:dogfood-github-external-issues"
predicate = "implements"
target = "plan:deliver-external-entity-sources#s-2PX8WMMF3A"

[[docgraph_generated.incoming]]
source = "task:dogfood-github-external-issues"
predicate = "part_of"
target = "plan:deliver-external-entity-sources"

[[docgraph_generated.incoming]]
source = "task:integrate-external-entities-with-retrieval"
predicate = "implements"
target = "plan:deliver-external-entity-sources#s-HR8FQ9NYF4"

[[docgraph_generated.incoming]]
source = "task:integrate-external-entities-with-retrieval"
predicate = "part_of"
target = "plan:deliver-external-entity-sources"

[[docgraph_generated.incoming]]
source = "task:map-external-issues-into-project-work"
predicate = "implements"
target = "plan:deliver-external-entity-sources#s-0J00WB5MDA"

[[docgraph_generated.incoming]]
source = "task:map-external-issues-into-project-work"
predicate = "part_of"
target = "plan:deliver-external-entity-sources"

[[docgraph_generated.incoming]]
source = "task:persist-derived-external-entities"
predicate = "implements"
target = "plan:deliver-external-entity-sources#s-PX3AX3JRAW"

[[docgraph_generated.incoming]]
source = "task:persist-derived-external-entities"
predicate = "part_of"
target = "plan:deliver-external-entity-sources"

[[docgraph_generated.inverses]]
source = "plan:deliver-external-entity-sources"
type = "contains"
target = "task:add-github-external-entity-source"

[[docgraph_generated.inverses]]
source = "plan:deliver-external-entity-sources"
type = "contains"
target = "task:define-external-entity-source-contract"

[[docgraph_generated.inverses]]
source = "plan:deliver-external-entity-sources"
type = "contains"
target = "task:dogfood-github-external-issues"

[[docgraph_generated.inverses]]
source = "plan:deliver-external-entity-sources"
type = "contains"
target = "task:integrate-external-entities-with-retrieval"

[[docgraph_generated.inverses]]
source = "plan:deliver-external-entity-sources"
type = "contains"
target = "task:map-external-issues-into-project-work"

[[docgraph_generated.inverses]]
source = "plan:deliver-external-entity-sources"
type = "contains"
target = "task:persist-derived-external-entities"

[[docgraph_generated.inverses]]
source = "plan:deliver-external-entity-sources#s-0J00WB5MDA"
type = "implemented_by"
target = "task:map-external-issues-into-project-work"

[[docgraph_generated.inverses]]
source = "plan:deliver-external-entity-sources#s-0WCAQFSYKB"
type = "implemented_by"
target = "task:add-github-external-entity-source"

[[docgraph_generated.inverses]]
source = "plan:deliver-external-entity-sources#s-2PX8WMMF3A"
type = "implemented_by"
target = "task:dogfood-github-external-issues"

[[docgraph_generated.inverses]]
source = "plan:deliver-external-entity-sources#s-6DE1F7THEQ"
type = "implemented_by"
target = "task:define-external-entity-source-contract"

[[docgraph_generated.inverses]]
source = "plan:deliver-external-entity-sources#s-HR8FQ9NYF4"
type = "implemented_by"
target = "task:integrate-external-entities-with-retrieval"

[[docgraph_generated.inverses]]
source = "plan:deliver-external-entity-sources#s-PX3AX3JRAW"
type = "implemented_by"
target = "task:persist-derived-external-entities"

+++
<a id="s-VPJDD3WZHN"></a>
# Deliver external entity sources

<a id="s-FVT69E22RP"></a>
## Objective

Make configured remote entities usable as first-class graph participants without
copying their content into canonical repository Markdown. Prove the model with the
open issues in this repository through a read-only GitHub source.

<a id="s-ANCKR8ANEN"></a>
## Scope

This first slice adds the provider-neutral source boundary, disposable persistence,
GitHub issue reads, retrieval integration, and an explicit repository-logic bridge
from remote issue facts into project work. It preserves offline identity and cached
operation throughout.

Remote mutation, background synchronization, webhooks, GitLab network access, and a
general plugin ABI remain outside this plan. The interfaces must leave those
capabilities open without requiring them now.

<a id="s-AVA49V62F4"></a>
## Steps

<a id="s-6DE1F7THEQ"></a>
### Define the external entity source contract

Turn the directional external-data design into an executable provider-neutral record,
capability, configuration, trust, and failure contract.

<a id="s-PX3AX3JRAW"></a>
### Persist derived external entity records

Store fetched records and freshness metadata as disposable per-worktree state with
deterministic cached and identity-only fallbacks.

<a id="s-0WCAQFSYKB"></a>
### Add the GitHub external entity source

Read GitHub issues addressed by canonical external identities and normalize responses
into the provider-neutral model without introducing GitHub types into the graph core.

<a id="s-HR8FQ9NYF4"></a>
### Integrate external entities with retrieval

Expose enriched external nodes through `get`, `context`, full-text search, and vector
retrieval while clearly reporting provenance and freshness.

<a id="s-0J00WB5MDA"></a>
### Map external issues into project work

Expose provider-neutral external facts to repository logic and explicitly include
this repository's open GitHub issues in project-level work discovery.

<a id="s-2PX8WMMF3A"></a>
### Conform and dogfood GitHub external issues

Exercise online, cached, offline, stale, missing-credential, and deleted-record paths,
then use the real reported issues without creating local mirrors.

<a id="s-BGATNBFKSQ"></a>
## Completion

An agent can discover an open GitHub issue through `docgraph next`, inspect it through
ordinary retrieval commands, and continue using its stable external identity when the
network or credentials are unavailable. No remote content becomes canonical, no live
network is required by conformance tests, and the ten current reports remain hosted
only in GitHub.

<a id="s-73Q3RX06Q9"></a>
## Result

Delivered the provider-neutral source boundary, per-worktree external cache, read-only
GitHub issue source, exact/context/text/vector retrieval integration, and restricted
logic facts. This repository explicitly maps its open GitHub reports into `next_work`;
all ten reports remain canonical only in GitHub, while deterministic conformance tests
run without live network access.
