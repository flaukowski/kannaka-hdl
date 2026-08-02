//! KannakaHDL — a growth language for informational materials.
//!
//! Architectures are not drawn; they are *grown*: cells carry rewrite
//! rules that recursively split space until they bottom out in `base`
//! cases, and every base case is a **registry query** against
//! kannaka-crystal's catalog of discovered primitives. The language is
//! the H3 layer of the Kannaka Crystal PRD — composing discovered
//! structure into candidate memory architectures.
//!
//! Pipeline: [`parser::parse`] → [`grow::grow`] →
//! [`registry::resolve_plan`] → [`emit`] (json / crystal / html).

pub mod emit;
pub mod grow;
pub mod parser;
pub mod registry;
