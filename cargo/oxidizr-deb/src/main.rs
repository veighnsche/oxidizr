use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand, ValueEnum};
use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::safepath::SafePath;
use switchyard::types::{ApplyMode, LinkRequest, PlanInput, RestoreRequest};
use switchyard::adapters::{DefaultSmokeRunner, FileLockManager, FsOwnershipOracle};
use switchyard::Switchyard;

#[derive(Debug, Copy, Clone, Eq, PartialEq, ValueEnum)]
enum Preset {
    Coreutils,
    Sudo,
}

#[derive(Debug, Parser)]
#[command(name = "oxidizr-deb", version, about = "Debian-family CLI to swap GNU coreutils with uutils-coreutils and sudo with sudo-rs using Switchyard")] 
struct Cli {
    /// Root of the filesystem tree to operate on (default "/")
    #[arg(long, global = true, default_value = "/")]
    root: PathBuf,

    /// Commit changes to disk (default is dry-run)
    #[arg(long, global = true, default_value_t = false)]
    commit: bool,

    /// Select a high-level preset for policy configuration
    #[arg(long, value_enum, global = true)]
    preset: Option<Preset>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Link a single target to a source (backed up and TOCTOU-safe)
    Link(LinkCmd),
    /// Restore one or more targets from backup sidecars
    Restore(RestoreCmd),
    /// Enable a profile of replacements (e.g., coreutils, sudo)
    Enable(EnableCmd),
    /// Disable a profile (restore originals)
    Disable(DisableCmd),
}

#[derive(Debug, Args)]
struct LinkCmd {
    /// Source binary path (replacement)
    #[arg(long)]
    src: PathBuf,
    /// Target path to replace (e.g., /usr/bin/ls)
    #[arg(long)]
    dst: PathBuf,
}

#[derive(Debug, Args)]
struct RestoreCmd {
    /// One or more target paths to restore
    #[arg(required = true)]
    targets: Vec<PathBuf>,
}

#[derive(Debug, Args)]
struct EnableCmd {
    /// Which profile to enable (coreutils|sudo)
    #[arg(long, value_enum)]
    profile: Preset,
    /// Path to the unified replacement binary (for coreutils), or sudo-rs binary for sudo
    #[arg(long)]
    source_bin: Option<PathBuf>,
    /// Destination directory for applets (default /usr/bin) used by coreutils profile
    #[arg(long)]
    dest_dir: Option<PathBuf>,
    /// Applets to link for coreutils; defaults to a conservative subset if omitted
    #[arg(long, value_delimiter = ',')]
    applets: Option<Vec<String>>,
}

#[derive(Debug, Args)]
struct DisableCmd {
    /// Which profile to disable (coreutils|sudo)
    #[arg(long, value_enum)]
    profile: Preset,
    /// Destination directory for applets (default /usr/bin) used by coreutils profile
    #[arg(long)]
    dest_dir: Option<PathBuf>,
    /// Applets to restore for coreutils; defaults to a conservative subset if omitted
    #[arg(long, value_delimiter = ',')]
    applets: Option<Vec<String>>,
}

