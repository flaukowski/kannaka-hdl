//! The grower — expands a program's `grow` statements into a spatial plan.
//!
//! Each cell instance owns a rectangular region of the unit square. A
//! `split` divides the region into equal slices along the current axis
//! (alternating per depth, Morpho-style), recursing into children; a
//! `base` claims the region as a leaf carrying a registry query. Bridges
//! connect consecutive sibling centers.

use crate::parser::{Action, Call, CellDef, Cmp, Expr, Guard, Program, Query};
use serde::Serialize;
use std::collections::HashMap;

const MAX_DEPTH: usize = 32;
const MAX_LEAVES: usize = 4096;

#[derive(Debug, thiserror::Error)]
pub enum GrowError {
    #[error("unknown cell: {0}")]
    UnknownCell(String),
    #[error("cell {cell} expects {expected} arg(s), got {got}")]
    Arity {
        cell: String,
        expected: usize,
        got: usize,
    },
    #[error("no rule matched {cell}({args:?})")]
    NoRule { cell: String, args: Vec<i64> },
    #[error("division by zero in cell {0}")]
    DivZero(String),
    #[error("growth exceeded limits (max depth {MAX_DEPTH}, max leaves {MAX_LEAVES}) — missing base case?")]
    Runaway,
}

/// The component domain the compiler can resolve today. Queries may
/// name other domains (`memory.glyph`, `swarm.agent`); until their
/// providers arrive they resolve to an honest warning.
pub const DOMAIN_CRYSTAL: &str = "crystal";

/// Version of the Abstract Holographic Plan schema (ADR-0002 §7).
/// "2": queries are typed (domain/type/floors/strategy) and bridges are
/// typed couplings carrying a full query instead of a bare class.
pub const PLAN_SCHEMA_VERSION: &str = "2";

/// How a plan treats unresolved components (ADR-0002 §10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum UnresolvedMode {
    /// Compilation fails when a required component cannot be resolved.
    Strict,
    /// The plan carries placeholders that must not execute until resolved.
    Stub,
    /// Backends may approximate unresolved components (proxy pulses).
    Speculative,
}

#[derive(Debug, Clone, Serialize)]
pub struct Leaf {
    pub cell: String,
    pub domain: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub depth: usize,
    pub query: Query,
    /// Filled by the resolver; None = no registry match.
    pub resolved: Option<crate::registry::Resolved>,
}

/// A typed coupling between sibling regions (ADR-0002 §5). The `bridge`
/// keyword is sugar for a resonance-bridge coupling; the query carries
/// the same typed attributes as a `base` case.
#[derive(Debug, Clone, Serialize)]
pub struct Bridge {
    pub domain: String,
    pub coupling_type: &'static str,
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub depth: usize,
    pub query: Query,
    pub resolved: Option<crate::registry::Resolved>,
}

/// The one coupling type the `bridge` keyword produces today; `couple`
/// syntax with more types (ADR-0002 §5) arrives with later phases.
pub const COUPLING_RESONANCE_BRIDGE: &str = "resonance_bridge";

/// The Abstract Holographic Plan — KannakaHDL's stable intermediate
/// representation (ADR-0002 §7). Versioned, hashed, and honest about
/// what resolved and what did not.
#[derive(Debug, Serialize)]
pub struct Plan {
    pub schema_version: &'static str,
    pub compiler_version: &'static str,
    /// Deterministic hash of the source program; filled by [`Plan::seal`].
    pub program_hash: String,
    /// Deterministic hash of this plan's content; filled by [`Plan::seal`].
    pub plan_hash: String,
    pub unresolved_mode: UnresolvedMode,
    pub grown_from: String,
    pub leaves: Vec<Leaf>,
    pub bridges: Vec<Bridge>,
    /// Which registries resolution consulted (ADR-0002 §7).
    pub registry_snapshots: Vec<crate::registry::RegistrySnapshot>,
    pub resolution_report: Option<crate::registry::ResolutionReport>,
    pub warnings: Vec<String>,
}

/// Deterministic FNV-1a 64-bit over bytes — stable across platforms and
/// runs, so identical sources and plans hash identically everywhere.
pub fn fnv1a64(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv1a64:{h:016x}")
}

impl Plan {
    /// Stamp the deterministic program and plan hashes (ADR-0002 Phase 1).
    /// Call after resolution, before emission — the plan hash covers
    /// resolution results, so re-resolving against a changed registry
    /// yields a different plan hash for the same program hash.
    pub fn seal(&mut self, source: &str) {
        self.program_hash = fnv1a64(source.as_bytes());
        self.plan_hash = String::new();
        let json = serde_json::to_string(self).expect("plan serializes");
        self.plan_hash = fnv1a64(json.as_bytes());
    }
}

