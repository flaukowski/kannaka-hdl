//! Cross-domain transforms (ADR-0002 §13) — versioned, deterministic,
//! and explicit about information loss. A transform is not assumed to
//! preserve meaning merely because two systems use the word resonance:
//! each one names what it drops, and the mapping is an analogy until
//! Crystal's pairwise experiments say otherwise.

use crate::registry::Resolved;
use serde::Serialize;

/// Forward transform: Crystal primitive signature → HRM glyph.
pub const SIGNATURE_TO_GLYPH: &str = "crystal-signature-to-hrm-glyph-v1";
/// Reverse transform: HRM glyph resonance → Crystal excitation.
pub const GLYPH_TO_ENCODING: &str = "hrm-glyph-to-crystal-encoding-v1";

/// How many dominant modes the glyph text and the reverse encoding
/// carry; the full resonance vector is preserved on the glyph itself.
const TOP_MODES: usize = 8;

/// A Kannaka Memory glyph representation of a Crystal primitive
/// (ADR-0002 §13, "Crystal Signature to Glyph").
#[derive(Debug, Clone, Serialize)]
pub struct GlyphRepr {
    pub transform: &'static str,
    pub source_id: String,
    pub class: String,
    /// L2-normalized signature — the glyph's resonance vector.
    pub resonance: Vec<f64>,
    /// Executable glyph text for `kannaka remember`.
    pub text: String,
    /// What this transform cannot carry into the HRM.
    pub information_loss: Vec<&'static str>,
}

/// A Crystal excitation derived from a glyph's resonance vector
/// (ADR-0002 §13, "HRM Memory to Crystal Encoding"). The signature's
/// 256 dims map onto a 16×16 grid; each dominant mode becomes a PULSE
/// at its grid cell, sign carried as phase. This is a speculative
/// pulse-placement encoding, same caveat as the crystal emitter.
#[derive(Debug, Clone, Serialize)]
pub struct CrystalEncoding {
    pub transform: &'static str,
    pub source_id: String,
    /// `PULSE x y radius amplitude frequency phase` lines.
    pub pulses: Vec<String>,
    /// What this transform cannot carry back into a Crystal field.
    pub information_loss: Vec<&'static str>,
}

/// The mode indices and values that dominate a resonance vector,
/// strongest magnitude first, zeros excluded.
fn dominant_modes(resonance: &[f64]) -> Vec<(usize, f64)> {
    let mut modes: Vec<(usize, f64)> = resonance
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, v)| *v != 0.0)
        .collect();
    modes.sort_by(|a, b| b.1.abs().total_cmp(&a.1.abs()));
    modes.truncate(TOP_MODES);
    modes
}

/// Convert a resolved Crystal component's signature into a glyph
/// representation. `None` when the component has no usable signature
/// (old registry rows, or a zero vector) — a transform with no input
/// is not a transform.
pub fn crystal_signature_to_glyph(resolved: &Resolved) -> Option<GlyphRepr> {
    let norm = resolved.signature.iter().map(|v| v * v).sum::<f64>().sqrt();
    if norm == 0.0 {
        return None;
    }
    let resonance: Vec<f64> = resolved.signature.iter().map(|v| v / norm).collect();
    let modes = dominant_modes(&resonance);
    let modes_text = modes
        .iter()
        .map(|(i, v)| format!("{i}:{v:.3}"))
        .collect::<Vec<_>>()
        .join(" ");
    Some(GlyphRepr {
        transform: SIGNATURE_TO_GLYPH,
        source_id: resolved.id.clone(),
        class: resolved.class.clone(),
        text: format!(
            "glyph {} from {} via {SIGNATURE_TO_GLYPH}; {} dims; dominant modes {modes_text}",
            resolved.class,
            resolved.id,
            resonance.len()
        ),
        resonance,
        information_loss: vec![
            "material",
            "persistence and noise_tolerance (numeric fidelity)",
            "spatial extent (centroid, area)",
            "lineage and provenance",
        ],
    })
}