fn main() {
    let cli = Cli::parse();

    // Build policy based on preset and defaults
    let mut policy = match cli.preset.unwrap_or(Preset::Coreutils) {
        Preset::Coreutils => Policy::coreutils_switch_preset(),
        Preset::Sudo => Policy::production_preset(),
    };

    // Narrow scope to requested root; callers may further narrow via dest_dir
    // Note: SafePath enforces absolute root.
    policy.scope.allow_roots.push(cli.root.clone());

    let lock_path = cli.root.join("var/lock/oxidizr-deb.lock");
    let api: Switchyard<JsonlSink, JsonlSink> = Switchyard::builder(
        JsonlSink::default(),
        JsonlSink::default(),
        policy,
    )
    .with_lock_manager(Box::new(FileLockManager::new(lock_path)))
    .with_smoke_runner(Box::new(DefaultSmokeRunner::default()))
    .with_ownership_oracle(Box::new(FsOwnershipOracle::default()))
    .build();

    let apply_mode = if cli.commit { ApplyMode::Commit } else { ApplyMode::DryRun };

    let result = match cli.command {
        Commands::Link(args) => run_link(&api, &cli.root, &args, apply_mode),
        Commands::Restore(args) => run_restore(&api, &cli.root, &args, apply_mode),
        Commands::Enable(args) => run_enable(&api, &cli.root, &args, apply_mode),
        Commands::Disable(args) => run_disable(&api, &cli.root, &args, apply_mode),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run_link(api: &Switchyard<JsonlSink, JsonlSink>, root: &Path, args: &LinkCmd, mode: ApplyMode) -> Result<(), String> {
    let src = SafePath::from_rooted(root, &args.src).map_err(|e| format!("invalid src: {e:?}"))?;
    let dst = SafePath::from_rooted(root, &args.dst).map_err(|e| format!("invalid dst: {e:?}"))?;

    let input = PlanInput {
        link: vec![LinkRequest { source: src, target: dst }],
        restore: vec![],
    };
    let plan = api.plan(input);
    // Optionally preflight
    let _pre = api.preflight(&plan).map_err(|e| format!("preflight failed: {e:?}"))?;
    let rep = api.apply(&plan, mode).map_err(|e| format!("apply failed: {e:?}"))?;
    if matches!(mode, ApplyMode::DryRun) {
        eprintln!("dry-run: planned {} actions", rep.executed.len());
    }
    Ok(())
}

fn run_restore(api: &Switchyard<JsonlSink, JsonlSink>, root: &Path, args: &RestoreCmd, mode: ApplyMode) -> Result<(), String> {
    let mut restores = Vec::new();
    for t in &args.targets {
        let sp = SafePath::from_rooted(root, t).map_err(|e| format!("invalid target: {e:?}"))?;
        restores.push(RestoreRequest { target: sp });
    }
    let plan = api.plan(PlanInput { link: vec![], restore: restores });
    let _pre = api.preflight(&plan).map_err(|e| format!("preflight failed: {e:?}"))?;
    let rep = api.apply(&plan, mode).map_err(|e| format!("apply failed: {e:?}"))?;
    if matches!(mode, ApplyMode::DryRun) {
        eprintln!("dry-run: planned {} actions", rep.executed.len());
    }
    Ok(())
}

fn run_enable(api: &Switchyard<JsonlSink, JsonlSink>, root: &Path, args: &EnableCmd, mode: ApplyMode) -> Result<(), String> {
    match args.profile {
        Preset::Coreutils => {
            let source_bin = args
                .source_bin
                .clone()
                .unwrap_or_else(|| PathBuf::from("/usr/bin/uutils"));
            let dest_dir = args.dest_dir.clone().unwrap_or_else(|| PathBuf::from("/usr/bin"));
            let applets = args.applets.clone().unwrap_or_else(default_coreutils_applets);
            enable_coreutils(api, root, &source_bin, &dest_dir, &applets, mode)
        }
        Preset::Sudo => {
            let source_bin = args
                .source_bin
                .clone()
                .unwrap_or_else(|| PathBuf::from("/usr/bin/sudo-rs"));
            let dest_dir = args.dest_dir.clone().unwrap_or_else(|| PathBuf::from("/usr/bin"));
            enable_sudo(api, root, &source_bin, &dest_dir, mode)
        }
    }
}

fn run_disable(api: &Switchyard<JsonlSink, JsonlSink>, root: &Path, args: &DisableCmd, mode: ApplyMode) -> Result<(), String> {
    match args.profile {
        Preset::Coreutils => {
            let dest_dir = args.dest_dir.clone().unwrap_or_else(|| PathBuf::from("/usr/bin"));
            let applets = args.applets.clone().unwrap_or_else(default_coreutils_applets);
            restore_many(api, root, &dest_dir, &applets, mode)
        }
        Preset::Sudo => {
            let dest_dir = args.dest_dir.clone().unwrap_or_else(|| PathBuf::from("/usr/bin"));
            restore_many(api, root, &dest_dir, &vec!["sudo".to_string()], mode)
        }
    }
}

fn enable_coreutils(
    api: &Switchyard<JsonlSink, JsonlSink>,
    root: &Path,
    source_bin: &Path,
    dest_dir: &Path,
    applets: &[String],
    mode: ApplyMode,
) -> Result<(), String> {
    let mut links = Vec::new();
    for app in applets {
        let dst = dest_dir.join(app);
        let src = source_bin;
        let s_sp = SafePath::from_rooted(root, src).map_err(|e| format!("invalid source_bin: {e:?}"))?;
        let d_sp = SafePath::from_rooted(root, &dst).map_err(|e| format!("invalid dest: {e:?}"))?;
        links.push(LinkRequest { source: s_sp.clone(), target: d_sp });
    }
    let plan = api.plan(PlanInput { link: links, restore: vec![] });
    let _pre = api.preflight(&plan).map_err(|e| format!("preflight failed: {e:?}"))?;
    let rep = api.apply(&plan, mode).map_err(|e| format!("apply failed: {e:?}"))?;
    if matches!(mode, ApplyMode::DryRun) {
        eprintln!("dry-run: planned {} actions", rep.executed.len());
    }
    Ok(())
}

fn enable_sudo(
    api: &Switchyard<JsonlSink, JsonlSink>,
    root: &Path,
    source_bin: &Path,
    dest_dir: &Path,
    mode: ApplyMode,
) -> Result<(), String> {
    let dst = dest_dir.join("sudo");
    let s_sp = SafePath::from_rooted(root, source_bin).map_err(|e| format!("invalid source_bin: {e:?}"))?;
    let d_sp = SafePath::from_rooted(root, &dst).map_err(|e| format!("invalid dest: {e:?}"))?;
    let plan = api.plan(PlanInput { link: vec![LinkRequest { source: s_sp, target: d_sp }], restore: vec![] });
    let _pre = api.preflight(&plan).map_err(|e| format!("preflight failed: {e:?}"))?;
    let rep = api.apply(&plan, mode).map_err(|e| format!("apply failed: {e:?}"))?;
    if matches!(mode, ApplyMode::DryRun) {
        eprintln!("dry-run: planned {} actions", rep.executed.len());
    }
    Ok(())
}

fn restore_many(
    api: &Switchyard<JsonlSink, JsonlSink>,
    root: &Path,
    dest_dir: &Path,
    applets: &[String],
    mode: ApplyMode,
) -> Result<(), String> {
    let mut restores = Vec::new();
    for app in applets {
        let dst = dest_dir.join(app);
        let sp = SafePath::from_rooted(root, &dst).map_err(|e| format!("invalid target: {e:?}"))?;
        restores.push(RestoreRequest { target: sp });
    }
    let plan = api.plan(PlanInput { link: vec![], restore: restores });
    let _pre = api.preflight(&plan).map_err(|e| format!("preflight failed: {e:?}"))?;
    let rep = api.apply(&plan, mode).map_err(|e| format!("apply failed: {e:?}"))?;
    if matches!(mode, ApplyMode::DryRun) {
        eprintln!("dry-run: planned {} actions", rep.executed.len());
    }
    Ok(())
}

fn default_coreutils_applets() -> Vec<String> {
    // Conservative subset; users can override via --applets
    [
        "ls", "cp", "mv", "rm", "cat", "echo", "touch", "mkdir", "rmdir", "chmod", "chown", "ln",
        "head", "tail", "sort", "uniq", "wc", "basename", "dirname", "date",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}