fn eval(expr: &Expr, env: &HashMap<String, i64>, cell: &str) -> Result<i64, GrowError> {
    Ok(match expr {
        Expr::Int(v) => *v,
        Expr::Param(name) => *env
            .get(name)
            .ok_or_else(|| GrowError::UnknownCell(format!("{cell}: unbound parameter {name}")))?,
        Expr::Add(a, b) => eval(a, env, cell)? + eval(b, env, cell)?,
        Expr::Sub(a, b) => eval(a, env, cell)? - eval(b, env, cell)?,
        Expr::Mul(a, b) => eval(a, env, cell)? * eval(b, env, cell)?,
        Expr::Div(a, b) => {
            let d = eval(b, env, cell)?;
            if d == 0 {
                return Err(GrowError::DivZero(cell.to_string()));
            }
            eval(a, env, cell)? / d
        }
    })
}

fn guard_holds(guard: &Guard, env: &HashMap<String, i64>, cell: &str) -> Result<bool, GrowError> {
    Ok(match guard {
        Guard::Always => true,
        Guard::Compare(lhs, cmp, rhs) => {
            let a = eval(lhs, env, cell)?;
            let b = eval(rhs, env, cell)?;
            match cmp {
                Cmp::Lt => a < b,
                Cmp::Le => a <= b,
                Cmp::Gt => a > b,
                Cmp::Ge => a >= b,
                Cmp::Eq => a == b,
                Cmp::Ne => a != b,
            }
        }
    })
}

struct Grower<'a> {
    cells: HashMap<&'a str, &'a CellDef>,
    leaves: Vec<Leaf>,
    bridges: Vec<Bridge>,
}

