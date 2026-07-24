# Changelog entries

Each file in `.changes/` is a single changelog entry.
When you run `pnpm changelog version`, they are consumed and written into `CHANGELOG.md`.

## Format

Every entry is a markdown file with YAML frontmatter:

```md
---
bump: minor
packages: []
---

Changes go here...
```

## Frontmatter

| Field | Description |
|---|---|
| `bump` | `patch`, `minor`, or `major` |
| `packages` | List of package names to bump (or `[]` for all) |

### Available package names

| Name | Description |
|---|---|
| `ikihs-core` | Core types and traits |
| `ikihs-themes` | Theme parsers |
| `ikihs-engine-syntect` | Syntect engine |
| `ikihs-engine-treesitter` | Tree-sitter engine |
| `ikihs-engine-composite` | Composite engine |
| `ikihs-cli` | CLI binary |
| `ikihsjs` | npm package + Rust crate |
| `@ikihs/compare` | Fixture comparison tool |

## Body placeholders

The body supports these placeholders, replaced at version time:

| Placeholder | Replaced with |
|---|---|
| `{version}` | Workspace version (e.g. `0.3.0`) |
| `{version:pkg-name}` | Specific package version (e.g. `0.1.0` for `ikihs-core`) |

### Example

```md
---
bump: minor
packages: []
---

- **ikihs-core** (v{version:ikihs-core}): Description
- **ikihs-cli** (v{version:ikihs-cli}): Description
- **ikihsjs** (v{version:ikihsjs}): Description
```

Produces:

```md
- **ikihs-core** (v0.1.0): Description
- **ikihs-cli** (v0.3.0): Description
- **ikihsjs** (v0.3.0): Description
```

## Commands

| Command | What it does |
|---|---|
| `pnpm changelog add` | Interactive prompt to create an entry |
| `pnpm changelog version` | Consumes all entries, bumps versions, updates CHANGELOG.md |
| `pnpm changelog publish` | Builds + publishes ikihsjs to npm |
| `pnpm changelog update-release` | Syncs GitHub release body from CHANGELOG.md |

## Adding an entry manually

Create a file in `.changes/` named `<timestamp>.md`:

```bash
touch .changes/$(date +%s%N).md
```

Then open it and write the frontmatter + body.
