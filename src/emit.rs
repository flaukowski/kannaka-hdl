//! Emitters — a grown, resolved plan leaves as:
//!
//! - `json`    — the plan itself (machine-readable, stable schema)
//! - `crystal` — a kannaka-crystal `.crystal` program that approximates
//!   the architecture with placed pulses (until kannaka-crystal grows a
//!   first-class PLACE op; see ADR-0001 "ports" caveat)
//! - `html`    — a standalone growth visualization (no dependencies)
//!
//! Every emitter declares its lowering model (ADR-0002 §11) so a
//! consumer can tell a faithful serialization from an approximation.

use crate::grow::{Plan, UnresolvedMode, DOMAIN_MEMORY};
use std::collections::HashMap;

pub const LOWERING_JSON: &str = "plan-json-v1";
pub const LOWERING_CRYSTAL: &str = "crystal-pulse-placement-v1";
pub const LOWERING_HTML: &str = "html-growth-viz-v1";
pub const LOWERING_MEMORY: &str = "memory-plan-v1";

pub fn emit_json(plan: &Plan) -> String {
    let mut v = serde_json::to_value(plan).expect("plan serializes");
    v["lowering_model"] = serde_json::Value::String(LOWERING_JSON.into());
    serde_json::to_string_pretty(&v).expect("value serializes")
}

/// Deterministic per-class carrier frequency so different primitive
/// classes ring distinguishably in the approximated field.
fn class_frequency(class: &str) -> f64 {
    let mut h: u32 = 0;
    for b in class.bytes() {
        h = h.wrapping_mul(31).wrapping_add(b as u32);
    }
    0.5 + (h % 5) as f64 * 0.45
}

pub fn emit_crystal(plan: &Plan) -> String {
    // Field material: the most common among resolved leaves. Flattening
    // a heterogeneous plan to one material is a declared approximation
    // (ADR-0002 §11) — never a silent one.
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for leaf in &plan.leaves {
        if let Some(r) = &leaf.resolved {
            *counts.entry(r.material.as_str()).or_default() += 1;
        }
    }
    let distinct_materials = counts.len();
    let material = counts
        .into_iter()
        .max_by_key(|(_, n)| *n)
        .map(|(m, _)| m)
        .unwrap_or("ideal_resonator");

    // Unresolved components only get proxy pulses in speculative mode;
    // in stub mode they stay declared-but-inert (ADR-0002 §10).
    let speculate = plan.unresolved_mode == UnresolvedMode::Speculative;

    let mut out = String::new();
    out.push_str(&format!(
        "# grown by kannaka-hdl from: {}\n# {} leaves, {} bridges — pulse-placement approximation\n# lowering model: {LOWERING_CRYSTAL}\n",
        plan.grown_from,
        plan.leaves.len(),
        plan.bridges.len()
    ));
    if distinct_materials > 1 {
        out.push_str(&format!(
            "# WARNING: {distinct_materials} materials among resolved leaves; field flattened to majority \"{material}\" — abstract approximation, not a material-region map\n"
        ));
    }
    out.push_str(&format!("MATERIAL {material}\nSEED 21\n"));

    for leaf in &plan.leaves {
        if leaf.resolved.is_none() && !speculate {
            out.push_str(&format!(
                "# STUB unresolved {} \"{}\" — no pulse emitted (re-resolve, or lower with --unresolved speculative)\n",
                leaf.cell, leaf.query.class
            ));
            continue;
        }
        let (cx, cy) = (leaf.x + leaf.w / 2.0, leaf.y + leaf.h / 2.0);
        let radius = (leaf.w.min(leaf.h) * 0.3).clamp(0.03, 0.15);
        let amplitude = 0.8 + leaf.resolved.as_ref().map(|r| r.persistence).unwrap_or(0.2);
        let freq = class_frequency(&leaf.query.class);
        out.push_str(&format!(
            "PULSE {cx:.4} {cy:.4} {radius:.4} {amplitude:.3} {freq:.3} 0.0   # {} {}\n",
            leaf.query.class,
            leaf.resolved
                .as_ref()
                .map(|r| r.id.as_str())
                .unwrap_or("(unresolved proxy)")
        ));
    }
    for bridge in &plan.bridges {
        if bridge.resolved.is_none() && !speculate {
            out.push_str(&format!(
                "# STUB unresolved bridge \"{}\" — no pulse emitted\n",
                bridge.query.class
            ));
            continue;
        }
        let (mx, my) = ((bridge.x1 + bridge.x2) / 2.0, (bridge.y1 + bridge.y2) / 2.0);
        let freq = class_frequency(&bridge.query.class);
        out.push_str(&format!(
            "PULSE {mx:.4} {my:.4} 0.05 0.600 {freq:.3} 0.0   # bridge {}\n",
            bridge.query.class
        ));
    }
    out.push_str("RESONATE 400\nDREAM deep\nSTABILIZE\n");
    out
}

