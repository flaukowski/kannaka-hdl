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
    /// Normalized 256-dim primitive signature (crystal ADR-0004);
    /// absent in very old rows.
    #[serde(default)]
    signature: Vec<f64>,
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
    /// Whether this provider can resolve the given component type
    /// (`base crystal.primitive …`). Untyped queries always pass.
    fn supports_type(&self, component_type: &str) -> bool;
    /// Every component satisfying the query's filters — strategy
    /// selection over the candidates happens in [`resolve_plan`]
    /// (ADR-0002 §9).
    fn candidates(&self, query: &Query) -> Vec<Resolved>;
    /// Best single answer (highest persistence); `None` means this
    /// provider has no component satisfying the query (a research
    /// TODO, not an error).
    fn resolve_query(&self, query: &Query) -> Option<Resolved> {
        self.candidates(query)
            .into_iter()
            .max_by(|a, b| a.persistence.total_cmp(&b.persistence))
    }
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
    /// Crystal primitive signature when the provider has one — the
    /// input to cross-domain transforms (ADR-0002 §13). Elided from
    /// plan JSON when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub signature: Vec<f64>,
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

/// A structured research request for a capability the swarm has not
/// grown yet (ADR-0002 §14): a missing component is a discovery task,
/// not a dead end. Publishable over NATS as-is; `requested_by_plan` is
/// stamped with the plan hash by [`crate::grow::Plan::seal`].
#[derive(Debug, Clone, Serialize)]
pub struct DiscoveryRequest {
    pub request_type: &'static str,
    pub domain: String,
    pub component_type: Option<String>,
    pub class: String,
    pub constraints: DiscoveryConstraints,
    pub requested_by_plan: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiscoveryConstraints {
    pub min_persistence: f64,
    pub min_noise_tolerance: f64,
    pub material: Option<String>,
}

impl DiscoveryRequest {
    fn for_query(query: &Query) -> Self {
        DiscoveryRequest {
            request_type: "capability_discovery",
            domain: query.domain.clone(),
            component_type: query.component_type.clone(),
            class: query.class.clone(),
            constraints: DiscoveryConstraints {
                min_persistence: query.min_persistence,
                min_noise_tolerance: query.min_noise_tolerance,
                material: query.material.clone(),
            },
            requested_by_plan: String::new(),
        }
    }
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
                signature: p.signature.clone(),
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

    fn supports_type(&self, component_type: &str) -> bool {
        component_type == "primitive"
    }

    fn candidates(&self, query: &Query) -> Vec<Resolved> {
        let want = normalize(&query.class);
        self.primitives
            .iter()
            .filter(|p| normalize(&p.class) == want)
            .filter(|p| p.persistence >= query.min_persistence)
            .filter(|p| p.noise_tolerance >= query.min_noise_tolerance)
            .filter(|p| query.material.as_deref().is_none_or(|m| p.material_id == m))
            .map(|p| Resolved {
                provider: PROVIDER_CRYSTAL,
                id: p.id.clone(),
                class: p.class.clone(),
                persistence: p.persistence,
                noise_tolerance: p.noise_tolerance,
                material: p.material_id.clone(),
                signature: p.signature.clone(),
            })
            .collect()
    }
}

/// A static, in-memory provider for tests and offline development
/// (ADR-0002 Phase 2) — no registry file, no sibling repo required.
/// Resolution semantics match the Crystal registry: tolerant class
/// normalization, persistence floor, optional material, best
/// persistence wins. `with_domain` makes it stand in for domains whose
/// live providers haven't arrived yet (e.g. `memory`).
pub struct FixtureProvider {
    pub domain: &'static str,
    pub components: Vec<Resolved>,
}

/// The provider identifier fixtures answer as.
pub const PROVIDER_FIXTURE: &str = "fixture";

impl FixtureProvider {
    pub fn new(components: Vec<Resolved>) -> Self {
        Self::with_domain(crate::grow::DOMAIN_CRYSTAL, components)
    }

    pub fn with_domain(domain: &'static str, components: Vec<Resolved>) -> Self {
        FixtureProvider { domain, components }
    }
}

impl Provider for FixtureProvider {
    fn id(&self) -> &'static str {
        PROVIDER_FIXTURE
    }

