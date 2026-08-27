+++
id = "decision:souffle-runtime"
type = "decision"
state = "accepted"

[properties]
title = "Use Soufflé as the logic runtime"

[docgraph_generated]
schema_version = 1
+++
<a id="s-A0N5ZETTD3"></a>
# Use Soufflé as the logic runtime

<a id="s-R4SBEPC12C"></a>
## Context

Docgraph needs recursive, repository-defined inference without embedding repository-specific behavior in Rust. Earlier Rust-native candidates were unsuitable or insufficiently maintained.

<a id="s-J0DTEEFCW0"></a>
## Decision

Expose a restricted, engine-independent Datalog contract and execute it with a pinned Soufflé runtime. Docgraph owns declarations, SQLite transport, process isolation, time limits, and result validation.

<a id="s-S3YFHFSPZ5"></a>
## Consequences

Repositories do not depend on Soufflé syntax outside the supported subset. The runtime is an opaque packaged dependency, including on native Windows, and may be replaced without changing the repository-facing contract.
