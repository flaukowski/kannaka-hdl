```
██╗  ██╗ █████╗ ███╗   ██╗███╗   ██╗ █████╗ ██╗  ██╗ █████╗
██║ ██╔╝██╔══██╗████╗  ██║████╗  ██║██╔══██╗██║ ██╔╝██╔══██╗
█████╔╝ ███████║██╔██╗ ██║██╔██╗ ██║███████║█████╔╝ ███████║
██╔═██╗ ██╔══██║██║╚██╗██║██║╚██╗██║██╔══██║██╔═██╗ ██╔══██║
██║  ██╗██║  ██║██║ ╚████║██║ ╚████║██║  ██║██║  ██╗██║  ██║
╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝  ╚═══╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝
                      H  D  L
        A R C H I T E C T U R E S   A R E   G R O W N
```

**KannakaHDL** is a growth language for informational materials. You don't draw an architecture — you write rewrite rules, and the architecture *grows*: cells recursively split space until they bottom out in base cases, and **every base case is a registry query** against [kannaka-crystal](https://github.com/flaukowski/kannaka-crystal)'s catalog of discovered primitives. Nothing is hand-drawn; grown structures are composed from what the swarm actually found.

[![CI](https://github.com/flaukowski/kannaka-hdl/actions/workflows/ci.yml/badge.svg)](https://github.com/flaukowski/kannaka-hdl/actions/workflows/ci.yml) [![License](https://img.shields.io/badge/license-Space%20Child-blueviolet)]() [![Rust](https://img.shields.io/badge/rust-2021-orange)]()

This is the **H3 layer** of the Kannaka Crystal PRD ("collections of informational primitives become an alternative memory architecture"), split into its own repo so the language can evolve independently. Growth semantics are inspired by [MorphoHDL](https://github.com/paradigms-of-intelligence/morpho)'s recursive rewrite model — no code shared; circuits have wires, resonant media have couplings, and that difference is this language's whole reason to exist.

## The language

```text
# an 8-seed memory bank, bridged pairwise at every level
cell MemoryBank(n) {
    when n > 1  => split MemoryBank(n / 2), MemoryBank(n / 2) bridge "Harmonic Bridge"
    when always => base "Memory Seed" min_persistence 0.3
}

grow MemoryBank(8)
```

- `cell` — a parameterized definition carrying ordered `when … => …` rules; the first guard that holds fires.
- `split` — divide this cell's region into equal slices (axis alternates per depth, Morpho-style) and recurse; `bridge "<class>"` couples consecutive siblings.
- `base` — a **registry query**: class + optional `min_persistence` floor + optional `material`. Resolution picks the best discovered primitive; unresolvable queries are warnings ("the swarm hasn't grown one yet"), not errors.
- Guards are integer expressions over cell parameters (`n > 1`, `n + m * 2 >= (k - 1) / 2`, `always`).
- Growth is bounded (depth 32, 4096 leaves) — a missing base case is an error, not a hang.

## Usage

```bash
kannaka-hdl check examples/membank.khdl
kannaka-hdl grow examples/membank.khdl --emit json            # plan (stdout)
kannaka-hdl grow examples/membank.khdl --emit html -o bank.html    # growth animation
kannaka-hdl grow examples/membank.khdl --emit crystal -o bank.crystal
kannaka-crystal run bank.crystal                              # instantiate + STABILIZE
```

Resolution reads kannaka-crystal's registry (`--registry`, else `$KANNAKA_CRYSTAL_DATA_DIR/registry.json`, else `~/.kannaka-crystal/registry.json`). Installed beside the `kannaka` CLI it is discovered as a plugin: `kannaka hdl grow …`.

### Running grown architectures (isolated lab)

Run `.crystal` output against a **snapshot copy** of the registry, not the live data dir — a running archivist saves every few seconds and last-writer-wins will eat your run's STABILIZE registrations (learned live; kannaka-crystal ADR-0002):

```bash
mkdir lab && cp ~/.kannaka-crystal/registry.json lab/
kannaka-hdl grow examples/membank.khdl --emit crystal -o bank.crystal
KANNAKA_CRYSTAL_DATA_DIR=./lab kannaka-crystal run bank.crystal
KANNAKA_CRYSTAL_DATA_DIR=./lab kannaka-crystal primitives   # what the bank crystallized
```

First lab result worth knowing: **resolution also chooses the field material** (most common among resolved primitives), and the medium dominates outcomes — a resolved bank in dissipative metamaterial crystallized weaker structures than the same architecture's unresolved fallback in `ideal_resonator`. See `examples/membank-resolved.khdl`.

## Emitters

| target | what you get |
|---|---|
| `json` | the plan: leaves (region, query, resolved primitive), bridges, warnings |
| `crystal` | a runnable `.crystal` program — pulse-placement **approximation** of the architecture (honest caveat: real coupling ports are open research, see ADR-0001) |
| `html` | standalone growth animation, zero dependencies |

## Status

v0.1 — experimental, evolving separately from kannaka-crystal on purpose. The open research question that gates v1: **what is a port?** A circuit wire is discrete; a coupling between resonant structures is a continuous overlap. Until that has empirical footing (experiments live in kannaka-crystal), the `crystal` emitter stays an approximation and the syntax stays small.

## Development

```bash
cargo test                  # parser, grower, resolver, emitters
cargo clippy --all-targets  # warning-free, enforced in CI
```

Releases are tagged `v*` and ship musl-static Linux binaries, Windows, and macOS builds with SHA-256 sidecars.

## License

[Space Child License v1.0](LICENSE) — free for peaceful use.
