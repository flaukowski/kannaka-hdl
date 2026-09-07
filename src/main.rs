use clap::{Parser, Subcommand, ValueEnum};
use kannaka_hdl::emit;
use kannaka_hdl::grow::{fnv1a64, grow, UnresolvedMode, DOMAIN_CODE, DOMAIN_CRYSTAL, DOMAIN_MIND};
use kannaka_hdl::parser::parse;
use kannaka_hdl::registry::{
    composites_path, default_path, evaluate_expectations, resolve_plan, unresolved_count,
    CodeGraphProvider, CompositeProvider, MemoryCliProvider, Provider, Registry, PROVIDER_MIND,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "kannaka-hdl",
    version,
    about = "KannakaHDL — the Holographic Development Language: grow architectures from discovered components (ADR-0002)"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Copy, ValueEnum)]
enum EmitKind {
    Json,
    Crystal,
    Html,
    /// Kannaka Memory architecture plan (ADR-0002 §12, memory-plan-v1)
    Memory,
}

/// Unresolved-component policy (ADR-0002 §10). `speculative` matches the
/// historical behavior (proxy pulses); scientific runs want `strict`.
#[derive(Clone, Copy, ValueEnum)]
enum UnresolvedCli {
    Strict,
    Stub,
    Speculative,
}

impl From<UnresolvedCli> for UnresolvedMode {
    fn from(mode: UnresolvedCli) -> Self {
        match mode {
            UnresolvedCli::Strict => UnresolvedMode::Strict,
            UnresolvedCli::Stub => UnresolvedMode::Stub,
            UnresolvedCli::Speculative => UnresolvedMode::Speculative,
        }
    }
}

#[derive(Subcommand)]
enum Command {
    /// Parse and grow a .khdl program, reporting counts (no output file)
    Check {
        /// Path to a .khdl file
        file: PathBuf,
    },
    /// Grow a .khdl program, resolve against the primitive registry, emit
    Grow(Box<GrowArgs>),
}

/// `grow`'s arguments. A struct rather than variant fields, boxed into
/// [`Command::Grow`]: the variant outgrew `Check` by 250 bytes and this crate
/// suppresses no lints.
#[derive(clap::Args)]
pub struct GrowArgs {
    /// Path to a .khdl file
    file: PathBuf,
    /// kannaka-crystal registry.json (default: crystal's data dir)
    #[arg(long)]
    registry: Option<PathBuf>,
    /// A mind registry (crystal schema) answering `base mind.faculty …`
    /// queries (v0.10; default: $KANNAKA_MIND_REGISTRY)
    #[arg(long)]
    mind_registry: Option<PathBuf>,
    /// Skip registry resolution entirely
    #[arg(long)]
    no_resolve: bool,
    /// Unresolved-component policy: strict fails, stub withholds
    /// execution, speculative approximates (ADR-0002 §10)
    #[arg(long, value_enum, default_value = "speculative")]
    unresolved: UnresolvedCli,
    /// Resolve memory-domain queries against a live Kannaka Memory
    /// via the kannaka CLI (optionally give the binary path)
    #[arg(long, num_args = 0..=1, default_missing_value = "kannaka")]
    memory_provider: Option<PathBuf>,
    /// Resolve `base code.symbol …` queries against a kannaka-memory
    /// code-graph index (v0.11; default: $KANNAKA_CODE_INDEX)
    #[arg(long)]
    code_index: Option<PathBuf>,
    /// The index tool that reads it (default: $KANNAKA_CODE_TOOL, else
    /// graph_index.py beside the index)
    #[arg(long)]
    code_tool: Option<PathBuf>,
    /// Publish the plan's capability discovery requests to the
    /// swarm work queue via `kannaka swarm enqueue` (ADR-0002 §14)
    #[arg(long, num_args = 0..=1, default_missing_value = "kannaka")]
    publish_discovery: Option<PathBuf>,
    /// Register the validated plan as a composite component under
    /// this name (ADR-0002 §15); requires full resolution
    #[arg(long, value_name = "NAME")]
    register_composite: Option<String>,
    #[arg(long, value_enum, default_value = "json")]
    emit: EmitKind,
    /// Output file (default: stdout)
    #[arg(short, long)]
    out: Option<PathBuf>,
}

