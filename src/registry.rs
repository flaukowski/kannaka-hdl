//! Resolver against a kannaka-crystal registry.
//!
//! KannakaHDL never invents structures: every `base` query resolves to a
//! primitive that a swarm actually discovered. Reads the sibling
//! project's `registry.json` (`--registry`, else
//! `$KANNAKA_CRYSTAL_DATA_DIR/registry.json`, else
//! `~/.kannaka-crystal/registry.json`).

use crate::grow::Plan;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
struct RawPrimitive {
    id: String,
    /// Serialized enum variant name, e.g. `StandingEcho` — normalized
    /// against query classes like "Standing Echo" / `standing_echo`.
    class: String,
    persistence: f64,
    noise_tolerance: f64,
    material_id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RawRegistry {
    primitives: Vec<RawPrimitive>,
}

/// What a resolved query carries into the plan.
#[derive(Debug, Clone, Serialize)]
pub struct Resolved {
    pub id: String,
    pub class: String,
    pub persistence: f64,
    pub noise_tolerance: f64,
    pub material: String,
}

pub struct Registry {
    primitives: Vec<RawPrimitive>,
    pub source: PathBuf,
}

fn normalize(class: &str) -> String {
    class
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

pub fn default_path() -> PathBuf {
    if let Ok(dir) = std::env::var("KANNAKA_CRYSTAL_DATA_DIR") {
        return PathBuf::from(dir).join("registry.json");
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".kannaka-crystal")
        .join("registry.json")
}

impl Registry {
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let raw: RawRegistry =
            serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(Registry {
            primitives: raw.primitives,
            source: path.to_path_buf(),
        })
    }

    pub fn len(&self) -> usize {
        self.primitives.len()
    }

    pub fn is_empty(&self) -> bool {
        self.primitives.is_empty()
    }

    /// Best match: highest persistence among primitives of the queried
    /// class meeting the floor (and material, when specified).
    pub fn resolve(
        &self,
        class: &str,
        min_persistence: f64,
        material: Option<&str>,
    ) -> Option<Resolved> {
        let want = normalize(class);
        self.primitives
            .iter()
            .filter(|p| normalize(&p.class) == want)
            .filter(|p| p.persistence >= min_persistence)
            .filter(|p| material.is_none_or(|m| p.material_id == m))
            .max_by(|a, b| a.persistence.total_cmp(&b.persistence))
            .map(|p| Resolved {
                id: p.id.clone(),
                class: p.class.clone(),
                persistence: p.persistence,
                noise_tolerance: p.noise_tolerance,
                material: p.material_id.clone(),
            })
    }
}

/// Resolve every leaf and bridge in a plan; unresolvable queries become
/// warnings, not errors — an architecture can name structure the swarm
/// has not discovered yet (that is a research TODO, not a crash).
pub fn resolve_plan(plan: &mut Plan, registry: &Registry) {
    for leaf in &mut plan.leaves {
        leaf.resolved = registry.resolve(
            &leaf.query.class,
            leaf.query.min_persistence,
            leaf.query.material.as_deref(),
        );
        if leaf.resolved.is_none() {
            plan.warnings.push(format!(
                "no primitive for {} (class \"{}\", min_persistence {}{}) — swarm has not grown one yet",
                leaf.cell,
                leaf.query.class,
                leaf.query.min_persistence,
                leaf.query
                    .material
                    .as_deref()
                    .map(|m| format!(", material {m}"))
                    .unwrap_or_default()
            ));
        }
    }
    for bridge in &mut plan.bridges {
        bridge.resolved = registry.resolve(&bridge.class, 0.0, None);
    }
    plan.warnings.dedup();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{grow::grow, parser::parse};

    fn fake_registry(dir: &Path) -> PathBuf {
        let path = dir.join("registry.json");
        std::fs::write(
            &path,
            r#"{"next_serial":3,"primitives":[
                {"id":"CRY-000001","uuid":"00000000-0000-0000-0000-000000000001","hash":"aa","class":"MemorySeed","persistence":0.7,"noise_tolerance":0.9,"stability_score":2.0,"energy_profile":[],"material_id":"metamaterial","centroid":[0.5,0.5],"area":10,"signature":[],"lineage":[],"discovered_at":"2026-08-01T00:00:00Z","provenance":"t"},
                {"id":"CRY-000002","uuid":"00000000-0000-0000-0000-000000000002","hash":"bb","class":"MemorySeed","persistence":0.5,"noise_tolerance":0.9,"stability_score":2.0,"energy_profile":[],"material_id":"silicon","centroid":[0.5,0.5],"area":10,"signature":[],"lineage":[],"discovered_at":"2026-08-01T00:00:00Z","provenance":"t"},
                {"id":"CRY-000003","uuid":"00000000-0000-0000-0000-000000000003","hash":"cc","class":"HarmonicBridge","persistence":0.6,"noise_tolerance":0.8,"stability_score":2.0,"energy_profile":[],"material_id":"metamaterial","centroid":[0.5,0.5],"area":10,"signature":[],"lineage":[],"discovered_at":"2026-08-01T00:00:00Z","provenance":"t"}
            ]}"#,
        )
        .unwrap();
        path
    }

    #[test]
    fn resolves_by_class_floor_and_material_picking_best() {
        let dir = std::env::temp_dir().join(format!("khdl-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let reg = Registry::load(&fake_registry(&dir)).unwrap();

        // "Memory Seed" (display form) matches serialized "MemorySeed";
        // best persistence wins.
        let r = reg.resolve("Memory Seed", 0.0, None).unwrap();
        assert_eq!(r.id, "CRY-000001");
        // Material narrows it.
        let r = reg.resolve("memory_seed", 0.0, Some("silicon")).unwrap();
        assert_eq!(r.id, "CRY-000002");
        // Floor excludes.
        assert!(reg.resolve("Memory Seed", 0.8, None).is_none());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn plan_resolution_fills_leaves_and_warns_on_misses() {
        let dir = std::env::temp_dir().join(format!("khdl2-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let reg = Registry::load(&fake_registry(&dir)).unwrap();

        let src = r#"
            cell Bank(n) {
                when n > 1  => split Bank(n / 2), Bank(n / 2) bridge "Harmonic Bridge"
                when always => base "Memory Seed" min_persistence 0.4
            }
            cell Ghost() { when always => base "Phase Knot" }
            grow Bank(4)
            grow Ghost()
        "#;
        let mut plan = grow(&parse(src).unwrap()).unwrap();
        resolve_plan(&mut plan, &reg);

        let resolved: Vec<_> = plan
            .leaves
            .iter()
            .filter(|l| l.resolved.is_some())
            .collect();
        assert_eq!(resolved.len(), 4, "all Bank seeds resolve");
        assert!(plan.bridges.iter().all(|b| b.resolved.is_some()));
        assert_eq!(
            plan.warnings.len(),
            1,
            "Phase Knot unresolved -> one warning"
        );
        assert!(plan.warnings[0].contains("Phase Knot"));
        let _ = std::fs::remove_dir_all(dir);
    }
}
