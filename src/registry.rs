//! Resolver against a kannaka-crystal registry.
//!
//! KannakaHDL never invents structures: every `base` query resolves to a
//! primitive that a swarm actually discovered. Reads the sibling
//! project's `registry.json` (`--registry`, else
//! `$KANNAKA_CRYSTAL_DATA_DIR/registry.json`, else
//! `~/.kannaka-crystal/registry.json`).

use crate::grow::Plan;
use crate::parser::Query;
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

/// The provider identifier the Crystal registry answers as (ADR-0002 §2).
pub const PROVIDER_CRYSTAL: &str = "crystal-registry";

/// A component provider (ADR-0002 §2): something that answers component
/// queries for one domain. The Crystal registry is the first
/// implementation; Memory (HRM) and NATS swarm providers take the same
/// contract when their domains arrive with typed queries (Phase 3) —
/// a Memory provider resolves glyph/memory/belief/context/dream
/// components against a Kannaka Memory instance, a Swarm provider
/// resolves agent roles and NATS subjects against live or declared
/// swarm capabilities. Providers may be backed by local files,
/// registries, services, NATS requests, or static fixtures.
pub trait Provider {
    /// Stable identifier recorded on every component this provider
    /// resolves (e.g. `crystal-registry`, `fixture`).
    fn id(&self) -> &'static str;
    /// The component domain this provider answers for.
    fn domain(&self) -> &'static str;
    /// A snapshot of this provider's current evidence source, recorded
    /// in the plan so resolution is reproducible.
    fn snapshot(&self) -> RegistrySnapshot;
    /// Answer a query; `None` means this provider has no component
    /// satisfying it (which is a research TODO, not an error).
    fn resolve_query(&self, query: &Query) -> Option<Resolved>;
}

/// What a resolved query carries into the plan.
#[derive(Debug, Clone, Serialize)]
pub struct Resolved {
    pub provider: &'static str,
    pub id: String,
    pub class: String,
    pub persistence: f64,
    pub noise_tolerance: f64,
    pub material: String,
}

/// A record of which registry resolution consulted (ADR-0002 §7) — the
/// plan names its evidence sources instead of leaving them implicit.
#[derive(Debug, Clone, Serialize)]
pub struct RegistrySnapshot {
    pub provider: &'static str,
    pub domain: &'static str,
    pub source: String,
    pub primitives: usize,
}

/// Resolution outcome counts, recorded in the plan (ADR-0002 §7).
#[derive(Debug, Clone, Serialize)]
pub struct ResolutionReport {
    pub leaves_total: usize,
    pub leaves_resolved: usize,
    pub bridges_total: usize,
    pub bridges_resolved: usize,
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
                provider: PROVIDER_CRYSTAL,
                id: p.id.clone(),
                class: p.class.clone(),
                persistence: p.persistence,
                noise_tolerance: p.noise_tolerance,
                material: p.material_id.clone(),
            })
    }
}

impl Provider for Registry {
    fn id(&self) -> &'static str {
        PROVIDER_CRYSTAL
    }

    fn domain(&self) -> &'static str {
        crate::grow::DOMAIN_CRYSTAL
    }

    fn snapshot(&self) -> RegistrySnapshot {
        RegistrySnapshot {
            provider: PROVIDER_CRYSTAL,
            domain: crate::grow::DOMAIN_CRYSTAL,
            source: self.source.display().to_string(),
            primitives: self.len(),
        }
    }

    fn resolve_query(&self, query: &Query) -> Option<Resolved> {
        self.resolve(
            &query.class,
            query.min_persistence,
            query.material.as_deref(),
        )
    }
}

/// A static, in-memory provider for tests and offline development
/// (ADR-0002 Phase 2) — no registry file, no sibling repo required.
/// Resolution semantics match the Crystal registry: tolerant class
/// normalization, persistence floor, optional material, best
/// persistence wins.
pub struct FixtureProvider {
    pub components: Vec<Resolved>,
}

/// The provider identifier fixtures answer as.
pub const PROVIDER_FIXTURE: &str = "fixture";

impl FixtureProvider {
    pub fn new(components: Vec<Resolved>) -> Self {
        FixtureProvider { components }
    }
}

