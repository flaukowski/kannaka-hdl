# Claude Code Configuration — kannaka-hdl

KannakaHDL: the Holographic Development Language (ADR-0002) — architectures
grown by rewrite rules whose base cases are registry queries against
discovered components (today: kannaka-crystal's primitives). Compilation
emits a versioned, hashed Abstract Holographic Plan with an explicit
strict/stub/speculative unresolved-component mode. See
`docs/adr/0001-language-design.md` (the "ports" caveat) and
`docs/adr/0002-holographic-development-language.md` (the roadmap).

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
