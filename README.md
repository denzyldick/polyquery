# polyquery

An LSP server that detects embedded SQL in 15+ programming languages, validates syntax with tree-sitter-sequel, provides schema-aware autocomplete, and can execute queries against live PostgreSQL / SQLite / MySQL databases.

## Languages supported

TypeScript, TSX, JavaScript, JSX, Python, Ruby, Go, Rust, Java, Kotlin, C/C++, C#, Swift, Elixir, PHP, and more.

SQL is detected in tagged template literals, function calls (e.g. `db.query("...")`), comment-annotated strings, and raw string literals — configurable per language.

## Features

- **Red squiggles**: Invalid SQL gets diagnostic underlines with error messages
- **Schema-aware autocomplete**: Table and column names from your live database
- **Run queries**: Code lens / command to execute the SQL and see results
- **15+ languages**: Embedded SQL detection for virtually every major language
- **Auto-introspection**: Schema is pulled from the database automatically (no manual definitions)

## Editor setup

### VS Code
```bash
code editors/vscode
npm install
vsce package
code --install-extension polyquery-*.vsix
```

### Neovim (lazy.nvim)
```lua
{
  name = "polyquery",
  dir = "/path/to/polyquery/editors/neovim",
}
```

### Emacs
```el
(add-to-list 'load-path "/path/to/polyquery/editors/emacs")
(require 'polyquery)
```

## Build

```bash
cargo build --release
```

## Run

Polyquery is an LSP server — it communicates over stdin/stdout. Point your editor's LSP client at the `polyquery` binary.

Set a database URL for schema support and query execution:

```
POLYQUERY_DATABASE_URL=postgres://user:pass@localhost/mydb
```

You can also set it interactively from within your editor:
- VS Code: `Polyquery: Set Database URL` command
- Neovim: `:PolyquerySetDatabaseUrl`
- Emacs: `M-x polyquery-set-database-url`

## Configuration

Optionally create `.polyquery.toml` in your project root:

```toml
[profile.default]
database_url = "postgres://..."
```

## Architecture

```
Editor (LSP client)
    ↕ LSP (stdin/stdout)
polyquery (LSP server)
    ├── Language detection (tree-sitter, 15+ languages)
    ├── SQL validation (tree-sitter-sequel)
    ├── Schema introspection (sqlx → PostgreSQL / SQLite / MySQL)
    ├── Completion provider (keywords + schema-aware)
    └── Query execution (sqlx, via runQuery command)
```

## Integration tests

```bash
cargo test
```

Database-dependent tests are marked `#[ignore]` — run them against a live DB:

```bash
POLYQUERY_DATABASE_URL=postgres://... cargo test -- --ignored
```

## License

MIT
