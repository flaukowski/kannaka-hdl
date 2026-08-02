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

**KannakaHDL — the Holographic Development Language** (ADR-0002). A backend-independent language for growing, resolving, composing, testing, and deploying holographic and resonant information architectures across the Kannaka ecosystem. You don't draw an architecture — you write rewrite rules, and the architecture *grows*: cells recursively split until they bottom out in base cases, and **every base case is a component query** — today against [kannaka-crystal](https://github.com/flaukowski/kannaka-crystal)'s catalog of discovered primitives, with Kannaka Memory HRM structures and NATS swarm capabilities as the next provider domains. Nothing is hand-drawn; grown structures are composed from what the swarm actually found.

*"HDL" here does not mean Hardware Description Language — wires, discrete ports, and defined logic levels are the wrong metaphors for resonant media. Couplings may be regions, fields, relationships, protocols, or transformations.*

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
- `split` — divide this cell's region into equal slices (axis alternates per depth, Morpho-style) and recurse; `bridge "<class>"` couples consecutive siblings as a typed **resonance-bridge coupling**, and takes the same query attributes as `base`.
- `base` — a **typed component query** (ADR-0002 §3): optional `domain.type` prefix (`base crystal.primitive "Memory Seed"`, `base memory.glyph "Identity Glyph"` — domain defaults to `crystal`), class, optional `min_persistence` / `min_noise_tolerance` floors, optional `material`, optional `strategy`. Unresolvable queries are warnings ("the swarm hasn't grown one yet"), not errors; domains without a provider warn honestly too.
- `strategy` — how resolution picks among candidates (ADR-0002 §9): `best` (default, highest persistence), `robust` (highest noise tolerance), `unique` (each resolution claims a distinct component; exhaustion warns), `diverse` (round-robin across candidates).
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

Unresolved components follow an explicit policy (`--unresolved`, ADR-0002 §10): **`speculative`** (default) lets backends approximate them — the historical proxy-pulse behavior; **`stub`** keeps them declared-but-inert (no pulses); **`strict`** fails the build — use it for scientific runs:

```bash
kannaka-hdl grow examples/membank.khdl --unresolved strict --emit crystal -o bank.crystal
```

Memory-domain queries (`base memory.glyph "…"`) resolve against a **live Kannaka Memory** with `--memory-provider` (bilateral resonance recall via the `kannaka` CLI; candidates must resonate at ≥ 0.5 similarity so resolution is never vacuous). Unsatisfied queries become `capability_discovery` requests in the plan (ADR-0002 §14) — `--publish-discovery` pushes them onto the swarm work queue via `kannaka swarm enqueue`.

Programs can declare **evidence requirements** (ADR-0002 §16) that gate emission:

```text
expect unresolved_components == 0
expect noise_tolerance >= 0.6      # worst case over resolved components
expect capacity >= 8
```

Compiler-verifiable metrics evaluate immediately (`pass`/`fail` — a fail aborts with nonzero exit); runtime metrics like `recall_accuracy` report `unsupported` until backend runners exist, never a silent pass.

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

| target | lowering model | what you get |
|---|---|---|
| `json` | `plan-json-v1` | the Abstract Holographic Plan: leaves (region, query, resolved component), typed couplings, snapshots, hashes, warnings |
| `crystal` | `crystal-pulse-placement-v1` | a runnable `.crystal` program — pulse-placement **approximation** of the architecture (honest caveat: real coupling ports are open research, see ADR-0001) |
| `memory` | `memory-plan-v1` | a Kannaka Memory architecture plan (ADR-0002 §12): nodes, HRM relationships from typed couplings, and an executable `kannaka` CLI command list |
| `html` | `html-growth-viz-v1` | standalone growth animation, zero dependencies |

## Status

v0.2 — experimental, evolving separately from kannaka-crystal on purpose. The trajectory is set by [ADR-0002 (Holographic Development Language)](docs/adr/0002-holographic-development-language.md); **Phases 1–2 (identity + IR, provider abstraction) have landed**: plans are versioned (`schema_version`, `compiler_version`), deterministically hashed (`program_hash` / `plan_hash`), carry provider/domain fields plus registry snapshots and a resolution report, every emitter declares its lowering model, and resolution runs through a generic `Provider` trait (Crystal registry + static fixture provider today). Still ahead: typed component queries over multiple providers (Crystal / Memory / Swarm / Hybrid), typed couplings replacing geometric bridges, Memory as a first-class backend, and the swarm feedback loop. Today's `crystal` emitter is classified **speculative** (`crystal-pulse-placement-v1`): the open research question that gates honest lowering is **what is a coupling?** — a circuit wire is discrete; a coupling between resonant structures is a continuous overlap. Until kannaka-crystal's pairwise coupling experiments give that empirical footing (see its ADR-0004 evidence model), the emitter stays an approximation and the syntax stays small.

## Development

```bash
cargo test                  # parser, grower, resolver, emitters
cargo clippy --all-targets  # warning-free, enforced in CI
```

Releases are tagged `v*` and ship musl-static Linux binaries, Windows, and macOS builds with SHA-256 sidecars.

## License

[Space Child License v1.0](LICENSE) — free for peaceful use.
