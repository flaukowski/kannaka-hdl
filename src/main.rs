use clap::{Parser, Subcommand, ValueEnum};
use kannaka_hdl::emit;
use kannaka_hdl::grow::grow;
use kannaka_hdl::parser::parse;
use kannaka_hdl::registry::{default_path, resolve_plan, Registry};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "kannaka-hdl",
    version,
    about = "KannakaHDL — grow architectures from discovered Crystal Primitives"
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
                "ok: {} cell(s), grown {} -> {} leaves, {} bridges, max depth {}",
                program.cells.len(),
                plan.grown_from,
                plan.leaves.len(),
                plan.bridges.len(),
                plan.leaves.iter().map(|l| l.depth).max().unwrap_or(0)
            );
            Ok(())
        }
        Command::Grow {
            file,
            registry,
            no_resolve,
            emit,
            out,
        } => {
            let source =
                std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
            let program = parse(&source).map_err(|e| e.to_string())?;
            let mut plan = grow(&program).map_err(|e| e.to_string())?;

            if !no_resolve {
                let path = registry.unwrap_or_else(default_path);
                match Registry::load(&path) {
                    Ok(reg) => {
                        resolve_plan(&mut plan, &reg);
                        eprintln!(
                            "resolved against {} ({} primitives), {} warning(s)",
                            reg.source.display(),
                            reg.len(),
                            plan.warnings.len()
                        );
                        for w in &plan.warnings {
                            eprintln!("  warning: {w}");
                        }
                    }
                    Err(e) => {
                        plan.warnings.push(format!("registry unavailable: {e}"));
                        eprintln!("warning: {e} — emitting unresolved plan");
                    }
                }
            }

            let output = match emit {
                EmitKind::Json => emit::emit_json(&plan),
                EmitKind::Crystal => emit::emit_crystal(&plan),
                EmitKind::Html => emit::emit_html(&plan),
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