/// Lower the plan's memory-domain components into a Kannaka Memory
/// architecture plan (ADR-0002 §12): nodes, HRM-oriented relationships
/// derived from typed couplings, and an executable `kannaka` CLI
/// command list — the first executable Memory lowering path. Components
/// from other domains are recorded under `skipped`, never silently
/// dropped; unresolved nodes only lower to commands in speculative
/// mode (stub mode declares them and counts them under
/// `stubbed_nodes`).
pub fn emit_memory(plan: &Plan) -> String {
    use serde_json::{json, Value};

    let speculate = plan.unresolved_mode == UnresolvedMode::Speculative;
    let memory_leaves: Vec<&crate::grow::Leaf> = plan
        .leaves
        .iter()
        .filter(|l| l.domain == DOMAIN_MEMORY)
        .collect();
    let node_id = |ordinal: usize| format!("m{ordinal}");

    let nodes: Vec<Value> = memory_leaves
        .iter()
        .enumerate()
        .map(|(ordinal, leaf)| {
            json!({
                "id": node_id(ordinal),
                "cell": leaf.cell,
                "component_type": leaf.query.component_type.as_deref().unwrap_or("memory"),
                "class": leaf.query.class,
                "resolved": leaf.resolved,
                "region": { "x": leaf.x, "y": leaf.y, "w": leaf.w, "h": leaf.h },
            })
        })
        .collect();

    // A coupling endpoint belongs to the memory node whose region
    // contains it; couplings touching non-memory regions don't lower.
    let locate = |px: f64, py: f64| -> Option<usize> {
        memory_leaves
            .iter()
            .position(|l| px >= l.x && px <= l.x + l.w && py >= l.y && py <= l.y + l.h)
    };
    let relationships: Vec<Value> = plan
        .bridges
        .iter()
        .filter_map(|bridge| {
            let from = locate(bridge.x1, bridge.y1)?;
            let to = locate(bridge.x2, bridge.y2)?;
            Some(json!({
                "coupling_type": bridge.coupling_type,
                "relation": bridge.query.class,
                "domain": bridge.domain,
                "from": node_id(from),
                "to": node_id(to),
                "resolved": bridge.resolved,
            }))
        })
        .collect();

    // Executable lowering: one kannaka CLI command per lowerable node.
    let mut commands = Vec::new();
    let mut stubbed = 0usize;
    for (ordinal, leaf) in memory_leaves.iter().enumerate() {
        if leaf.resolved.is_none() && !speculate {
            stubbed += 1;
            continue;
        }
        let importance = leaf
            .resolved
            .as_ref()
            .map(|r| r.persistence)
            .unwrap_or_else(|| leaf.query.min_persistence.max(0.5));
        match leaf.query.component_type.as_deref().unwrap_or("memory") {
            "dream" => commands.push(format!(
                "kannaka dream --mode deep   # node {}",
                node_id(ordinal)
            )),
            ty => commands.push(format!(
                "kannaka remember \"{} {} (node {}, plan {})\" --importance {:.2}",
                leaf.query.class,
                ty,
                node_id(ordinal),
                plan.plan_hash,
                importance
            )),
        }
    }

    // Resolved crystal components with signatures cross into the HRM
    // through the versioned forward transform (ADR-0002 §13); everything
    // else non-memory is skipped, never silently dropped.
    let mut transformed = Vec::new();
    let mut skipped = Vec::new();
    for leaf in plan.leaves.iter().filter(|l| l.domain != DOMAIN_MEMORY) {
        let glyph = leaf
            .resolved
            .as_ref()
            .and_then(crate::transform::crystal_signature_to_glyph);
        match glyph {
            Some(glyph) => {
                commands.push(format!(
                    "kannaka remember \"{}\" --importance {:.2}",
                    glyph.text,
                    leaf.resolved.as_ref().map(|r| r.persistence).unwrap_or(0.5)
                ));
                transformed.push(json!({
                    "cell": leaf.cell,
                    "source": glyph.source_id,
                    "class": glyph.class,
                    "via_transform": glyph.transform,
                    "resonance_dims": glyph.resonance.len(),
                    "information_loss": glyph.information_loss,
                }));
            }
            None => skipped.push(json!({
                "cell": leaf.cell,
                "domain": leaf.domain,
                "class": leaf.query.class,
            })),
        }
    }

    let doc = json!({
        "lowering_model": LOWERING_MEMORY,
        "schema_version": plan.schema_version,
        "compiler_version": plan.compiler_version,
        "program_hash": plan.program_hash,
        "plan_hash": plan.plan_hash,
        "unresolved_mode": plan.unresolved_mode,
        "nodes": nodes,
        "relationships": relationships,
        "transformed": transformed,
        "commands": commands,
        "stubbed_nodes": stubbed,
        "skipped": skipped,
    });
    serde_json::to_string_pretty(&doc).expect("memory plan serializes")
}

