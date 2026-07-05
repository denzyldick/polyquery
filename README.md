# polyquery

An LSP server that detects SQL queries embedded in TypeScript/TSX source files using tree-sitter.

## How it works

Polyquery parses `.ts` / `.tsx` files and runs tree-sitter queries to locate embedded SQL in two forms:

- **Tagged template literals** — calls like `` sql`SELECT * FROM users` ``
- **Comment-annotated strings** — a comment containing "sql" immediately followed by a string, e.g. `// sql` + `"SELECT * FROM users"`

Detected SQL is logged to stderr via `tracing` at `info` level.

## Build

```bash
cargo build
```

## Run

Polyquery is an LSP server that communicates over stdin/stdout. Configure your editor to use the `polyquery` binary as a language server for TypeScript/TSX files.

Log output goes to stderr. Set `RUST_LOG=polyquery=debug` for verbose logging.

## Project status

Early prototype — currently logs found SQL but does not send diagnostics or perform analysis.