    fn domain(&self) -> &'static str {
        self.domain
    }

    fn snapshot(&self) -> RegistrySnapshot {
        RegistrySnapshot {
            provider: PROVIDER_FIXTURE,
            domain: self.domain(),
            source: "(static fixture)".into(),
            primitives: self.components.len(),
        }
    }

    fn supports_type(&self, _component_type: &str) -> bool {
        true
    }

    fn candidates(&self, query: &Query) -> Vec<Resolved> {
        let want = normalize(&query.class);
        self.components
            .iter()
            .filter(|c| normalize(&c.class) == want)
            .filter(|c| c.persistence >= query.min_persistence)
            .filter(|c| c.noise_tolerance >= query.min_noise_tolerance)
            .filter(|c| query.material.as_deref().is_none_or(|m| c.material == m))
            .map(|c| Resolved {
                provider: PROVIDER_FIXTURE,
                ..c.clone()
            })
            .collect()
    }
}

/// Per-plan resolution state for stateful strategies (ADR-0002 §9):
/// `unique` claims component ids plan-wide; `diverse` round-robins per
/// domain/class.
#[derive(Default)]
struct StrategyState {
    claimed: std::collections::HashSet<String>,
    cursors: std::collections::HashMap<String, usize>,
}

/// Resolve one query against the provider answering its domain.
/// Returns the pick, or `None` plus the reason it could not be
/// satisfied — and, when the capability itself is missing (rather than
/// contended), a structured discovery request for the swarm (§14).
fn resolve_component(
    providers: &[&dyn Provider],
    query: &Query,
    state: &mut StrategyState,
) -> (Option<Resolved>, Option<String>, Option<DiscoveryRequest>) {
    use crate::parser::Strategy;

    let Some(provider) = providers.iter().find(|p| p.domain() == query.domain) else {
        return (
            None,
            Some(format!(
                "no provider for domain \"{}\" — Memory and Swarm providers arrive with later ADR-0002 phases",
                query.domain
            )),
            Some(DiscoveryRequest::for_query(query)),
        );
    };
    if let Some(t) = &query.component_type {
        if !provider.supports_type(t) {
            return (
                None,
                Some(format!(
                    "provider {} does not resolve component type \"{}\"",
                    provider.id(),
                    t
                )),
                None,
            );
        }
    }
    let mut candidates = provider.candidates(query);
    candidates.sort_by(|a, b| b.persistence.total_cmp(&a.persistence));
    if candidates.is_empty() {
        let noise = if query.min_noise_tolerance > 0.0 {
            format!(", min_noise_tolerance {}", query.min_noise_tolerance)
        } else {
            String::new()
        };
        let material = query
            .material
            .as_deref()
            .map(|m| format!(", material {m}"))
            .unwrap_or_default();
        return (
            None,
            Some(format!(
                "no component (class \"{}\", min_persistence {}{noise}{material}) — swarm has not grown one yet",
                query.class, query.min_persistence
            )),
            Some(DiscoveryRequest::for_query(query)),
        );
    }
    match query.strategy {
        Strategy::Best => (Some(candidates.swap_remove(0)), None, None),
        Strategy::Robust => (
            candidates
                .into_iter()
                .max_by(|a, b| a.noise_tolerance.total_cmp(&b.noise_tolerance)),
            None,
            None,
        ),
        Strategy::Unique => match candidates
            .into_iter()
            .find(|c| !state.claimed.contains(&c.id))
        {
            Some(c) => {
                state.claimed.insert(c.id.clone());
                (Some(c), None, None)
            }
            // The capability exists — it is contended, not missing — so
            // no discovery request is generated.
            None => (
                None,
                Some(format!(
                    "unique strategy exhausted for class \"{}\" — every matching component is already claimed",
                    query.class
                )),
                None,
            ),
        },
        Strategy::Diverse => {
            let key = format!("{}/{}", query.domain, normalize(&query.class));
            let cursor = state.cursors.entry(key).or_insert(0);
            let pick = candidates[*cursor % candidates.len()].clone();
            *cursor += 1;
            (Some(pick), None, None)
        }
    }
}

