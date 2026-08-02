# Changelog

All notable changes to kannaka-hdl are documented here.

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
