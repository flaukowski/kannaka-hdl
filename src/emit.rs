//! Emitters — a grown, resolved plan leaves as:
//!
//! - `json`    — the plan itself (machine-readable, stable schema)
//! - `crystal` — a kannaka-crystal `.crystal` program that approximates
//!   the architecture with placed pulses (until kannaka-crystal grows a
//!   first-class PLACE op; see ADR-0001 "ports" caveat)
//! - `html`    — a standalone growth visualization (no dependencies)

use crate::grow::Plan;
use std::collections::HashMap;

pub fn emit_json(plan: &Plan) -> String {
    serde_json::to_string_pretty(plan).expect("plan serializes")
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
    // Field material: the most common among resolved leaves.
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for leaf in &plan.leaves {
        if let Some(r) = &leaf.resolved {
            *counts.entry(r.material.as_str()).or_default() += 1;
        }
    }
    let material = counts
        .into_iter()
        .max_by_key(|(_, n)| *n)
        .map(|(m, _)| m)
        .unwrap_or("ideal_resonator");

    let mut out = String::new();
    out.push_str(&format!(
        "# grown by kannaka-hdl from: {}\n# {} leaves, {} bridges — pulse-placement approximation\n",
        plan.grown_from,
        plan.leaves.len(),
        plan.bridges.len()
    ));
    out.push_str(&format!("MATERIAL {material}\nSEED 21\n"));

    for leaf in &plan.leaves {
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
                .unwrap_or("(unresolved)")
        ));
    }
    for bridge in &plan.bridges {
        let (mx, my) = ((bridge.x1 + bridge.x2) / 2.0, (bridge.y1 + bridge.y2) / 2.0);
        let freq = class_frequency(&bridge.class);
        out.push_str(&format!(
            "PULSE {mx:.4} {my:.4} 0.05 0.600 {freq:.3} 0.0   # bridge {}\n",
            bridge.class
        ));
    }
    out.push_str("RESONATE 400\nDREAM deep\nSTABILIZE\n");
    out
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
  (plan.warnings.length ? ` — ${plan.warnings.length} unresolved (grey)` : '');
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
    fn json_round_trips_counts() {
        let p = plan();
        let v: serde_json::Value = serde_json::from_str(&emit_json(&p)).unwrap();
        assert_eq!(v["leaves"].as_array().unwrap().len(), 4);
        assert_eq!(v["bridges"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn crystal_program_is_runnable_shape() {
        let text = emit_crystal(&plan());
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
    fn html_embeds_the_plan() {
        let html = emit_html(&plan());
        assert!(html.contains("\"leaves\""));
        assert!(
            !html.contains("/*PLAN*/null"),
            "placeholder must be replaced"
        );
    }
}
