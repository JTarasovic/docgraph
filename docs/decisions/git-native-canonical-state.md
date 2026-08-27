+++
id = "decision:git-native-state"
type = "decision"
state = "accepted"

[properties]
title = "Keep canonical state in Git-native documents"

# docgraph:generated:v1:begin
[docgraph_generated]
# docgraph:generated:end
+++
<a id="s-AWWRH5TPT6"></a>
# Keep canonical state in Git-native documents

<a id="s-J0PPY12WNF"></a>
## Context

Docgraph needs structured semantics without replacing the documents people and agents already review in Git.

<a id="s-A19JAJT7J5"></a>
## Decision

Markdown and its managed TOML frontmatter are canonical. Indexes, search databases, inferred facts, and generated projections are disposable derived state.

<a id="s-X979NT9R08"></a>
## Consequences

Changes remain reviewable with ordinary Git tooling. Derived storage must be reproducible from the repository, and docgraph must not require a hosted database to interpret canonical state.
