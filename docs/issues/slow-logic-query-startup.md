+++

id = "issue:slow-logic-query-startup"
type = "issue"
state = "resolved"

[properties]
title = "Logic-backed commands have high startup latency"

[[relations]]
type = "affects"
target = "decision:souffle-runtime"

[[relations]]
type = "affects"
target = "reference:config-grammar#s-QVTHMJGF4H"

[docgraph_generated]
schema_version = 1

+++
<a id="s-X2M4X6KV7K"></a>
# Logic-backed commands have high startup latency

On this repository’s small corpus, a warm release build takes about 1.22 seconds for `docgraph next` and `docgraph validate`, while `get` and `describe` take about 45 milliseconds. Starting the packaged logic runtime alone takes about 27 milliseconds.

The logic path creates and populates a scratch SQLite database for every invocation. Fact insertion currently runs without an explicit transaction and is the first suspected bottleneck, especially on Windows. Measure the query phases, fix the dominant cost, and keep project-aware commands responsive enough for frequent human and agent use.

<a id="s-MTKNA3ZC88"></a>
## Resolution

Populate the complete scratch database in one SQLite transaction. Warm release `next` and `validate` runs fell to about 125 milliseconds, while non-logic reads remained around 44 milliseconds. The remaining difference is runtime and program-execution overhead rather than repeated durable commits.
