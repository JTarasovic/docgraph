+++

id = "plan:complete-v0"
type = "plan"
state = "completed"

[properties]
title = "Complete the v0 product contract"

[[relations]]
type = "implements"
target = "reference:config-grammar#s-D6B9JYV06F"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:audit-v0-success"
predicate = "implements"

[[docgraph_generated.incoming]]
source = "task:audit-v0-success"
predicate = "part_of"

[[docgraph_generated.incoming]]
source = "task:expand-dogfood-ontology"
predicate = "implements"

[[docgraph_generated.incoming]]
source = "task:expand-dogfood-ontology"
predicate = "part_of"

[[docgraph_generated.incoming]]
source = "task:generate-model-appendix"
predicate = "implements"

[[docgraph_generated.incoming]]
source = "task:generate-model-appendix"
predicate = "part_of"

[[docgraph_generated.incoming]]
source = "task:support-section-path-endpoints"
predicate = "implements"

[[docgraph_generated.incoming]]
source = "task:support-section-path-endpoints"
predicate = "part_of"

[[docgraph_generated.inverses]]
type = "contains"
target = "task:audit-v0-success"

[[docgraph_generated.inverses]]
type = "contains"
target = "task:expand-dogfood-ontology"

[[docgraph_generated.inverses]]
type = "contains"
target = "task:generate-model-appendix"

[[docgraph_generated.inverses]]
type = "contains"
target = "task:support-section-path-endpoints"

[[docgraph_generated.inverses]]
type = "implemented_by"
target = "task:audit-v0-success"

[[docgraph_generated.inverses]]
type = "implemented_by"
target = "task:expand-dogfood-ontology"

[[docgraph_generated.inverses]]
type = "implemented_by"
target = "task:generate-model-appendix"

[[docgraph_generated.inverses]]
type = "implemented_by"
target = "task:support-section-path-endpoints"

+++
<a id="s-9Q7F6PPPE0"></a>
# Complete the v0 product contract

<a id="s-B2N4PV81M7"></a>
## Objective

Finish and verify the v0 behavior defined by the reference corpus while using this repository as a representative docgraph project.

<a id="s-AQ57NFXKHB"></a>
## Scope

Track only concrete gaps against the v0 delivery scope. Post-v0 features remain outside this plan.

<a id="s-3SD16SBDQJ"></a>
## Steps

<a id="s-XPNKP8XTZW"></a>
### Expand the dogfood ontology

[Expand the dogfood ontology](../tasks/expand-dogfood-ontology.md) so the repository can represent its own decisions, plans, tasks, and delivery relationships.

<a id="s-87BFAA4CNY"></a>
### Support section endpoints in graph paths

[Support section endpoints in graph paths](../tasks/support-section-path-endpoints.md) so traversal works across the same entity and stable-section references accepted elsewhere by the CLI.

<a id="s-XPG39N6DGV"></a>
### Generate the repository-model appendix

[Generate the repository-model appendix](../tasks/generate-model-appendix.md) so agents can inspect the configured model and its common operations without reconstructing them from configuration.

The section-path and appendix steps may proceed independently.

<a id="s-RGKKJ07YJ3"></a>
### Audit the v0 success criterion

[Audit the v0 success criterion](../tasks/audit-v0-success.md) after both preceding implementation steps are complete, and record any remaining gaps against the end-to-end contract.

<a id="s-36G49JMN52"></a>
## Completion

The generated repository-model appendix exists, the success criterion has been audited end to end, and the repository passes its checks using its own configured graph.