/// Resolve every leaf and bridge in a plan; each query goes to the
/// provider answering its domain (a plan may span Crystal and Memory
/// components — ADR-0002 §12). Unresolvable queries become warnings,
/// not errors — an architecture can name structure the swarm has not
/// discovered yet (that is a research TODO, not a crash). Strict mode
/// turns those warnings into failure at the CLI layer.
pub fn resolve_plan(plan: &mut Plan, providers: &[&dyn Provider]) {
    let mut state = StrategyState::default();
    let mut requested = std::collections::HashSet::new();
    let mut request_if_new = |requests: &mut Vec<DiscoveryRequest>,
                              request: Option<DiscoveryRequest>| {
        if let Some(request) = request {
            let key = serde_json::to_string(&request).expect("request serializes");
            if requested.insert(key) {
                requests.push(request);
            }
        }
    };

    for leaf in &mut plan.leaves {
        let (resolved, reason, request) = resolve_component(providers, &leaf.query, &mut state);
        leaf.resolved = resolved;
        if let Some(reason) = reason {
            plan.warnings.push(format!("{}: {reason}", leaf.cell));
        }
        request_if_new(&mut plan.discovery_requests, request);
    }
    for bridge in &mut plan.bridges {
        let (resolved, reason, request) = resolve_component(providers, &bridge.query, &mut state);
        bridge.resolved = resolved;
        if let Some(reason) = reason {
            plan.warnings
                .push(format!("coupling \"{}\": {reason}", bridge.query.class));
        }
        request_if_new(&mut plan.discovery_requests, request);
    }
    plan.warnings.dedup();

    for provider in providers {
        plan.registry_snapshots.push(provider.snapshot());
    }
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

/// The provider identifier the live Kannaka Memory CLI answers as.
pub const PROVIDER_MEMORY_CLI: &str = "kannaka-memory-cli";

/// Resonance recall always returns the nearest memories, so resolving
/// a class against unrelated content would be vacuous. Without an
/// explicit floor a candidate must resonate at least this strongly; a
/// query's explicit `min_noise_tolerance` replaces the default in
/// either direction — today's hash encoder rarely clears 0.5 even for
/// related content (the known encoder floor), and an author declaring
/// their own evidence threshold beats a conservative default.
const MEMORY_MIN_SIMILARITY: f64 = 0.5;

/// A live Kannaka Memory provider (ADR-0002 §2/§12) backed by the
/// `kannaka` CLI's bilateral resonance recall
/// (`kannaka recall <class> --top-k N --envelope`, envelope schema 1.0).
///
/// Contract mapping (`memory-recall-v1`): memory `strength` →
/// `persistence`, recall `similarity` → `noise_tolerance`, material is
/// always `"hrm"`. The mapping is an analogy, not an identity — see
/// ADR-0002 §13 ("a transform is not assumed to preserve meaning
/// merely because two systems use the word resonance").
pub struct MemoryCliProvider {
    pub binary: PathBuf,
    pub top_k: usize,
}

impl MemoryCliProvider {
    pub fn new(binary: PathBuf) -> Self {
        MemoryCliProvider { binary, top_k: 8 }
    }
}

/// Parse a `kannaka recall --envelope` stdout into candidates for a
/// query — pure so it stays testable without the binary. Lines before
/// the JSON envelope (backend banners) are skipped.
fn parse_recall_envelope(stdout: &str, query: &Query) -> Vec<Resolved> {
    let Some(envelope) = stdout
        .lines()
        .filter(|l| l.trim_start().starts_with('{'))
        .find_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .filter(|v| v.get("schema_version").is_some())
    else {
        return Vec::new();
    };
    let Some(data) = envelope.get("data").and_then(|d| d.as_array()) else {
        return Vec::new();
    };
    let similarity_floor = if query.min_noise_tolerance > 0.0 {
        query.min_noise_tolerance
    } else {
        MEMORY_MIN_SIMILARITY
    };
    data.iter()
        .filter_map(|m| {
            let similarity = m.get("similarity")?.as_f64()?;
            let strength = m.get("strength")?.as_f64()?;
            let id = m.get("id")?.as_str()?;
            (similarity >= similarity_floor && strength >= query.min_persistence).then(|| {
                Resolved {
                    provider: PROVIDER_MEMORY_CLI,
                    id: id.to_string(),
                    class: query.class.clone(),
                    persistence: strength,
                    noise_tolerance: similarity,
                    material: "hrm".into(),
                    signature: Vec::new(),
                }
            })
        })
        .collect()
}

impl Provider for MemoryCliProvider {
    fn id(&self) -> &'static str {
        PROVIDER_MEMORY_CLI
    }

    fn domain(&self) -> &'static str {
        crate::grow::DOMAIN_MEMORY
    }

    fn snapshot(&self) -> RegistrySnapshot {
        RegistrySnapshot {
            provider: PROVIDER_MEMORY_CLI,
            domain: crate::grow::DOMAIN_MEMORY,
            source: format!("{} recall --envelope", self.binary.display()),
            primitives: 0,
        }
    }

    fn supports_type(&self, component_type: &str) -> bool {
        matches!(
            component_type,
            "glyph" | "memory" | "belief" | "context" | "dream"
        )
    }

    fn candidates(&self, query: &Query) -> Vec<Resolved> {
        // Material floors other than "hrm" can never match this medium.
        if query.material.as_deref().is_some_and(|m| m != "hrm") {
            return Vec::new();
        }
        let output = std::process::Command::new(&self.binary)
            .args([
                "recall",
                &query.class,
                "--top-k",
                &self.top_k.to_string(),
                "--envelope",
            ])
            .output();
        match output {
            Ok(out) => parse_recall_envelope(&String::from_utf8_lossy(&out.stdout), query),
            Err(e) => {
                eprintln!(
                    "warning: memory provider unavailable ({}: {e}) — memory queries stay unresolved",
                    self.binary.display()
                );
                Vec::new()
            }
        }
    }
}

