# Changelog

All notable changes to kannaka-hdl are documented here.

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