/// Convert a glyph's resonance vector back into a Crystal excitation.
pub fn glyph_to_crystal_encoding(glyph: &GlyphRepr) -> CrystalEncoding {
    let side = (glyph.resonance.len() as f64).sqrt().ceil().max(1.0) as usize;
    let pulses = dominant_modes(&glyph.resonance)
        .into_iter()
        .map(|(index, value)| {
            let x = 0.1 + 0.8 * ((index % side) as f64 + 0.5) / side as f64;
            let y = 0.1 + 0.8 * ((index / side) as f64 + 0.5) / side as f64;
            let phase = if value < 0.0 {
                std::f64::consts::PI
            } else {
                0.0
            };
            format!(
                "PULSE {x:.4} {y:.4} 0.05 {:.3} 1.000 {phase:.4}   # mode {index}",
                value.abs()
            )
        })
        .collect();
    CrystalEncoding {
        transform: GLYPH_TO_ENCODING,
        source_id: glyph.source_id.clone(),
        pulses,
        information_loss: vec![
            "glyph text and memory content",
            "memory tier, salience, agent ownership, context",
            "modes beyond the dominant set",
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::PROVIDER_CRYSTAL;

    fn resolved_with(signature: Vec<f64>) -> Resolved {
        Resolved {
            provider: PROVIDER_CRYSTAL,
            id: "CRY-000042".into(),
            class: "MemorySeed".into(),
            persistence: 0.7,
            noise_tolerance: 0.8,
            material: "metamaterial".into(),
            signature,
        }
    }

    #[test]
    fn forward_transform_normalizes_and_records_loss() {
        let mut sig = vec![0.0; 256];
        sig[3] = 3.0;
        sig[100] = -4.0;
        let glyph = crystal_signature_to_glyph(&resolved_with(sig)).unwrap();

        assert_eq!(glyph.transform, SIGNATURE_TO_GLYPH);
        let norm: f64 = glyph.resonance.iter().map(|v| v * v).sum::<f64>().sqrt();
        assert!((norm - 1.0).abs() < 1e-9, "resonance is unit length");
        assert!((glyph.resonance[100] + 0.8).abs() < 1e-9);
        assert!(glyph.text.contains("CRY-000042"));
        assert!(glyph.text.contains("100:-0.800"), "dominant mode named");
        assert!(!glyph.information_loss.is_empty(), "loss must be declared");

        // No signature, no transform — never an invented glyph.
        assert!(crystal_signature_to_glyph(&resolved_with(vec![])).is_none());
        assert!(crystal_signature_to_glyph(&resolved_with(vec![0.0; 256])).is_none());
    }

    #[test]
    fn reverse_transform_maps_modes_to_grid_pulses_with_phase() {
        let mut sig = vec![0.0; 256];
        sig[0] = 1.0; // grid cell (0,0)
        sig[255] = -1.0; // grid cell (15,15), negative -> pi phase
        let glyph = crystal_signature_to_glyph(&resolved_with(sig)).unwrap();
        let enc = glyph_to_crystal_encoding(&glyph);

        assert_eq!(enc.transform, GLYPH_TO_ENCODING);
        assert_eq!(enc.pulses.len(), 2);
        assert!(enc
            .pulses
            .iter()
            .any(|p| p.contains("0.1250 0.1250") && p.ends_with("# mode 0")));
        assert!(enc
            .pulses
            .iter()
            .any(|p| p.contains("3.1416") && p.contains("# mode 255")));
        for p in &enc.pulses {
            let x: f64 = p.split_whitespace().nth(1).unwrap().parse().unwrap();
            let y: f64 = p.split_whitespace().nth(2).unwrap().parse().unwrap();
            assert!((0.0..=1.0).contains(&x) && (0.0..=1.0).contains(&y));
        }
        assert!(!enc.information_loss.is_empty());

        // Determinism: same glyph, same encoding.
        let again = glyph_to_crystal_encoding(&glyph);
        assert_eq!(enc.pulses, again.pulses);
    }
}
