# Claude Code Configuration — kannaka-hdl

KannakaHDL: a growth language for informational materials — architectures
grown by rewrite rules whose base cases are registry queries against
kannaka-crystal's discovered primitives. See
`docs/adr/0001-language-design.md`, especially the "ports" caveat.

## Rules

- ALWAYS read a file before editing it
- NEVER commit secrets, credentials, or .env files
- CI enforces `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`
- The `registry.json` schema is the contract with kannaka-crystal — class
  names are serialized enum variants (`MemorySeed`) matched via
  normalization against display forms ("Memory Seed"); keep the
  normalization tolerant, never exact-match
- The `crystal` emitter is an APPROXIMATION by design until the ports
  question has empirical footing — do not "improve" it into implying
  real coupling semantics

## Quick Reference

```bash
cargo test
cargo run -- check examples/membank.khdl
cargo run -- grow examples/membank.khdl --emit html -o bank.html
```

- Releases: tag `v*` → multi-platform binaries + SHA-256 sidecars
- Sibling: `C:\Users\nickf\Source\kannaka-crystal` (registry producer)
