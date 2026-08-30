# Configuration authorship

For a new repository, preview `docgraph init --dry-run`, then run `docgraph init`.
Use `--name`, `--documents`, and repeated `--instruction-target` options only when
the defaults are unsuitable. Init adopts valid existing configuration and refuses
conflicting or ambiguous state rather than overwriting it.

Use `docgraph describe` to inspect the current model. Edit `.docgraph/*.toml` and the
restricted `.docgraph/logic.dl` module, then run `docgraph validate`. Configuration
defines ontology and inference; it does not add executable repository plugins.