/// The domain and provider id for registered composite architectures.
pub const DOMAIN_COMPOSITE: &str = "composite";
pub const PROVIDER_COMPOSITE: &str = "composite-registry";

/// A successfully validated architecture registered as a reusable
/// component (ADR-0002 §15): program+plan hashes, component
/// identities, coupling count, worst-case behavioral metrics, and the
/// expectation verdicts that validated it. Level 3 of the development
/// ladder — validated architectures become composite primitives.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct Composite {
    pub name: String,
    pub program_hash: String,
    pub plan_hash: String,
    pub components: Vec<String>,
    pub couplings: usize,
    /// Worst case over the plan's resolved components.
    pub persistence: f64,
    pub noise_tolerance: f64,
    /// Expectation verdicts at registration time (the evidence).
    pub evidence: serde_json::Value,
}

/// Where composites register by default.
pub fn composites_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".kannaka-hdl")
        .join("composites.json")
}

/// Register a fully resolved plan as a composite component. Refuses
/// plans with unresolved components — only validated architectures
/// become primitives. Replaces an existing composite of the same name.
pub fn register_composite(plan: &Plan, name: &str, path: &Path) -> Result<Composite, String> {
    if unresolved_count(plan) > 0 {
        return Err(format!(
            "cannot register \"{name}\": {} component(s) unresolved — only validated architectures become composites",
            unresolved_count(plan)
        ));
    }
    let resolved: Vec<&Resolved> = plan
        .leaves
        .iter()
        .filter_map(|l| l.resolved.as_ref())
        .chain(plan.bridges.iter().filter_map(|b| b.resolved.as_ref()))
        .collect();
    let worst = |f: fn(&&Resolved) -> f64| resolved.iter().map(f).fold(f64::INFINITY, f64::min);
    let composite = Composite {
        name: name.to_string(),
        program_hash: plan.program_hash.clone(),
        plan_hash: plan.plan_hash.clone(),
        components: resolved.iter().map(|r| r.id.clone()).collect(),
        couplings: plan.bridges.len(),
        persistence: worst(|r| r.persistence),
        noise_tolerance: worst(|r| r.noise_tolerance),
        evidence: serde_json::to_value(&plan.expectations).map_err(|e| e.to_string())?,
    };
    let mut all: Vec<Composite> = match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?,
        Err(_) => Vec::new(),
    };
    all.retain(|c| normalize(&c.name) != normalize(name));
    all.push(composite.clone());
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(
        path,
        serde_json::to_string_pretty(&all).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(composite)
}

/// Answers `base composite.architecture "Name"` queries from the
/// composite registry — validated architectures satisfying future base
/// cases (ADR-0002 §15, acceptance #11).
pub struct CompositeProvider {
    pub composites: Vec<Composite>,
    pub source: PathBuf,
}

impl CompositeProvider {
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(CompositeProvider {
            composites: serde_json::from_str(&text)
                .map_err(|e| format!("{}: {e}", path.display()))?,
            source: path.to_path_buf(),
        })
    }
}