impl Provider for FixtureProvider {
    fn id(&self) -> &'static str {
        PROVIDER_FIXTURE
    }

    fn domain(&self) -> &'static str {
        crate::grow::DOMAIN_CRYSTAL
    }

    fn snapshot(&self) -> RegistrySnapshot {
        RegistrySnapshot {
            provider: PROVIDER_FIXTURE,
            domain: self.domain(),
            source: "(static fixture)".into(),
            primitives: self.components.len(),
        }
    }

    fn resolve_query(&self, query: &Query) -> Option<Resolved> {
        let want = normalize(&query.class);
        self.components
            .iter()
            .filter(|c| normalize(&c.class) == want)
            .filter(|c| c.persistence >= query.min_persistence)
            .filter(|c| query.material.as_deref().is_none_or(|m| c.material == m))
            .max_by(|a, b| a.persistence.total_cmp(&b.persistence))
            .map(|c| Resolved {
                provider: PROVIDER_FIXTURE,
                ..c.clone()
            })
    }
}

/// Resolve every leaf and bridge in a plan against a provider;
/// unresolvable queries become warnings, not errors — an architecture
/// can name structure the swarm has not discovered yet (that is a
/// research TODO, not a crash). Strict mode turns those warnings into
/// failure at the CLI layer.
pub fn resolve_plan(plan: &mut Plan, provider: &dyn Provider) {
    for leaf in &mut plan.leaves {
        leaf.resolved = provider.resolve_query(&leaf.query);
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
        bridge.resolved = provider.resolve_query(&Query {
            class: bridge.class.clone(),
            min_persistence: 0.0,
            material: None,
        });
    }
    plan.warnings.dedup();

    plan.registry_snapshots.push(provider.snapshot());
    plan.resolution_report = Some(ResolutionReport {
        leaves_total: plan.leaves.len(),
        leaves_resolved: plan.leaves.iter().filter(|l| l.resolved.is_some()).count(),
        bridges_total: plan.bridges.len(),
        bridges_resolved: plan.bridges.iter().filter(|b| b.resolved.is_some()).count(),
    });
}

/// How many required components a resolved plan still lacks — what
/// `strict` mode refuses to ship (ADR-0002 §10).
pub fn unresolved_count(plan: &Plan) -> usize {
    plan.leaves.iter().filter(|l| l.resolved.is_none()).count()
        + plan.bridges.iter().filter(|b| b.resolved.is_none()).count()
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
    fn fixture_provider_resolves_without_any_registry_file() {
        let fixture = FixtureProvider::new(vec![
            Resolved {
                provider: PROVIDER_FIXTURE,
                id: "FIX-000001".into(),
                class: "MemorySeed".into(),
                persistence: 0.9,
                noise_tolerance: 0.9,
                material: "ideal_resonator".into(),
            },
            Resolved {
                provider: PROVIDER_FIXTURE,
                id: "FIX-000002".into(),
                class: "HarmonicBridge".into(),
                persistence: 0.5,
                noise_tolerance: 0.5,
                material: "ideal_resonator".into(),
            },
        ]);

        let src = r#"
            cell Bank(n) {
                when n > 1  => split Bank(n / 2), Bank(n / 2) bridge "Harmonic Bridge"
                when always => base "Memory Seed" min_persistence 0.4
            }
            grow Bank(4)
        "#;
        let mut plan = grow(&parse(src).unwrap()).unwrap();
        resolve_plan(&mut plan, &fixture);

        assert_eq!(unresolved_count(&plan), 0);
        assert!(plan
            .leaves
            .iter()
            .all(|l| l.resolved.as_ref().unwrap().provider == PROVIDER_FIXTURE));
        assert!(plan.bridges.iter().all(|b| b.resolved.is_some()));
        assert_eq!(plan.registry_snapshots[0].provider, PROVIDER_FIXTURE);
        assert_eq!(plan.registry_snapshots[0].source, "(static fixture)");
        // Same query semantics as the Crystal registry: floor excludes.
        let q = crate::parser::Query {
            class: "Memory Seed".into(),
            min_persistence: 0.95,
            material: None,
        };
        assert!(fixture.resolve_query(&q).is_none());
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

        // ADR-0002 Phase 1: the plan names its evidence sources and
        // records resolution outcomes.
        assert_eq!(plan.registry_snapshots.len(), 1);
        assert_eq!(plan.registry_snapshots[0].provider, PROVIDER_CRYSTAL);
        assert_eq!(plan.registry_snapshots[0].primitives, 3);
        let report = plan.resolution_report.as_ref().unwrap();
        assert_eq!(report.leaves_total, 5);
        assert_eq!(report.leaves_resolved, 4);
        assert_eq!(report.bridges_total, 3);
        assert_eq!(report.bridges_resolved, 3);
        assert_eq!(unresolved_count(&plan), 1);
        assert!(plan
            .leaves
            .iter()
            .filter_map(|l| l.resolved.as_ref())
            .all(|r| r.provider == PROVIDER_CRYSTAL));
        let _ = std::fs::remove_dir_all(dir);
    }
}