#[derive(Clone, Copy)]
struct Region {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl Region {
    fn center(&self) -> (f64, f64) {
        (self.x + self.w / 2.0, self.y + self.h / 2.0)
    }
}

impl<'a> Grower<'a> {
    fn instantiate(
        &mut self,
        call: &Call,
        env_outer: &HashMap<String, i64>,
        region: Region,
        depth: usize,
    ) -> Result<(), GrowError> {
        if depth > MAX_DEPTH || self.leaves.len() > MAX_LEAVES {
            return Err(GrowError::Runaway);
        }
        let def = *self
            .cells
            .get(call.cell.as_str())
            .ok_or_else(|| GrowError::UnknownCell(call.cell.clone()))?;
        if def.params.len() != call.args.len() {
            return Err(GrowError::Arity {
                cell: def.name.clone(),
                expected: def.params.len(),
                got: call.args.len(),
            });
        }
        let mut env = HashMap::new();
        for (param, arg) in def.params.iter().zip(&call.args) {
            env.insert(param.clone(), eval(arg, env_outer, &def.name)?);
        }

        for rule in &def.rules {
            if !guard_holds(&rule.guard, &env, &def.name)? {
                continue;
            }
            match &rule.action {
                Action::Base(query) => {
                    self.leaves.push(Leaf {
                        cell: def.name.clone(),
                        domain: query.domain.clone(),
                        x: region.x,
                        y: region.y,
                        w: region.w,
                        h: region.h,
                        depth,
                        query: query.clone(),
                        resolved: None,
                    });
                }
                Action::Split { children, bridge } => {
                    // Equal slices along the alternating axis.
                    let k = children.len() as f64;
                    let horizontal = depth.is_multiple_of(2);
                    let mut child_regions = Vec::new();
                    for (i, child) in children.iter().enumerate() {
                        let f = i as f64;
                        let sub = if horizontal {
                            Region {
                                x: region.x + region.w * f / k,
                                y: region.y,
                                w: region.w / k,
                                h: region.h,
                            }
                        } else {
                            Region {
                                x: region.x,
                                y: region.y + region.h * f / k,
                                w: region.w,
                                h: region.h / k,
                            }
                        };
                        child_regions.push(sub);
                        self.instantiate(child, &env, sub, depth + 1)?;
                    }
                    if let Some(bridge_query) = bridge {
                        for pair in child_regions.windows(2) {
                            let (x1, y1) = pair[0].center();
                            let (x2, y2) = pair[1].center();
                            self.bridges.push(Bridge {
                                domain: bridge_query.domain.clone(),
                                coupling_type: COUPLING_RESONANCE_BRIDGE,
                                x1,
                                y1,
                                x2,
                                y2,
                                depth,
                                query: bridge_query.clone(),
                                resolved: None,
                            });
                        }
                    }
                }
            }
            return Ok(());
        }
        let args: Vec<i64> = def.params.iter().map(|p| env[p]).collect();
        Err(GrowError::NoRule {
            cell: def.name.clone(),
            args,
        })
    }
}

pub fn grow(program: &Program) -> Result<Plan, GrowError> {
    let cells: HashMap<&str, &CellDef> =
        program.cells.iter().map(|c| (c.name.as_str(), c)).collect();
    let mut grower = Grower {
        cells,
        leaves: Vec::new(),
        bridges: Vec::new(),
    };

    let mut names = Vec::new();
    for call in &program.grows {
        names.push(call.cell.clone());
        grower.instantiate(
            call,
            &HashMap::new(),
            Region {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0,
            },
            0,
        )?;
    }
    Ok(Plan {
        schema_version: PLAN_SCHEMA_VERSION,
        compiler_version: env!("CARGO_PKG_VERSION"),
        program_hash: String::new(),
        plan_hash: String::new(),
        unresolved_mode: UnresolvedMode::Speculative,
        grown_from: names.join(", "),
        leaves: grower.leaves,
        bridges: grower.bridges,
        registry_snapshots: Vec::new(),
        resolution_report: None,
        warnings: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    const BANK: &str = r#"
        cell MemoryBank(n) {
            when n > 1  => split MemoryBank(n / 2), MemoryBank(n / 2) bridge "Harmonic Bridge"
            when always => base "Memory Seed" min_persistence 0.4
        }
        grow MemoryBank(8)
    "#;

    #[test]
    fn binary_bank_grows_8_leaves_7_bridges() {
        let plan = grow(&parse(BANK).unwrap()).unwrap();
        assert_eq!(plan.leaves.len(), 8);
        // A full binary tree over 8 leaves has 7 internal splits, each
        // bridging its 2 children once.
        assert_eq!(plan.bridges.len(), 7);
        // All leaves inside the unit square, distinct positions.
        for l in &plan.leaves {
            assert!(l.x >= 0.0 && l.x + l.w <= 1.0 + 1e-9);
            assert!(l.y >= 0.0 && l.y + l.h <= 1.0 + 1e-9);
        }
        let mut centers: Vec<(i64, i64)> = plan
            .leaves
            .iter()
            .map(|l| {
                (
                    ((l.x + l.w / 2.0) * 1e6) as i64,
                    ((l.y + l.h / 2.0) * 1e6) as i64,
                )
            })
            .collect();
        centers.sort();
        centers.dedup();
        assert_eq!(centers.len(), 8, "leaf regions must not overlap");
    }

    #[test]
    fn plan_carries_schema_identity() {
        let plan = grow(&parse(BANK).unwrap()).unwrap();
        assert_eq!(plan.schema_version, "2");
        assert_eq!(plan.compiler_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(plan.unresolved_mode, UnresolvedMode::Speculative);
        assert!(plan.leaves.iter().all(|l| l.domain == DOMAIN_CRYSTAL));
        assert!(plan.bridges.iter().all(|b| b.domain == DOMAIN_CRYSTAL));
        assert!(plan.program_hash.is_empty(), "hashes are set by seal()");
    }

    #[test]
    fn seal_hashes_are_deterministic_and_source_sensitive() {
        let mut a = grow(&parse(BANK).unwrap()).unwrap();
        let mut b = grow(&parse(BANK).unwrap()).unwrap();
        a.seal(BANK);
        b.seal(BANK);
        assert_eq!(a.program_hash, b.program_hash);
        assert_eq!(a.plan_hash, b.plan_hash);
        assert!(a.program_hash.starts_with("fnv1a64:"));
        assert_ne!(a.program_hash, a.plan_hash);

        let other = BANK.replace("MemoryBank(8)", "MemoryBank(4)");
        let mut c = grow(&parse(&other).unwrap()).unwrap();
        c.seal(&other);
        assert_ne!(a.program_hash, c.program_hash);
        assert_ne!(a.plan_hash, c.plan_hash);
    }

    #[test]
    fn missing_base_case_is_a_runaway_error() {
        let src = r#"
            cell Loop(n) {
                when always => split Loop(n), Loop(n)
            }
            grow Loop(1)
        "#;
        let err = grow(&parse(src).unwrap()).unwrap_err();
        assert!(matches!(err, GrowError::Runaway));
    }

    #[test]
    fn guards_select_rules_and_arity_is_checked() {
        let src = r#"
            cell X(n) {
                when n >= 3 => base "Echo Ring"
                when always => base "Memory Seed"
            }
            grow X(3)
        "#;
        let plan = grow(&parse(src).unwrap()).unwrap();
        assert_eq!(plan.leaves[0].query.class, "Echo Ring");

        let bad = r#"
            cell X(n) { when always => base "Seed" }
            grow X(1, 2)
        "#;
        assert!(matches!(
            grow(&parse(bad).unwrap()).unwrap_err(),
            GrowError::Arity { .. }
        ));
    }

    #[test]
    fn heterogeneous_split_and_no_rule_error() {
        let src = r#"
            cell Pair(n) {
                when n > 0 => split Ring(), Seed()
            }
            cell Ring() { when always => base "Echo Ring" }
            cell Seed() { when always => base "Memory Seed" }
            grow Pair(1)
        "#;
        let plan = grow(&parse(src).unwrap()).unwrap();
        let classes: Vec<&str> = plan.leaves.iter().map(|l| l.query.class.as_str()).collect();
        assert_eq!(classes, vec!["Echo Ring", "Memory Seed"]);

        let dead = r#"
            cell X(n) { when n > 5 => base "Seed" }
            grow X(1)
        "#;
        assert!(matches!(
            grow(&parse(dead).unwrap()).unwrap_err(),
            GrowError::NoRule { .. }
        ));
    }
}
