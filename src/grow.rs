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

#[derive(Debug, Clone, Serialize)]
pub struct Leaf {
    pub cell: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub depth: usize,
    pub query: Query,
    /// Filled by the resolver; None = no registry match.
    pub resolved: Option<crate::registry::Resolved>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Bridge {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub depth: usize,
    pub class: String,
    pub resolved: Option<crate::registry::Resolved>,
}

#[derive(Debug, Serialize)]
pub struct Plan {
    pub grown_from: String,
    pub leaves: Vec<Leaf>,
    pub bridges: Vec<Bridge>,
    pub warnings: Vec<String>,
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
                    if let Some(class) = bridge {
                        for pair in child_regions.windows(2) {
                            let (x1, y1) = pair[0].center();
                            let (x2, y2) = pair[1].center();
                            self.bridges.push(Bridge {
                                x1,
                                y1,
                                x2,
                                y2,
                                depth,
                                class: class.clone(),
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
        grown_from: names.join(", "),
        leaves: grower.leaves,
        bridges: grower.bridges,
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
