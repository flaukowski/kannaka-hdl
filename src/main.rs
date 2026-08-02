use clap::{Parser, Subcommand, ValueEnum};
use kannaka_hdl::emit;
use kannaka_hdl::grow::{fnv1a64, grow, UnresolvedMode};
use kannaka_hdl::parser::parse;
use kannaka_hdl::registry::{default_path, resolve_plan, unresolved_count, Registry};
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
    Grow {
        /// Path to a .khdl file
        file: PathBuf,
        /// kannaka-crystal registry.json (default: crystal's data dir)
        #[arg(long)]
        registry: Option<PathBuf>,
        /// Skip registry resolution entirely
        #[arg(long)]
        no_resolve: bool,
        /// Unresolved-component policy: strict fails, stub withholds
        /// execution, speculative approximates (ADR-0002 §10)
        #[arg(long, value_enum, default_value = "speculative")]
        unresolved: UnresolvedCli,
        #[arg(long, value_enum, default_value = "json")]
        emit: EmitKind,
        /// Output file (default: stdout)
        #[arg(short, long)]
        out: Option<PathBuf>,
    },
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
        Command::Grow {
            file,
            registry,
            no_resolve,
            unresolved,
            emit,
            out,
        } => {
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
                match Registry::load(&path) {
                    Ok(reg) => {
                        resolve_plan(&mut plan, &[&reg]);
                        eprintln!(
                            "resolved against {} ({} primitives), {} warning(s)",
                            reg.source.display(),
                            reg.len(),
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
                    Err(e) if strict => {
                        return Err(format!("strict mode: registry unavailable: {e}"));
                    }
                    Err(e) => {
                        plan.warnings.push(format!("registry unavailable: {e}"));
                        eprintln!("warning: {e} — emitting unresolved plan");
                    }
                }
            }

            plan.seal(&source);
            if !plan.discovery_requests.is_empty() {
                eprintln!(
                    "{} capability discovery request(s) in plan — publishable to the swarm (ADR-0002 §14)",
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
