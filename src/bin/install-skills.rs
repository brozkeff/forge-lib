use forge_lib::deploy::provider::Provider;
use forge_lib::sidecar::SidecarConfig;
use forge_lib::skill::{self, SkillInstallAction};
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

struct Args {
    skills_dir: String,
    provider: Provider,
    scope: String,
    dry_run: bool,
    clean: bool,
    dst_override: Option<String>,
    agents_dir: String,
    include_agent_wrappers: bool,
}

fn parse_args() -> Result<Args, ExitCode> {
    let args: Vec<String> = env::args().collect();
    let mut skills_dir: Option<String> = None;
    let mut provider_str: Option<String> = None;
    let mut scope = "workspace".to_string();
    let mut dry_run = false;
    let mut clean = false;
    let mut dst_override: Option<String> = None;
    let mut agents_dir = "agents".to_string();
    let mut include_agent_wrappers = false;
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--version" => {
                println!("install-skills {}", env!("CARGO_PKG_VERSION"));
                return Err(ExitCode::SUCCESS);
            }
            "--provider" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --provider requires a value");
                    return Err(ExitCode::from(1));
                }
                provider_str = Some(args[i].clone());
            }
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
            "--agents-dir" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --agents-dir requires a value");
                    return Err(ExitCode::from(1));
                }
                agents_dir.clone_from(&args[i]);
            }
            "--dry-run" => dry_run = true,
            "--clean" => clean = true,
            "--include-agent-wrappers" => include_agent_wrappers = true,
            "-h" | "--help" => {
                println!(
                    "Usage: install-skills <skills-dir> --provider claude|gemini|codex \
                     [--scope user|workspace] [--dry-run] [--clean] [--dst <path>] \
                     [--agents-dir <path>] [--include-agent-wrappers]"
                );
                return Err(ExitCode::SUCCESS);
            }
            arg if arg.starts_with('-') => {
                eprintln!("Error: unknown flag {arg}");
                return Err(ExitCode::from(1));
            }
            _ => {
                skills_dir = Some(args[i].clone());
            }
        }
        i += 1;
    }

    let Some(skills_dir) = skills_dir else {
        eprintln!("Error: skills directory required.");
        eprintln!(
            "Usage: install-skills <skills-dir> --provider claude|gemini|codex \
             [--scope user|workspace] [--dry-run] [--clean] [--dst <path>]"
        );
        return Err(ExitCode::from(1));
    };

    let Some(ref prov) = provider_str else {
        eprintln!("Error: --provider is required.");
        return Err(ExitCode::from(1));
    };

    let Some(provider) = Provider::from_str(prov) else {
        eprintln!("Error: invalid provider {prov:?}: use claude, gemini, or codex");
        return Err(ExitCode::from(1));
    };

    Ok(Args {
        skills_dir,
        provider,
        scope,
        dry_run,
        clean,
        dst_override,
        agents_dir,
        include_agent_wrappers,
    })
}

fn default_dst(provider: Provider) -> PathBuf {
    let home = env::var("HOME").unwrap_or_default();
    match provider {
        Provider::Claude => PathBuf::from(format!("{home}/.claude/skills")),
        Provider::Gemini => PathBuf::from(format!("{home}/.gemini/skills")),
        Provider::Codex => PathBuf::from(format!("{home}/.codex/skills")),
    }
}

fn clean_skill_dir(dst_dir: &Path) {
    if dst_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(dst_dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() {
                    let _ = std::fs::remove_dir_all(&path);
                }
            }
        }
    }
}

