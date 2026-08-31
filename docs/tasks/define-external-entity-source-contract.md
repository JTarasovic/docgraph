+++

id = "task:define-external-entity-source-contract"
type = "task"
state = "done"

[properties]
title = "Define the external entity source contract"

[[relations]]
type = "part_of"
target = "plan:deliver-external-entity-sources"

[[relations]]
type = "implements"
target = "plan:deliver-external-entity-sources#s-6DE1F7THEQ"

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
predicate = "depends_on"
target = "task:define-external-entity-source-contract"

[[docgraph_generated.incoming]]
source = "task:persist-derived-external-entities"
predicate = "depends_on"
target = "task:define-external-entity-source-contract"

[[docgraph_generated.inverses]]
source = "task:define-external-entity-source-contract"
type = "required_by"
target = "task:add-github-external-entity-source"

[[docgraph_generated.inverses]]
source = "task:define-external-entity-source-contract"
type = "required_by"
target = "task:persist-derived-external-entities"

+++
<a id="s-5YYV0B0DSE"></a>
# Define the external entity source contract

Convert the directional design for derived external reference data into a precise
implementation contract before adding remote I/O.

Define a provider-neutral record containing canonical identity, provider, remote kind,
title, body, state, author, timestamps, URL, and provider-defined attributes. Separate
read, search, and mutation capabilities so a source can implement only what it
supports. Specify source registration, configuration and authentication boundaries,
freshness vocabulary, stable structured output, and failure classes.

Remote content is untrusted derived input. It must not become canonical Markdown,
agent instructions, validation evidence, or workflow state without an explicit
repository-logic mapping. Reference normalization must remain deterministic and must
not depend on a configured source, credentials, cache, or network.

Update the design, configuration grammar, and conformance scenarios with the settled
contract. Prove the generic boundary with a fake source so a second provider requires
no forge-specific changes in graph construction or retrieval.

<a id="s-TYN2SC2Q6N"></a>
## Acceptance

- Provider identities remain usable when no source is configured or available.
- A source advertises read, search, and mutation capabilities independently.
- Generic code consumes provider-neutral records and errors.
- Configuration never persists credentials in canonical or derived repository data.
- The contract defines the facts repository logic may explicitly map into workflows.

<a id="s-GCBE1WGXW0"></a>
## Result

Implemented the provider-neutral record, capability, error, freshness, trust, and
configuration contract. A fake source proves deterministic offline identity plus
live, cached, stale, and identity-only behavior without forge-specific graph code.
