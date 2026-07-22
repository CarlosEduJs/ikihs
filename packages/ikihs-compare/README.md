# @ikihs/compare

<p align="center">
  <img src="../../assets/logo.svg" width="80" alt="Ikihs logo">
</p>

A comparison harness between Ikihs and Shiki.
It checks byte-level diffs on automated fixtures.
It generates an HTML report.

## Usage

```bash
npx tsx src/cli.ts <fixture-path> [theme]
```

Example:

```
$ npx tsx src/cli.ts rust/hello

  [rust/hello] score=98% exact=68 color=1 offset=0 extra=0 missing=0 total=69
  ------------------------------------
  OVERALL 98%  (68 / 69 tokens)
------------------------------------
  DIAGNOSE: rust/hello
  pos  char   shiki    ikihs   scope                          cat            match
  13   !      #D4D4D4  #DCDCAA support.macro                  Function       DIFF
------------------------------------
  Generated: compare-output/rust/hello-dark-plus.html
```

The HTML report shows the source side by side with color-coded diffs:

- **Yellow underline** — color mismatch
- **Blue underline** — token boundary difference
- **Red background** — missing token

## Output metrics

| Metric        | Meaning                                             |
| ------------- | --------------------------------------------------- |
| `exact`       | Byte positions where Ikihs and Shiki agree on color |
| `color_diff`  | Byte positions where colors differ                  |
| `offset_diff` | Byte position in one output but not the other       |
| `extra`       | Tokens in Ikihs that do not exist in Shiki          |
| `missing`     | Tokens in Shiki that do not exist in Ikihs          |
| `total`       | Byte positions with a color in either output        |
| `score`       | `exact / total * 100` (100% if no data)             |

## Fixtures

Fixtures are in `crates/ikihs-engine-syntect/fixtures/`.
Each fixture has a source file (`.rs`, `.js`, or `.py`) and a reference `.shiki.json` file from Shiki v4.

### Add a new fixture

1. Create a source file in the correct subdirectory under `fixtures/`.
2. Run `gen-fixtures` to regenerate all `.shiki.json` files:

```bash
npx tsx src/generate-fixtures.ts
```

3. Add a test case in `crates/ikihs-engine-syntect/tests/fixture_tests.rs`.
4. Run `cargo test -p ikihs-engine-syntect` to verify.

### Regenerate all references

```bash
npx tsx src/generate-fixtures.ts
```

This command highlights each source file with Shiki v4.
It writes the `.shiki.json` reference.
The committed `.shiki.json` files are the ground truth for all comparison runs.
