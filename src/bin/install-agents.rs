use forge_lib::deploy::provider::Provider;
use forge_lib::deploy::{self, DeployResult};
use forge_lib::sidecar::SidecarConfig;
use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

struct Args {
    src_dir: String,
    scope: String,
    dry_run: bool,
    clean: bool,
    dst_override: Option<String>,
}

fn parse_args() -> Result<Args, ExitCode> {
    let args: Vec<String> = env::args().collect();
    let mut src_dir: Option<String> = None;
    let mut scope = "all".to_string();
    let mut dry_run = false;
    let mut clean = false;
    let mut dst_override: Option<String> = None;
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--version" => {
                println!("install-agents {}", env!("CARGO_PKG_VERSION"));
                return Err(ExitCode::SUCCESS);
            }
            "--dry-run" => dry_run = true,
            "--clean" => clean = true,
            "--scope" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --scope requires a value");
                    return Err(ExitCode::from(1));
                }
                scope.clone_from(&args[i]);
            }
            "--dst" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --dst requires a value");
                    return Err(ExitCode::from(1));
                }
                dst_override = Some(args[i].clone());
            }
            "-h" | "--help" => {
                println!(
                    "Usage: install-agents <agents-dir> [--scope user|workspace|all] \
                     [--dry-run] [--clean] [--dst <path>]"
                );
                return Err(ExitCode::SUCCESS);
            }
            arg if arg.starts_with('-') => {
                eprintln!("Error: unknown flag {arg}");
                return Err(ExitCode::from(1));
            }
            _ => {
                src_dir = Some(args[i].clone());
            }
        }
        i += 1;
    }

    let Some(src_dir) = src_dir else {
        eprintln!("Error: source directory required.");
        eprintln!(
            "Usage: install-agents <agents-dir> [--scope user|workspace|all] \
             [--dry-run] [--clean] [--dst <path>]"
        );
        return Err(ExitCode::from(1));
    };

    Ok(Args {
        src_dir,
        scope,
        dry_run,
        clean,
        dst_override,
    })
}

fn read_module_name() -> Option<String> {
    let content = std::fs::read_to_string("module.yaml").ok()?;
    forge_lib::parse::fm_value(&content, "name").or_else(|| {
        content.lines().find_map(|l| {
            l.strip_prefix("name:")
                .map(|v| v.trim().trim_matches('"').trim_matches('\'').to_string())
        })
    })
}

fn run(args: &Args) -> ExitCode {
    let src_path = Path::new(&args.src_dir);
    if !src_path.is_dir() {
        eprintln!("Error: not a directory: {}", args.src_dir);
        return ExitCode::from(1);
    }

    let source_prefix = read_module_name()
        .map(|name| format!("{name}/{}", args.src_dir))
        .unwrap_or_default();

    let config = SidecarConfig::load(Path::new("."));

    let dirs = if let Some(ref dst) = args.dst_override {
        vec![PathBuf::from(dst)]
    } else {
        let home = env::var("HOME").unwrap_or_default();
        match deploy::scope_dirs(&args.scope, Path::new(&home)) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Error: {e}");
                return ExitCode::from(1);
            }
        }
    };

    for dst_dir in &dirs {
        let provider = Provider::from_path(dst_dir);
        eprintln!("Targeting provider directory: {}", dst_dir.display());

        if args.clean {
            match deploy::clean_agents(src_path, dst_dir, args.dry_run) {
                Ok(removed) => {
                    for name in &removed {
                        if args.dry_run {
                            println!("[dry-run] Would remove: {name}.md");
                        } else {
                            println!("Removed: {name}.md");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    return ExitCode::from(1);
                }
            }
        }

        if let Err(code) = deploy_to_dir(
            src_path,
            dst_dir,
            provider,
            &config,
            args.dry_run,
            &source_prefix,
        ) {
            return code;
        }
    }

    ExitCode::SUCCESS
}

fn deploy_to_dir(
    src_path: &Path,
    dst_dir: &Path,
    provider: Provider,
    config: &SidecarConfig,
    dry_run: bool,
    source_prefix: &str,
) -> Result<(), ExitCode> {
    let results =
        deploy::deploy_agents_from_dir(src_path, dst_dir, provider, config, dry_run, source_prefix)
            .map_err(|e| {
                eprintln!("Error: {e}");
                ExitCode::from(1)
            })?;

    for (filename, result) in &results {
        let name = filename.trim_end_matches(".md");
        match result {
            DeployResult::Deployed => {
                if dry_run {
                    println!(
                        "[dry-run] Would install: {name}.md to {}",
                        dst_dir.display()
                    );
                } else {
                    println!("Installed: {name}.md to {}", dst_dir.display());
                }
            }
            DeployResult::SkippedUserOwned => {
                eprintln!("Warning: Skipping {name}.md — user-created agent (no source field)");
            }
            DeployResult::SkippedTemplate | DeployResult::SkippedNoName => {}
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match parse_args() {
        Ok(ref args) => run(args),
        Err(code) => code,
    }
}