impl Provider for CompositeProvider {
    fn id(&self) -> &'static str {
        PROVIDER_COMPOSITE
    }

    fn domain(&self) -> &'static str {
        DOMAIN_COMPOSITE
    }

    fn snapshot(&self) -> RegistrySnapshot {
        RegistrySnapshot {
            provider: PROVIDER_COMPOSITE,
            domain: DOMAIN_COMPOSITE,
            source: self.source.display().to_string(),
            primitives: self.composites.len(),
        }
    }

    fn supports_type(&self, component_type: &str) -> bool {
        component_type == "architecture"
    }

    fn candidates(&self, query: &Query) -> Vec<Resolved> {
        // Composites have no material; a material floor can't match.
        if query.material.is_some() {
            return Vec::new();
        }
        let want = normalize(&query.class);
        self.composites
            .iter()
            .filter(|c| normalize(&c.name) == want)
            .filter(|c| c.persistence >= query.min_persistence)
            .filter(|c| c.noise_tolerance >= query.min_noise_tolerance)
            .map(|c| Resolved {
                provider: PROVIDER_COMPOSITE,
                id: c.plan_hash.clone(),
                class: c.name.clone(),
                persistence: c.persistence,
                noise_tolerance: c.noise_tolerance,
                material: "composite".into(),
                signature: Vec::new(),
            })
            .collect()
    }
}

/// A declared expectation and its verdict (ADR-0002 §16).
#[derive(Debug, Clone, Serialize)]
pub struct Expectation {
    pub metric: String,
    pub cmp: String,
    pub expected: f64,
    pub observed: Option<f64>,
    /// `pass` | `fail` | `unsupported` (needs a backend runner) |
    /// `inconclusive` (nothing resolved to measure)
    pub status: &'static str,
}

