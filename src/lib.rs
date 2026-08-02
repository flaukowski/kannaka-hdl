//! KannakaHDL — the **Holographic Development Language** (ADR-0002).
//!
//! Architectures are not drawn; they are *grown*: cells carry rewrite
//! rules that recursively split space until they bottom out in `base`
//! cases, and every base case is a **registry query** against a catalog
//! of discovered components — today kannaka-crystal's primitives, with
//! Kannaka Memory (HRM) and NATS swarm domains planned as providers.
//!
//! Compilation produces an **Abstract Holographic Plan**: a versioned,
//! deterministically hashed, domain-neutral intermediate representation
//! with an explicit unresolved-component policy (strict / stub /
//! speculative). Backends lower it honestly — every emitter declares its
//! lowering model and approximation status.
//!
//! Pipeline: [`parser::parse`] → [`grow::grow`] →
//! [`registry::resolve_plan`] → [`grow::Plan::seal`] → [`emit`]
//! (json / crystal / html).

pub mod emit;
pub mod grow;
pub mod parser;
pub mod registry;
