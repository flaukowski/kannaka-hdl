# Changelog

All notable changes to kannaka-hdl are documented here.

## v0.9.0 — 2026-08-07

Evidence-ladder and behavioral-capability floors: the query grammar
catches up with crystal v0.10 (`evidence_level`, ADR-0004 §9) and v0.11
(`behavioral_capabilities`, §10), so demand programs can ask for
*validated* structure, not just class + raw metrics.

### Added
- **`min_evidence N`** query attribute (integer 0-8): candidates must
  sit at or above N on the evidence ladder. Absent registry field reads
  as 1 = Observed, matching crystal's own semantics.
- **`capability "name"`** query attribute: candidates must hold a
  **PASSED** record of the named behavioral contract —
  recorded-but-failed never satisfies, matching crystal's search.
- `Resolved` carries `evidence_level` + passed `capabilities` (serde
  defaults keep old plans and composites readable).
- Discovery requests carry the unmet `min_evidence` / `capability`
  constraints, so §14 demand names the validation bar, not just the
  class.
- `Provider::supports_evidence_floors()` (default **false**): a provider
  that cannot evaluate these floors refuses the query with an explicit
  warning instead of silently resolving against weaker evidence — the
  memory and composite providers refuse; the crystal registry and
  fixtures evaluate.
- `examples/capability-frontier.khdl`: a deliberately-unresolved demand
  for a noise-shielded, replicated Standing Echo (no passed
  `noise_shielding` record exists in the live registry yet).

### Unchanged
- Programs without the new attributes parse, resolve, and hash exactly
  as before (floors default to 0 / absent).

## v0.8.0 — 2026-08-02

ADR-0002 §15: composite architectures become components — the final
acceptance criterion (#11). All 12 are now satisfied.

### Added
- **`--register-composite <NAME>`**: a fully resolved, sealed plan
  registers into `~/.kannaka-hdl/composites.json` with the §15
  contract — program/plan hashes, component identities, coupling
  count, worst-case persistence/noise_tolerance, and the expectation
  verdicts as evidence. Plans with unresolved components are refused:
  only validated architectures become primitives.
- **`CompositeProvider`** (domain `composite`, auto-attached when the
  registry exists): `base composite.architecture "Name"` resolves a
  registered architecture, with floors applied against its worst-case
  metrics — Level 3 of the development ladder, validated architectures
  satisfying future base queries.

## v0.7.0 — 2026-08-02

ADR-0002 Phase 5: cross-domain transforms (acceptance #9), built on
the 256-dim primitive signature contract that landed with
kannaka-crystal ADR-0004.

### Added
- **`crystal-signature-to-hrm-glyph-v1`** (forward): a resolved
  Crystal component's signature becomes an HRM glyph — L2-normalized
  resonance vector, dominant-mode glyph text executable via
  `kannaka remember`, and a declared `information_loss` list
  (material, numeric fidelity, spatial extent, lineage). Components
  without a usable signature are never invented (`None`).
- **`hrm-glyph-to-crystal-encoding-v1`** (reverse): a glyph's
  resonance maps back onto the 16×16 signature grid as PULSE lines,
  sign carried as phase — a speculative pulse-placement encoding with
  its own declared loss (glyph text, tier/salience, non-dominant
  modes). Both directions are versioned, deterministic, and tested.
- **Memory backend integration**: resolved crystal leaves with
  signatures now cross into the memory plan under `transformed`
  (naming the transform and its loss) with `kannaka remember`
  commands, instead of being skipped.
- `Resolved` carries the primitive `signature` (elided from plan JSON
  when empty; old registry rows default to empty).

## v0.6.0 — 2026-08-02

ADR-0002 Phases 6 (publish) and 7 (assertions) first slices, plus a
live Kannaka Memory provider.

### Added
- **Live Memory provider** (`--memory-provider [BINARY]`): resolves
  memory-domain queries against a running Kannaka Memory through the
  `kannaka` CLI's bilateral resonance recall (`recall --envelope`,
  envelope schema 1.0). Contract mapping `memory-recall-v1`: memory
  `strength` → `persistence`, recall `similarity` → `noise_tolerance`,
  material `"hrm"`. Resonance recall always returns nearest memories,
  so candidates must resonate at ≥ 0.5 similarity (raised by the
  query's `min_noise_tolerance`) — resolution is never vacuous.
- **`expect` assertions** (ADR-0002 §16, acceptance #12):
  `expect <metric> <cmp> <number>` at top level. Compiler-verifiable
  now: `unresolved_components`, `capacity`, `couplings`, and
  worst-case `persistence` / `noise_tolerance` over resolved
  components. Runtime metrics (e.g. `recall_accuracy`,
  `swarm_agents`) report `unsupported` — never a silent pass; nothing
  resolved reports `inconclusive`. Verdicts land in the plan; any
  `fail` aborts emission with a nonzero exit.
- **`--publish-discovery [BINARY]`** (ADR-0002 §14): pushes the plan's
  `capability_discovery` requests onto the swarm work queue via
  `kannaka swarm enqueue capability_discovery <json>`, reporting the
  published count. Swarm workers can pick them up; re-resolving plans
  on capability announcements remains future §14 work.

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
