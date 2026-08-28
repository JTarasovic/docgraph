+++

id = "task:index-searchable-markdown-content"
type = "task"
state = "ready"

[properties]
title = "Index searchable Markdown content"

[[relations]]
type = "part_of"
target = "plan:address-post-v0-reference-work"

[[relations]]
type = "implements"
target = "plan:address-post-v0-reference-work#s-86JXA5Y7AV"

[[relations]]
type = "implements"
target = "reference:design#s-FDMHXV5Q4Q"

[[relations]]
type = "depends_on"
target = "task:add-vector-retrieval"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "issue:search-index-includes-structured-frontmatter"
predicate = "affects"
target = "task:index-searchable-markdown-content"

[[docgraph_generated.incoming]]
source = "task:optimize-repeated-graph-computation"
predicate = "depends_on"
target = "task:index-searchable-markdown-content"

[[docgraph_generated.inverses]]
source = "task:index-searchable-markdown-content"
type = "affected_by"
target = "issue:search-index-includes-structured-frontmatter"

[[docgraph_generated.inverses]]
source = "task:index-searchable-markdown-content"
type = "required_by"
target = "task:optimize-repeated-graph-computation"

+++
<a id="s-NQHC0JH2PA"></a>
# Index searchable Markdown content

Project document and section search content from Markdown bodies without indexing
managed frontmatter or stable-anchor markup. Preserve headings, prose, inline code,
and fenced-code content that users reasonably expect lexical or semantic search to
find.

Compute search and embedding reuse hashes from the projected content so generated
frontmatter-only changes do not invalidate embeddings. Keep entity metadata and
relations available through structured retrieval rather than flattening them into
text tokens. Cover full-text results, vector input, and unchanged-vector reuse with
tests.