/// Evaluate a program's `expect` declarations against the resolved
/// plan (ADR-0002 §16), recording verdicts in the plan and returning
/// the failure count. Compiler-verifiable metrics:
/// `unresolved_components`, `capacity` (leaf count), `couplings`
/// (coupling count), and worst-case `persistence` / `noise_tolerance`
/// over resolved components. Metrics that need a backend runner
/// (e.g. `recall_accuracy`, `swarm_agents`) report `unsupported` —
/// never a silent pass.
pub fn evaluate_expectations(plan: &mut Plan, expects: &[crate::parser::Expect]) -> usize {
    use crate::parser::Cmp;

    let worst = |plan: &Plan, f: fn(&Resolved) -> f64| -> Option<f64> {
        plan.leaves
            .iter()
            .filter_map(|l| l.resolved.as_ref())
            .chain(plan.bridges.iter().filter_map(|b| b.resolved.as_ref()))
            .map(f)
            .min_by(f64::total_cmp)
    };

    let mut failures = 0;
    for expect in expects {
        let measured: Option<Option<f64>> = match expect.metric.as_str() {
            "unresolved_components" => Some(Some(unresolved_count(plan) as f64)),
            "capacity" => Some(Some(plan.leaves.len() as f64)),
            "couplings" => Some(Some(plan.bridges.len() as f64)),
            "persistence" => Some(worst(plan, |r| r.persistence)),
            "noise_tolerance" => Some(worst(plan, |r| r.noise_tolerance)),
            _ => None,
        };
        let (status, observed) = match measured {
            None => ("unsupported", None),
            Some(None) => ("inconclusive", None),
            Some(Some(observed)) => {
                let holds = match expect.cmp {
                    Cmp::Lt => observed < expect.value,
                    Cmp::Le => observed <= expect.value,
                    Cmp::Gt => observed > expect.value,
                    Cmp::Ge => observed >= expect.value,
                    Cmp::Eq => observed == expect.value,
                    Cmp::Ne => observed != expect.value,
                };
                (if holds { "pass" } else { "fail" }, Some(observed))
            }
        };
        if status == "fail" {
            failures += 1;
        }
        plan.expectations.push(Expectation {
            metric: expect.metric.clone(),
            cmp: match expect.cmp {
                Cmp::Lt => "<",
                Cmp::Le => "<=",
                Cmp::Gt => ">",
                Cmp::Ge => ">=",
                Cmp::Eq => "==",
                Cmp::Ne => "!=",
            }
            .into(),
            expected: expect.value,
            observed,
            status,
        });
    }
    failures
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
                signature: Vec::new(),
            },
            Resolved {
                provider: PROVIDER_FIXTURE,
                id: "FIX-000002".into(),
                class: "HarmonicBridge".into(),
                persistence: 0.5,
                noise_tolerance: 0.5,
                material: "ideal_resonator".into(),
                signature: Vec::new(),
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
        resolve_plan(&mut plan, &[&fixture]);

        assert_eq!(unresolved_count(&plan), 0);
        assert!(plan
            .leaves
            .iter()
            .all(|l| l.resolved.as_ref().unwrap().provider == PROVIDER_FIXTURE));
        assert!(plan.bridges.iter().all(|b| b.resolved.is_some()));
        assert_eq!(plan.registry_snapshots[0].provider, PROVIDER_FIXTURE);
        assert_eq!(plan.registry_snapshots[0].source, "(static fixture)");
        // Same query semantics as the Crystal registry: floor excludes.
        assert!(fixture.resolve_query(&query("Memory Seed", 0.95)).is_none());
    }

    /// A crystal-domain query with defaults, floors aside.
    fn query(class: &str, min_persistence: f64) -> Query {
        Query {
            domain: "crystal".into(),
            component_type: None,
            class: class.into(),
            min_persistence,
            min_noise_tolerance: 0.0,
            material: None,
            strategy: crate::parser::Strategy::Best,
        }
    }

    fn seed(id: &str, persistence: f64, noise_tolerance: f64) -> Resolved {
        Resolved {
            provider: PROVIDER_FIXTURE,
            id: id.into(),
            class: "MemorySeed".into(),
            persistence,
            noise_tolerance,
            material: "ideal_resonator".into(),
            signature: Vec::new(),
        }
    }

    const UNBRIDGED_BANK: &str = r#"
        cell Bank(n) {
            when n > 1  => split Bank(n / 2), Bank(n / 2)
            when always => base "Memory Seed" strategy STRAT
        }
        grow Bank(4)
    "#;

    #[test]
    fn robust_strategy_prefers_noise_tolerance_over_persistence() {
        let fixture = FixtureProvider::new(vec![seed("FIX-A", 0.9, 0.5), seed("FIX-B", 0.5, 0.95)]);
        let mut q = query("Memory Seed", 0.0);
        q.strategy = crate::parser::Strategy::Robust;
        let mut state = StrategyState::default();
        let (picked, _, _) = resolve_component(&[&fixture], &q, &mut state);
        assert_eq!(picked.unwrap().id, "FIX-B");
    }

    #[test]
    fn unresolved_components_generate_deduped_discovery_requests() {
        let fixture = FixtureProvider::new(vec![]);
        let src = r#"
            cell Bank(n) {
                when n > 1  => split Bank(n / 2), Bank(n / 2)
                when always => base "Phase Knot" min_noise_tolerance 0.6
            }
            cell M() { when always => base memory.glyph "Identity Glyph" }
            grow Bank(4)
            grow M()
        "#;
        let mut plan = grow(&parse(src).unwrap()).unwrap();
        resolve_plan(&mut plan, &[&fixture]);

        assert_eq!(
            plan.discovery_requests.len(),
            2,
            "four identical leaf misses dedupe to one request, plus the memory glyph"
        );
        let knot = &plan.discovery_requests[0];
        assert_eq!(knot.request_type, "capability_discovery");
        assert_eq!(knot.domain, "crystal");
        assert_eq!(knot.class, "Phase Knot");
        assert_eq!(knot.constraints.min_noise_tolerance, 0.6);
        assert_eq!(plan.discovery_requests[1].domain, "memory");
        assert!(knot.requested_by_plan.is_empty(), "stamped by seal()");

        plan.seal(src);
        assert_eq!(plan.discovery_requests[0].requested_by_plan, plan.plan_hash);
        assert!(!plan.plan_hash.is_empty());
    }

    #[test]
    fn unique_strategy_claims_distinct_components_then_warns() {
        let fixture = FixtureProvider::new(vec![
            seed("FIX-A", 0.9, 0.5),
            seed("FIX-B", 0.8, 0.5),
            seed("FIX-C", 0.7, 0.5),
        ]);
        let src = UNBRIDGED_BANK.replace("STRAT", "unique");
        let mut plan = grow(&parse(&src).unwrap()).unwrap();
        resolve_plan(&mut plan, &[&fixture]);

        let ids: std::collections::HashSet<_> = plan
            .leaves
            .iter()
            .filter_map(|l| l.resolved.as_ref())
            .map(|r| r.id.clone())
            .collect();
        assert_eq!(ids.len(), 3, "three distinct components claimed");
        assert_eq!(unresolved_count(&plan), 1, "fourth leaf exhausts the pool");
        assert!(plan
            .warnings
            .iter()
            .any(|w| w.contains("unique strategy exhausted")));
    }

    #[test]
    fn diverse_strategy_round_robins_candidates() {
        let fixture = FixtureProvider::new(vec![seed("FIX-A", 0.9, 0.5), seed("FIX-B", 0.8, 0.5)]);
        let src = UNBRIDGED_BANK.replace("STRAT", "diverse");
        let mut plan = grow(&parse(&src).unwrap()).unwrap();
        resolve_plan(&mut plan, &[&fixture]);

        assert_eq!(unresolved_count(&plan), 0);
        let ids: Vec<_> = plan
            .leaves
            .iter()
            .map(|l| l.resolved.as_ref().unwrap().id.as_str())
            .collect();
        assert_eq!(ids, vec!["FIX-A", "FIX-B", "FIX-A", "FIX-B"]);
    }

    #[test]
    fn foreign_domains_resolve_to_an_honest_warning() {
        let fixture = FixtureProvider::new(vec![seed("FIX-A", 0.9, 0.5)]);
        let src = r#"
            cell M() { when always => base memory.glyph "Identity Glyph" }
            grow M()
        "#;
        let mut plan = grow(&parse(src).unwrap()).unwrap();
        resolve_plan(&mut plan, &[&fixture]);
        assert_eq!(unresolved_count(&plan), 1);
        assert!(plan
            .warnings
            .iter()
            .any(|w| w.contains("no provider for domain \"memory\"")));
        assert_eq!(plan.leaves[0].domain, "memory");
    }

    #[test]
    fn hybrid_plans_resolve_across_crystal_and_memory_providers() {
        let crystal = FixtureProvider::new(vec![seed("FIX-C", 0.9, 0.9)]);
        let memory = FixtureProvider::with_domain(
            crate::grow::DOMAIN_MEMORY,
            vec![Resolved {
                provider: PROVIDER_FIXTURE,
                id: "GLYPH-000001".into(),
                class: "IdentityGlyph".into(),
                persistence: 0.8,
                noise_tolerance: 0.7,
                material: "hrm".into(),
                signature: Vec::new(),
            }],
        );
        let src = r#"
            cell Pair()   { when always => split Seed(), Mirror() }
            cell Seed()   { when always => base "Memory Seed" }
            cell Mirror() { when always => base memory.glyph "Identity Glyph" }
            grow Pair()
        "#;
        let mut plan = grow(&parse(src).unwrap()).unwrap();
        resolve_plan(&mut plan, &[&crystal, &memory]);

        assert_eq!(unresolved_count(&plan), 0, "both domains resolve");
        assert_eq!(plan.registry_snapshots.len(), 2);
        let domains: Vec<_> = plan.leaves.iter().map(|l| l.domain.as_str()).collect();
        assert_eq!(domains, vec!["crystal", "memory"]);
        assert_eq!(plan.leaves[1].resolved.as_ref().unwrap().id, "GLYPH-000001");
    }

    #[test]
    fn composites_register_and_satisfy_future_base_queries() {
        let fixture = FixtureProvider::new(vec![seed("FIX-A", 0.7, 0.6), seed("FIX-B", 0.9, 0.8)]);
        let src = r#"
            cell Bank(n) {
                when n > 1  => split Bank(n / 2), Bank(n / 2)
                when always => base "Memory Seed" strategy unique
            }
            grow Bank(2)
        "#;
        let mut plan = grow(&parse(src).unwrap()).unwrap();
        resolve_plan(&mut plan, &[&fixture]);
        plan.seal(src);

        let dir = std::env::temp_dir().join(format!("khdl-comp-{}", std::process::id()));
        let path = dir.join("composites.json");
        let composite = register_composite(&plan, "Seed Pair", &path).unwrap();
        assert_eq!(composite.components.len(), 2);
        assert_eq!(composite.plan_hash, plan.plan_hash);
        assert!((composite.persistence - 0.7).abs() < 1e-9, "worst case");

        // The registered architecture now satisfies a base query.
        let provider = CompositeProvider::load(&path).unwrap();
        let mut q = query("Seed Pair", 0.5);
        q.domain = DOMAIN_COMPOSITE.into();
        q.component_type = Some("architecture".into());
        let hit = provider.resolve_query(&q).unwrap();
        assert_eq!(hit.provider, PROVIDER_COMPOSITE);
        assert_eq!(hit.id, plan.plan_hash);
        assert!(provider.resolve_query(&query("Seed Pair", 0.95)).is_none());

        // Unvalidated plans are refused.
        let mut bare = grow(&parse(src).unwrap()).unwrap();
        assert!(register_composite(&bare, "Nope", &path).is_err());
        bare.seal(src);
        assert!(register_composite(&bare, "Nope", &path).is_err());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn recall_envelope_parses_with_similarity_floor() {
        // Modeled on a real `kannaka recall --envelope` capture:
        // banner lines precede the schema-1.0 JSON envelope on one line.
        let stdout = concat!(
            "Using HRM backend (Holographic Resonance Medium)\n",
            "[hrm] Using Holographic Resonance Medium - storage IS computation\n",
            r#"{"command":"recall","data":[{"age_hours":1.0,"content":"a","id":"MEM-1","layer":0,"similarity":0.81,"strength":0.62},{"age_hours":2.0,"content":"b","id":"MEM-2","layer":0,"similarity":0.31,"strength":0.9}],"errors":[],"schema_version":"1.0"}"#,
            "\n"
        );

        let q = query("Identity Glyph", 0.0);
        let got = parse_recall_envelope(stdout, &q);
        assert_eq!(got.len(), 1, "similarity 0.31 falls below the 0.5 floor");
        assert_eq!(got[0].id, "MEM-1");
        assert_eq!(got[0].provider, PROVIDER_MEMORY_CLI);
        assert_eq!(got[0].class, "Identity Glyph");
        assert!((got[0].persistence - 0.62).abs() < 1e-9);
        assert!((got[0].noise_tolerance - 0.81).abs() < 1e-9);

        // An explicit floor replaces the default in both directions:
        // raised, it excludes everything; lowered, it admits weaker
        // resonance (today's hash encoder rarely clears 0.5).
        let mut strict_q = query("Identity Glyph", 0.0);
        strict_q.min_noise_tolerance = 0.9;
        assert!(parse_recall_envelope(stdout, &strict_q).is_empty());
        let mut loose_q = query("Identity Glyph", 0.0);
        loose_q.min_noise_tolerance = 0.3;
        assert_eq!(parse_recall_envelope(stdout, &loose_q).len(), 2);

        // Garbage in, empty out — never a panic.
        assert!(parse_recall_envelope("no json here", &q).is_empty());
    }

    #[test]
    fn expectations_evaluate_pass_fail_unsupported_inconclusive() {
        use crate::parser::{Cmp, Expect};
        let fixture = FixtureProvider::new(vec![seed("FIX-A", 0.7, 0.6)]);
        let src = r#"
            cell Bank(n) {
                when n > 1  => split Bank(n / 2), Bank(n / 2)
                when always => base "Memory Seed"
            }
            grow Bank(4)
        "#;
        let mut plan = grow(&parse(src).unwrap()).unwrap();
        resolve_plan(&mut plan, &[&fixture]);

        let expects = vec![
            Expect {
                metric: "unresolved_components".into(),
                cmp: Cmp::Eq,
                value: 0.0,
                line: 1,
            },
            Expect {
                metric: "capacity".into(),
                cmp: Cmp::Ge,
                value: 8.0,
                line: 2,
            },
            Expect {
                metric: "noise_tolerance".into(),
                cmp: Cmp::Ge,
                value: 0.5,
                line: 3,
            },
            Expect {
                metric: "recall_accuracy".into(),
                cmp: Cmp::Ge,
                value: 0.8,
                line: 4,
            },
        ];
        let failures = evaluate_expectations(&mut plan, &expects);
        let statuses: Vec<&str> = plan.expectations.iter().map(|e| e.status).collect();
        assert_eq!(statuses, vec!["pass", "fail", "pass", "unsupported"]);
        assert_eq!(
            failures, 1,
            "capacity 4 < 8 fails; unsupported is not a failure"
        );
        assert_eq!(plan.expectations[1].observed, Some(4.0));

        // Nothing resolved -> worst-case metrics are inconclusive.
        let mut bare = grow(&parse(src).unwrap()).unwrap();
        let failures = evaluate_expectations(
            &mut bare,
            &[Expect {
                metric: "persistence".into(),
                cmp: Cmp::Ge,
                value: 0.1,
                line: 1,
            }],
        );
        assert_eq!(failures, 0);
        assert_eq!(bare.expectations[0].status, "inconclusive");
    }

    #[test]
    fn noise_tolerance_floor_filters_candidates() {
        let fixture = FixtureProvider::new(vec![seed("FIX-A", 0.9, 0.3), seed("FIX-B", 0.5, 0.9)]);
        let mut q = query("Memory Seed", 0.0);
        q.min_noise_tolerance = 0.8;
        assert_eq!(fixture.resolve_query(&q).unwrap().id, "FIX-B");
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
        resolve_plan(&mut plan, &[&reg]);

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
