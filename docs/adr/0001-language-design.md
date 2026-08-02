# ADR-0001: KannakaHDL v0.1 Language Design

Status: Accepted · Date: 2026-08-02

## Context

kannaka-crystal's ADR-0003 proposed a rewrite-rule composition language
over discovered primitives as the PRD's H3 layer, deliberately split from
the experiment-scripting Crystal Language. This repo is that language,
separated so it can evolve on its own cadence. The growth model is
inspired by MorphoHDL (paradigms-of-intelligence/morpho) — recursive,
size-agnostic structural rewriting — with no code taken: that project is
single-digit-commits old, and its core abstractions (discrete wires, LUT
base cases) don't transfer to resonant media.

## Decision

**A deliberately tiny v0.1:**

- **Cells + first-match rules.** `cell Name(params) { when guard => action }`.
  Guards are integer expressions over parameters. Two actions only:
  `split` (recurse into children, equal slices along a per-depth
  alternating axis, optional `bridge "<class>"` coupling consecutive
  siblings) and `base` (a leaf).
- **Base cases are registry queries, not structures.** `base "Memory Seed"
  min_persistence 0.4 material "metamaterial"` resolves to the best
  matching primitive in kannaka-crystal's registry at grow time.
  Unresolvable queries are *warnings* — an architecture may name
  structure the swarm hasn't discovered yet; that's a research TODO the
  plan carries, not a crash.
- **Growth is total and bounded**: depth ≤ 32, leaves ≤ 4096, first-match
  rule order, integer division. A cell with no matching rule or no
  reachable base case is an error. Determinism end-to-end — same program
  + same registry = same plan.
- **Three emitters.** `json` (the plan, stable schema), `html`
  (standalone growth animation), `crystal` (a runnable kannaka-crystal
  program approximating the architecture with placed pulses — leaf
  center/size/persistence → pulse position/radius/amplitude, class-hashed
  carrier frequency, bridges as low-amplitude midpoint couplings,
  finished with `RESONATE`/`DREAM`/`STABILIZE`).
- **The ports caveat, stated up front.** A wire is discrete; a resonant
  coupling is a continuous overlap between structures. v0.1 does not
  pretend to solve this: bridges are placement hints, and the `crystal`
  emitter is labeled an approximation. When kannaka-crystal's experiments
  give coupling an empirical definition (likely a first-class `PLACE`
  op + measured cross-structure energy transfer), the emitter graduates
  and the language may grow port syntax. Not before.

## Consequences

- Two repos, one contract: the `registry.json` schema (and class-name
  normalization — serialized variant `MemorySeed` vs display "Memory
  Seed") is the coupling surface between kannaka-hdl and kannaka-crystal.
  Schema changes there must stay backward-readable here.
- First-match rule order is load-bearing and simple; there is no
  stochastic growth in v0.1. Randomized/parametric variation (à la
  parametric L-systems) is an easy later add via a seeded `rand()` expr.
- The unit-square slicing layout is crude but sufficient for plans and
  the pulse approximation; a real placement engine (packing, coupling-
  aware distances) belongs to the same milestone as ports.
