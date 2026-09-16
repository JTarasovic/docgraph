# Document authoring

Edit prose freely. Run `docgraph normalize` after adding headings, use `docgraph
property set|unset` for entity properties, and run `docgraph frontmatter sync` after
relevant link or relation edits. Never hand-edit generated tables.

Use `docgraph document create|move|delete` for managed document lifecycle changes.
Use `docgraph section split|merge|delete` when a stable section's identity or extent
changes; remove or retarget durable references before retiring a section ID.

Docgraph canonical frontmatter is TOML in `+++` fences. YAML frontmatter in opening
`---` fences is diagnosed explicitly and excluded from Markdown heading parsing; use
`docgraph frontmatter migrate [PATH]... [--dry-run]` before normalizing or adopting it.
