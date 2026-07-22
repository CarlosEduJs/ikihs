# ikihs-cli

<p align="center">
  <img src="../../assets/logo.svg" width="80" alt="Ikihs logo">
</p>

Apply syntax highlight on the command line.

## Install

```bash
cargo install ikihs-cli
```

To build from source:

```bash
git clone https://github.com/carlosedujs/ikihs
cd ikihs
cargo build --release -p ikihs-cli
./target/release/ikihs --help
```

## Commands

### `ikihs highlight`

Read a source file or stdin.
Apply a VS Code theme.
Print the tokens as JSON.

```bash
# Use a file as input
ikihs highlight main.rs --theme dark-plus.json

# Use stdin as input (language defaults to rs)
cat main.rs | ikihs highlight

# Set the language explicitly
ikihs highlight app.js --lang javascript --theme ~/themes/my-theme.json
```

**Arguments**

| Argument       | Description                                   |
| -------------- | --------------------------------------------- |
| `source`       | Path to the source file (use `-` for stdin)   |
| `-l, --lang`   | Language token for grammar lookup (default: rs) |
| `-t, --theme`  | Path to the VS Code theme JSON (default: dark-plus.json) |

**Output**

The output is pretty-printed JSON with byte offsets.

```json
{
  "tokens": [
    [
      {
        "content": "fn",
        "offset": 0,
        "color": "#569CD6",
        "fontStyle": "",
        "scope": "keyword.control.fn rust",
        "category": "Keyword"
      }
    ]
  ],
  "fg": "#D4D4D4",
  "bg": "#1E1E1E",
  "language": "Rust"
}
```

Each token contains:

- `content` — the text
- `offset` — the byte position from the start of input
- `color` — the color as hex
- `fontStyle` — the font style
- `scope` — the raw TextMate scope
- `category` — the Ikihs classification