pub fn emit_html(plan: &Plan) -> String {
    let plan_json = serde_json::to_string(plan).expect("plan serializes");
    // Keep the embedded JSON safe inside a <script> block.
    let plan_json = plan_json.replace("</", "<\\/");
    HTML_TEMPLATE.replace("/*PLAN*/null", &plan_json)
}

const HTML_TEMPLATE: &str = r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>KannakaHDL — grown architecture</title>
<style>
  body { background:#0a0a14; color:#c8c8e8; font-family:'Segoe UI',system-ui,sans-serif; padding:1rem; }
  h1 { font-size:1rem; letter-spacing:.3em; color:#b49aff; font-weight:400; }
  canvas { background:#0d0d1a; border:1px solid #2a2a4a; border-radius:8px; display:block; margin-top:.8rem; max-width:100%; }
  p { font-size:.75rem; color:#9f9fc8; margin-top:.5rem; }
  button { background:#1c1c34; color:#c8c8e8; border:1px solid #2a2a4a; border-radius:5px; padding:.35rem .7rem; font-size:.75rem; margin-top:.6rem; }
</style>
</head>
<body>
<h1>KANNAKA HDL — GROWN ARCHITECTURE</h1>
<canvas id="cv" width="900" height="900"></canvas>
<button onclick="t0=performance.now()">RE-GROW</button>
<p id="info"></p>
<script>
const plan = /*PLAN*/null;
const cv = document.getElementById('cv'), ctx = cv.getContext('2d');
const maxDepth = Math.max(1, ...plan.leaves.map(l => l.depth));
function hue(s){let h=0;for(const c of s)h=(h*31+c.charCodeAt(0))>>>0;return h%360;}
let t0 = performance.now();
function draw(){
  const level = Math.min(maxDepth + 1, (performance.now() - t0) / 700);
  ctx.clearRect(0,0,cv.width,cv.height);
  const S = cv.width;
  ctx.strokeStyle = 'rgba(143,111,255,0.35)'; ctx.lineWidth = 1;
  for (const b of plan.bridges) if (b.depth < level) {
    ctx.beginPath(); ctx.moveTo(b.x1*S, b.y1*S); ctx.lineTo(b.x2*S, b.y2*S); ctx.stroke();
  }
  for (const l of plan.leaves) if (l.depth <= level) {
    ctx.strokeStyle = 'rgba(80,80,140,0.5)';
    ctx.strokeRect(l.x*S, l.y*S, l.w*S, l.h*S);
    const cx = (l.x + l.w/2)*S, cy = (l.y + l.h/2)*S;
    const pers = l.resolved ? l.resolved.persistence : 0.15;
    ctx.beginPath(); ctx.arc(cx, cy, 4 + pers*10, 0, Math.PI*2);
    ctx.fillStyle = l.resolved ? `hsl(${hue(l.query.class)} 70% 60%)` : '#444';
    ctx.fill();
  }
  requestAnimationFrame(draw);
}
document.getElementById('info').textContent =
  `${plan.grown_from} — ${plan.leaves.length} leaves, ${plan.bridges.length} bridges` +
  (plan.warnings.length ? ` — ${plan.warnings.length} unresolved (grey)` : '') +
  ' — lowering model: html-growth-viz-v1';
draw();
</script>
</body>
</html>
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{grow::grow, parser::parse};

    fn plan() -> Plan {
        let src = r#"
            cell Bank(n) {
                when n > 1  => split Bank(n / 2), Bank(n / 2) bridge "Harmonic Bridge"
                when always => base "Memory Seed"
            }
            grow Bank(4)
        "#;
        grow(&parse(src).unwrap()).unwrap()
    }

    #[test]
    fn json_round_trips_counts_and_declares_lowering_model() {
        let p = plan();
        let v: serde_json::Value = serde_json::from_str(&emit_json(&p)).unwrap();
        assert_eq!(v["leaves"].as_array().unwrap().len(), 4);
        assert_eq!(v["bridges"].as_array().unwrap().len(), 3);
        assert_eq!(v["lowering_model"], LOWERING_JSON);
        assert_eq!(v["schema_version"], "2");
        assert_eq!(v["unresolved_mode"], "speculative");
    }

    #[test]
    fn crystal_program_is_runnable_shape() {
        let text = emit_crystal(&plan());
        assert!(text.contains(&format!("# lowering model: {LOWERING_CRYSTAL}")));
        assert!(text.contains("MATERIAL "));
        assert_eq!(text.matches("PULSE").count(), 7, "4 leaves + 3 bridges");
        assert!(text.contains("STABILIZE"));
        // Coordinates stay in the unit square.
        for line in text.lines().filter(|l| l.starts_with("PULSE")) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let x: f64 = parts[1].parse().unwrap();
            let y: f64 = parts[2].parse().unwrap();
            assert!((0.0..=1.0).contains(&x) && (0.0..=1.0).contains(&y));
        }
    }

    #[test]
    fn stub_mode_withholds_unresolved_proxy_pulses() {
        // The fixture plan is fully unresolved: speculative mode (the
        // default) approximates everything; stub mode must not.
        let mut p = plan();
        p.unresolved_mode = UnresolvedMode::Stub;
        let text = emit_crystal(&p);
        assert_eq!(text.matches("PULSE").count(), 0);
        assert_eq!(text.matches("# STUB unresolved").count(), 7);
        assert!(text.contains("STABILIZE"), "program shape is preserved");
    }

    #[test]
    fn heterogeneous_materials_flatten_loudly_not_silently() {
        use crate::registry::{Resolved, PROVIDER_CRYSTAL};
        let mut p = plan();
        for (i, leaf) in p.leaves.iter_mut().enumerate() {
            leaf.resolved = Some(Resolved {
                provider: PROVIDER_CRYSTAL,
                id: format!("CRY-{i:06}"),
                class: "MemorySeed".into(),
                persistence: 0.6,
                noise_tolerance: 0.8,
                material: if i == 0 { "silicon" } else { "metamaterial" }.into(),
                signature: Vec::new(),
            });
        }
        let text = emit_crystal(&p);
        assert!(text.contains("MATERIAL metamaterial"), "majority wins");
        assert!(
            text.contains("# WARNING: 2 materials"),
            "flattening must be declared"
        );
    }

    #[test]
    fn memory_backend_lowers_memory_domain_and_skips_crystal() {
        let src = r#"
            cell Glyphs(n) {
                when n > 1  => split Glyphs(n / 2), Glyphs(n / 2) bridge memory.association "Mirror Link"
                when always => base memory.glyph "Identity Glyph"
            }
            cell Seed() { when always => base "Memory Seed" }
            grow Glyphs(2)
            grow Seed()
        "#;
        let mut p = grow(&parse(src).unwrap()).unwrap();
        p.seal(src);
        let v: serde_json::Value = serde_json::from_str(&emit_memory(&p)).unwrap();
        assert_eq!(v["lowering_model"], LOWERING_MEMORY);
        assert_eq!(v["nodes"].as_array().unwrap().len(), 2);
        assert_eq!(v["nodes"][0]["component_type"], "glyph");
        assert_eq!(v["relationships"].as_array().unwrap().len(), 1);
        assert_eq!(v["relationships"][0]["from"], "m0");
        assert_eq!(v["relationships"][0]["to"], "m1");
        assert_eq!(
            v["skipped"].as_array().unwrap().len(),
            1,
            "crystal leaf skipped, not dropped"
        );
        let commands = v["commands"].as_array().unwrap();
        assert_eq!(
            commands.len(),
            2,
            "speculative default lowers unresolved nodes"
        );

        // A resolved crystal leaf with a signature crosses via the
        // versioned forward transform instead of being skipped.
        let mut hybrid = grow(&parse(src).unwrap()).unwrap();
        let mut sig = vec![0.0; 256];
        sig[7] = 1.0;
        for leaf in hybrid.leaves.iter_mut().filter(|l| l.domain == "crystal") {
            leaf.resolved = Some(crate::registry::Resolved {
                provider: crate::registry::PROVIDER_CRYSTAL,
                id: "CRY-000777".into(),
                class: "MemorySeed".into(),
                persistence: 0.9,
                noise_tolerance: 0.9,
                material: "metamaterial".into(),
                signature: sig.clone(),
            });
        }
        let v: serde_json::Value = serde_json::from_str(&emit_memory(&hybrid)).unwrap();
        assert_eq!(v["skipped"].as_array().unwrap().len(), 0);
        let transformed = v["transformed"].as_array().unwrap();
        assert_eq!(transformed.len(), 1);
        assert_eq!(
            transformed[0]["via_transform"],
            crate::transform::SIGNATURE_TO_GLYPH
        );
        assert!(!transformed[0]["information_loss"]
            .as_array()
            .unwrap()
            .is_empty());
        assert!(v["commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c.as_str().unwrap().contains("CRY-000777")));
        assert!(commands[0]
            .as_str()
            .unwrap()
            .starts_with("kannaka remember"));

        // Stub mode declares the nodes but withholds executable commands.
        let mut stub = grow(&parse(src).unwrap()).unwrap();
        stub.unresolved_mode = UnresolvedMode::Stub;
        let v: serde_json::Value = serde_json::from_str(&emit_memory(&stub)).unwrap();
        assert_eq!(v["nodes"].as_array().unwrap().len(), 2);
        assert_eq!(v["commands"].as_array().unwrap().len(), 0);
        assert_eq!(v["stubbed_nodes"], 2);
    }

    #[test]
    fn html_embeds_the_plan() {
        let html = emit_html(&plan());
        assert!(html.contains("\"leaves\""));
        assert!(
            !html.contains("/*PLAN*/null"),
            "placeholder must be replaced"
        );
    }
}
