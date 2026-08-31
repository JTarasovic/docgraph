# Mutations

Preview with `--dry-run`, apply the same command, then validate. Mutations use
per-worktree locks, prospective validation, optimistic hashes, and recovery journals;
do not reproduce their frontmatter edits by hand.

Use `docgraph adopt --batch <manifest.toml> [--dry-run]` when adopting multiple
unnormalized documents together; the manifest contains one `[[document]]` table per
path, ID, type, and optional `property = ["name=value"]` list.

If an existing document begins with YAML frontmatter in `---` fences, run `docgraph
frontmatter migrate [PATH]... --dry-run`, inspect the exact patch, then repeat without
`--dry-run`. With no paths, migration converts every YAML-fronted document in the
configured corpus atomically. Migrate before normalization or adoption. Representable
YAML mappings, nested mappings, arrays, scalars, and date-like strings become TOML;
nulls, malformed YAML, non-string mapping keys, and other lossy shapes are refused.

Run `docgraph validate --changes <git-ref>` before committing. It permits prose and
supported semantic mutations but rejects unsupported managed metadata changes.
Use `docgraph review <git-ref>` alongside it when the semantic impact needs review.

After tightening a property schema exposes multiple existing errors, repair values
one at a time with `docgraph property set <entity> <property> <value> --repair`;
preview the same command with `--dry-run`. Repair mode is intentionally narrow: it is
accepted only when the prospective repository removes at least one validation error
and introduces or worsens none. Omit `--repair` for ordinary property changes and all
mutations of an already-valid repository.
