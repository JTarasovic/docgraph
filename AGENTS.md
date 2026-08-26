# Repository guidance

This repository builds docgraph. The product contract lives in `docs/reference/`; keep implementation and tests aligned with it. Use the pinned mise environment and run `mise run check` before handing off changes.

## Dogfooding boundary

The current Markdown is design input, not a docgraph-managed corpus. Until this repository commits a `.docgraph/project.toml` with an explicit docs root, do not add managed frontmatter or stable anchors, run hypothetical docgraph commands, or interpret ordinary links as graph policy.

Once dogfooding is enabled, use docgraph only for the configured corpus and use its CLI for managed frontmatter changes. Prose remains directly editable. Generated docgraph instructions may extend this file inside their marked block; they do not replace these implementation instructions.