fn main() {
    if let Err(e) = dispatch(Cli::parse().command) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn dispatch(command: Command) -> Result<(), String> {
    match command {
        Command::Check { file } => {
            let source =
                std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
            let program = parse(&source).map_err(|e| e.to_string())?;
            let plan = grow(&program).map_err(|e| e.to_string())?;
            println!(
                "ok: {} cell(s), grown {} -> {} leaves, {} bridges, max depth {}, program {}",
                program.cells.len(),
                plan.grown_from,
                plan.leaves.len(),
                plan.bridges.len(),
                plan.leaves.iter().map(|l| l.depth).max().unwrap_or(0),
                fnv1a64(source.as_bytes())
            );
            Ok(())
        }
        Command::Grow(args) => {
            let GrowArgs {
                file,
                registry,
                mind_registry,
                no_resolve,
                unresolved,
                memory_provider,
                code_index,
                code_tool,
                publish_discovery,
                register_composite,
                emit,
                out,
            } = *args;
            let source =
                std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
            let program = parse(&source).map_err(|e| e.to_string())?;
            let mut plan = grow(&program).map_err(|e| e.to_string())?;
            plan.unresolved_mode = unresolved.into();
            let strict = plan.unresolved_mode == UnresolvedMode::Strict;

            if no_resolve {
                if strict {
                    return Err(
                        "--unresolved strict requires registry resolution (drop --no-resolve)"
                            .into(),
                    );
                }
            } else {
                let path = registry.unwrap_or_else(default_path);
                let memory = memory_provider.map(MemoryCliProvider::new);
                // v0.10: a program that never asks for a crystal (a Mind, a memory
                // gate) does not need the crystal registry to exist — strict mode
                // only insists on the registries the plan actually queries.
                let wants_crystal = plan.leaves.iter().any(|l| l.domain == DOMAIN_CRYSTAL)
                    || plan.bridges.iter().any(|b| b.domain == DOMAIN_CRYSTAL);
                let crystal = match Registry::load(&path) {
                    Ok(reg) => Some(reg),
                    Err(e) if strict && wants_crystal => {
                        return Err(format!("strict mode: registry unavailable: {e}"));
                    }
                    Err(e) if wants_crystal => {
                        plan.warnings.push(format!("registry unavailable: {e}"));
                        eprintln!("warning: {e} — crystal queries stay unresolved");
                        None
                    }
                    Err(_) => None,
                };

                let mind_path = mind_registry
                    .or_else(|| std::env::var_os("KANNAKA_MIND_REGISTRY").map(PathBuf::from));
                let wants_mind = plan.leaves.iter().any(|l| l.domain == DOMAIN_MIND)
                    || plan.bridges.iter().any(|b| b.domain == DOMAIN_MIND);
                let mind = match mind_path {
                    Some(p) => match Registry::load_for(&p, DOMAIN_MIND, PROVIDER_MIND) {
                        Ok(reg) => Some(reg),
                        Err(e) if strict => {
                            return Err(format!("strict mode: mind registry unavailable: {e}"));
                        }
                        Err(e) => {
                            plan.warnings
                                .push(format!("mind registry unavailable: {e}"));
                            eprintln!("warning: {e} — mind queries stay unresolved");
                            None
                        }
                    },
                    None if wants_mind && strict => {
                        return Err(
                            "strict mode: program has mind.faculty queries but no --mind-registry / $KANNAKA_MIND_REGISTRY".into(),
                        );
                    }
                    None => None,
                };
                let code_index_path = code_index
                    .or_else(|| std::env::var_os("KANNAKA_CODE_INDEX").map(PathBuf::from));
                let wants_code = plan.leaves.iter().any(|l| l.domain == DOMAIN_CODE)
                    || plan.bridges.iter().any(|b| b.domain == DOMAIN_CODE);
                let code = match code_index_path {
                    Some(idx) if idx.exists() => {
                        let tool = code_tool
                            .or_else(|| std::env::var_os("KANNAKA_CODE_TOOL").map(PathBuf::from))
                            .unwrap_or_else(|| {
                                idx.parent()
                                    .unwrap_or_else(|| std::path::Path::new("."))
                                    .join("graph_index.py")
                            });
                        Some(CodeGraphProvider::new(tool, idx))
                    }
                    Some(idx) if strict && wants_code => {
                        return Err(format!(
                            "strict mode: code graph index not found at {}",
                            idx.display()
                        ));
                    }
                    Some(idx) => {
                        plan.warnings
                            .push(format!("code graph index not found: {}", idx.display()));
                        eprintln!(
                            "warning: no code graph index at {} — code queries stay unresolved",
                            idx.display()
                        );
                        None
                    }
                    None if wants_code && strict => {
                        return Err(
                            "strict mode: program has code.* queries but no --code-index / $KANNAKA_CODE_INDEX".into(),
                        );
                    }
                    None => None,
                };
                let mut providers: Vec<&dyn Provider> = Vec::new();
                if let Some(reg) = &crystal {
                    providers.push(reg);
                }
                if let Some(reg) = &mind {
                    providers.push(reg);
                }
                if let Some(memory) = &memory {
                    providers.push(memory);
                }
                if let Some(code) = &code {
                    providers.push(code);
                }
                let composites = CompositeProvider::load(&composites_path()).ok();
                if let Some(composites) = &composites {
                    providers.push(composites);
                }
                resolve_plan(&mut plan, &providers);
                eprintln!(
                    "resolved against {}{}, {} warning(s)",
                    crystal
                        .as_ref()
                        .map(|r| format!("{} ({} primitives)", r.source.display(), r.len()))
                        .unwrap_or_else(|| "no crystal registry".into()),
                    match (&mind, memory.is_some()) {
                        (Some(m), true) => format!(
                            " + mind {} ({} faculties) + live kannaka memory",
                            m.source.display(),
                            m.len()
                        ),
                        (Some(m), false) =>
                            format!(" + mind {} ({} faculties)", m.source.display(), m.len()),
                        (None, true) => " + live kannaka memory".to_string(),
                        (None, false) => String::new(),
                    },
                    plan.warnings.len()
                );
                for w in &plan.warnings {
                    eprintln!("  warning: {w}");
                }
                let missing = unresolved_count(&plan);
                if strict && missing > 0 {
                    return Err(format!(
                        "strict mode: {missing} component(s) unresolved — the swarm has not grown them yet (see warnings above)"
                    ));
                }
            }

            let expectation_failures = evaluate_expectations(&mut plan, &program.expects);
            for e in &plan.expectations {
                eprintln!(
                    "expect {} {} {} — {}{}",
                    e.metric,
                    e.cmp,
                    e.expected,
                    e.status,
                    e.observed
                        .map(|o| format!(" (observed {o})"))
                        .unwrap_or_default()
                );
            }
            if expectation_failures > 0 {
                return Err(format!(
                    "{expectation_failures} expectation(s) failed — evidence requirements not met, nothing emitted"
                ));
            }

            plan.seal(&source);
            if let Some(name) = register_composite {
                let path = composites_path();
                let composite = kannaka_hdl::registry::register_composite(&plan, &name, &path)?;
                eprintln!(
                    "registered composite \"{}\" ({} components, plan {}) at {}",
                    composite.name,
                    composite.components.len(),
                    composite.plan_hash,
                    path.display()
                );
            }
            if !plan.discovery_requests.is_empty() {
                eprintln!(
                    "{} capability discovery request(s) in plan — publishable to the swarm (ADR-0002 §14)",
                    plan.discovery_requests.len()
                );
            }
            if let Some(bin) = publish_discovery {
                let mut published = 0;
                for request in &plan.discovery_requests {
                    let payload = serde_json::to_string(request).map_err(|e| e.to_string())?;
                    match std::process::Command::new(&bin)
                        .args(["swarm", "enqueue", "capability_discovery", &payload])
                        .output()
                    {
                        Ok(o) if o.status.success() => published += 1,
                        Ok(o) => eprintln!(
                            "warning: enqueue failed for \"{}\": {}",
                            request.class,
                            String::from_utf8_lossy(&o.stderr).trim()
                        ),
                        Err(e) => {
                            eprintln!("warning: cannot run {}: {e}", bin.display());
                            break;
                        }
                    }
                }
                eprintln!(
                    "published {published}/{} discovery request(s) to the swarm work queue",
                    plan.discovery_requests.len()
                );
            }
            let output = match emit {
                EmitKind::Json => emit::emit_json(&plan),
                EmitKind::Crystal => emit::emit_crystal(&plan),
                EmitKind::Html => emit::emit_html(&plan),
                EmitKind::Memory => emit::emit_memory(&plan),
            };
            match out {
                Some(path) => {
                    std::fs::write(&path, output)
                        .map_err(|e| format!("{}: {e}", path.display()))?;
                    eprintln!("wrote {}", path.display());
                }
                None => println!("{output}"),
            }
            Ok(())
        }
    }
}
