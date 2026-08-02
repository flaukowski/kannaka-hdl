# Changelog

All notable changes to kannaka-hdl are documented here.

## v0.5.0 — 2026-08-02

ADR-0002 §14, first slice of the swarm feedback loop (acceptance #10).

### Added
- **Structured discovery requests**: every unsatisfied query whose
  capability is genuinely missing (no provider for the domain, or no
  matching component) generates a `capability_discovery` request in the
  plan — `request_type`, `domain`, `component_type`, `class`,
  `constraints` (persistence/noise/material floors), and
  `requested_by_plan` stamped with the plan hash at seal time.
  Requests dedupe across identical queries; a `unique`-strategy
  exhaustion does not generate one (the capability exists — it is
  contended, not missing). The CLI reports the request count on
  stderr. Publishing over NATS and re-resolving on announcements is
  the remaining §14 work.

## v0.4.0 — 2026-08-02

ADR-0002 Phase 4: Kannaka Memory becomes a target.

### Added
- **`--emit memory`** (`memory-plan-v1`, ADR-0002 §12): lowers the
  plan's memory-domain components into a Kannaka Memory architecture
  plan — `nodes` (glyph/memory/belief/context/dream component types),
  `relationships` derived from typed couplings whose endpoints land in
  memory nodes, and an executable `commands` list for the `kannaka`
  CLI (`kannaka remember` / `kannaka dream`) — the first executable
  Memory lowering path. Components from other domains are recorded
  under `skipped`, never silently dropped; unresolved nodes lower to
  commands only in speculative mode (stub mode counts them under
  `stubbed_nodes`).
- **Multi-provider resolution**: `resolve_plan` takes a provider set
  and routes each query to the provider answering its domain, so one
  plan can span Crystal and Memory components (ADR-0002 acceptance #8).
  All consulted providers land in `registry_snapshots`.
- `FixtureProvider::with_domain` — stand in for domains whose live
  providers haven't arrived (used to resolve `memory.glyph` today).
  A live Memory provider backed by a running Kannaka instance is the
  natural next step once the kannaka-memory query contract is settled.

## v0.3.0 — 2026-08-02

ADR-0002 Phase 3: typed component queries and resolution strategies.

### Added
- **Typed queries** (ADR-0002 §3): optional `domain.type` prefix —
  `base memory.glyph "Identity Glyph"`, `base crystal.primitive "Memory
  Seed"`. Unnamed domains default to `crystal`. Domains without a
  provider resolve to an honest warning ("no provider for domain") —
  never a parse error, never a silent approximation.
- **`min_noise_tolerance` floor** alongside `min_persistence`.
  (Evidence-level floors wait for evidence fields in the registry —
  kannaka-crystal ADR-0004.)
- **Resolution strategies** (ADR-0002 §9): `strategy best` (default,
  highest persistence — the historical behavior), `robust` (highest
  noise tolerance), `unique` (each resolution claims a component no
  other unique query got; exhaustion warns), `diverse` (round-robin
  across candidates). `unique`/`diverse` are stateful per plan, and
  every decision lands in the emitted plan.
- **Bridges are typed couplings** (ADR-0002 §5): `bridge` takes the
  same attributes as `base` (floors, material, strategy); bridge
  entries carry a `coupling_type` (`resonance_bridge`) and a full
  `query`; unresolved couplings now warn like leaves do.

### Changed
- **Plan `schema_version` is now "2"**: bridge entries serialize
  `coupling_type` + `query` instead of a bare `class`, and leaf/bridge
  `domain` comes from the query.
- `Provider` gains `supports_type` and `candidates`; strategy selection
  happens in `resolve_plan` over the candidate set.

## v0.2.0 — 2026-08-02

ADR-0002 Phases 1–2: HDL now officially stands for **Holographic
Development Language**, the plan becomes the versioned Abstract
Holographic Plan — the domain-neutral intermediate representation the
rest of the ADR builds on — and resolution goes through a generic
provider interface.

### Added
- **Provider abstraction** (ADR-0002 §2): resolution runs against the
  `Provider` trait (id + domain + snapshot + query). The Crystal
  registry is the first implementation; `FixtureProvider` offers
  static in-memory components for tests and offline development with
  identical query semantics. Memory (HRM) and NATS swarm providers
  implement the same contract once typed queries land (Phase 3).
- **Plan identity**: `schema_version` ("1"), `compiler_version`, and
  deterministic `program_hash` / `plan_hash` (FNV-1a 64, stable across
  platforms; the plan hash covers resolution results, so the same
  program re-resolved against a changed registry hashes differently).
  `check` now prints the program hash.
- **Provider and domain fields**: leaves and bridges carry `domain`
  (`crystal` — the only domain until Phase 3 typed queries); resolved
  components carry `provider` (`crystal-registry`); plans record
  `registry_snapshots` (which registry, how many primitives) and a
  `resolution_report` (resolved/total counts).
- **Explicit unresolved mode** (`--unresolved strict|stub|speculative`,
  default `speculative` = the historical proxy-pulse behavior):
  `strict` fails compilation when components are unresolved or the
  registry is unavailable; `stub` keeps unresolved components
  declared-but-inert in `.crystal` output (comments, no pulses).
- **Versioned lowering models** (ADR-0002 §11): every emitter declares
  itself — `plan-json-v1`, `crystal-pulse-placement-v1`,
  `html-growth-viz-v1`. The `.crystal` emitter now warns loudly when it
  flattens heterogeneous materials to the majority material instead of
  doing so silently.
- ADR-0002 (`docs/adr/0002-holographic-development-language.md`).

### Changed
- JSON plan output is emitted with alphabetically ordered keys (it now
  passes through a `serde_json::Value` to attach the lowering model).

## v0.1.0 — 2026-08-02

First release of KannakaHDL — the H3 composition layer of the Kannaka
Crystal PRD, split into its own repo to evolve independently
(kannaka-crystal ADR-0003 Part 2 made concrete).

### Added
- **The language**: `cell` definitions with first-match `when` rules over
  integer parameter expressions; `split` (recursive spatial subdivision,
  alternating axis, optional `bridge "<class>"`) and `base` (a registry
  query: class + `min_persistence` + `material`). Bounded growth
  (depth ≤ 32, leaves ≤ 4096); line-numbered parse errors.
- **Registry resolution** against kannaka-crystal's `registry.json` —
  best-persistence match per query; unresolved queries are carried as
  plan warnings, never errors.
- **Emitters**: `json` (stable plan schema), `html` (standalone growth
  animation), `crystal` (runnable pulse-placement approximation ending
  in RESONATE / DREAM / STABILIZE).
- **CLI**: `kannaka-hdl check`, `kannaka-hdl grow --emit json|crystal|html`.
- Examples (`membank.khdl`, `ringfarm.khdl`), ADR-0001 (language design +
  the ports caveat), CI / release / cargo-audit workflows matching the
  kannaka-crystal conventions.
