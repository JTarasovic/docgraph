+++

id = "issue:search-index-includes-structured-frontmatter"
type = "issue"
state = "resolved"

[properties]
title = "Search index includes structured frontmatter"

[[relations]]
type = "affects"
target = "reference:design#s-FDMHXV5Q4Q"

[[relations]]
type = "affects"
target = "task:index-searchable-markdown-content"

[docgraph_generated]
schema_version = 1

+++
<a id="s-YY9N99G0SW"></a>
# Search index includes structured frontmatter

Full-text and vector indexing currently consume complete managed files. This treats
entity IDs, workflow state, properties, relations, and generated projections as
searchable prose, creating noisy matches and unnecessary embedding refreshes.

Structured metadata already has deterministic graph and query surfaces. Retrieval
should instead index a deliberate Markdown-body projection.

<a id="s-6GMXSDP0F2"></a>
## Resolution

Search and embedding chunks now use parsed Markdown-body projections and projection
hashes. Managed metadata remains available through structured retrieval without
polluting text results or invalidating embeddings when only frontmatter changes.