fn execute_action(action: &SkillInstallAction, dry_run: bool) -> Result<(), String> {
    match action {
        SkillInstallAction::Copy {
            skill_name,
            src_dir,
            dst_dir,
        } => {
            if dry_run {
                println!(
                    "[dry-run] Would install skill: {skill_name} -> {}",
                    dst_dir.display()
                );
            } else {
                skill::execute_skill_copy(src_dir, skill_name, dst_dir)?;
                println!("Installed skill: {skill_name} -> {}", dst_dir.display());
            }
        }
        SkillInstallAction::GeminiCli {
            skill_name,
            skill_dir,
            scope,
        } => {
            if dry_run {
                println!("[dry-run] Would install Gemini skill: {skill_name} (scope: {scope})");
            } else {
                println!("Installing Gemini skill: {skill_name} (scope: {scope})...");
                let status = Command::new("gemini")
                    .args([
                        "skills",
                        "install",
                        &skill_dir.to_string_lossy(),
                        "--scope",
                        scope,
                    ])
                    .status()
                    .map_err(|e| format!("failed to run gemini CLI: {e}"))?;
                if !status.success() {
                    return Err(format!(
                        "gemini skills install failed for {skill_name} (exit {})",
                        status.code().unwrap_or(-1)
                    ));
                }
            }
        }
        SkillInstallAction::Skipped { .. } => {}
    }
    Ok(())
}

fn generate_and_plan_wrappers(
    agents_dir: &Path,
    provider: Provider,
    dst_dir: &Path,
    scope: &str,
    config: &SidecarConfig,
) -> Result<(Vec<SkillInstallAction>, Option<tempfile::TempDir>), String> {
    let generated = skill::generate_skills_from_agents_dir(agents_dir)?;
    if generated.is_empty() {
        return Ok((Vec::new(), None));
    }

    let tmp_dir = tempfile::tempdir().map_err(|e| format!("failed to create temp dir: {e}"))?;

    let mut actions = Vec::new();
    for gen in &generated {
        let skill_dir = tmp_dir.path().join(&gen.agent_name);
        std::fs::create_dir_all(&skill_dir)
            .map_err(|e| format!("failed to create {}: {e}", skill_dir.display()))?;
        std::fs::write(skill_dir.join("SKILL.md"), &gen.skill_md)
            .map_err(|e| format!("failed to write SKILL.md: {e}"))?;
        std::fs::write(skill_dir.join("SKILL.yaml"), &gen.skill_yaml)
            .map_err(|e| format!("failed to write SKILL.yaml: {e}"))?;

        let meta = skill::parse_skill_yaml(&gen.skill_yaml)?;
        actions.push(skill::plan_skill_install(
            &meta, &skill_dir, provider, dst_dir, scope, config,
        ));
    }

    Ok((actions, Some(tmp_dir)))
}

fn run(args: &Args) -> ExitCode {
    let skills_path = Path::new(&args.skills_dir);
    if !skills_path.is_dir() {
        eprintln!("Error: not a directory: {}", args.skills_dir);
        return ExitCode::from(1);
    }

    let dst_dir = args
        .dst_override
        .as_ref()
        .map_or_else(|| default_dst(args.provider), PathBuf::from);

    let config = SidecarConfig::load(Path::new("."));

    if args.clean && !args.dry_run {
        clean_skill_dir(&dst_dir);
    }

    let mut actions = match skill::plan_skills_from_dir(
        skills_path,
        args.provider,
        &dst_dir,
        &args.scope,
        &config,
    ) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: {e}");
            return ExitCode::from(1);
        }
    };

    let mut _wrapper_tmpdir = None;
    if args.include_agent_wrappers && args.provider != Provider::Gemini {
        let agents_path = Path::new(&args.agents_dir);
        match generate_and_plan_wrappers(agents_path, args.provider, &dst_dir, &args.scope, &config)
        {
            Ok((extra, tmpdir)) => {
                actions.extend(extra);
                _wrapper_tmpdir = tmpdir;
            }
            Err(e) => {
                eprintln!("Error generating agent wrappers: {e}");
                return ExitCode::from(1);
            }
        }
    }

    for action in &actions {
        if let Err(e) = execute_action(action, args.dry_run) {
            eprintln!("Error: {e}");
            return ExitCode::from(1);
        }
    }

    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    match parse_args() {
        Ok(ref args) => run(args),
        Err(code) => code,
    }
}
