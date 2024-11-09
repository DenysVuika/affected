use affected::{list_all_targets, list_projects};
use anyhow::Result;
use clap::{Parser, Subcommand};
use env_logger::{Builder, Env};
use git2::Repository;
use log::debug;
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "affected")]
#[command(about = "A tool to find affected files or projects in a git repository", long_about = None)]
struct Cli {
    /// Optional repo path, defaults to current directory
    #[arg(long)]
    repo: Option<PathBuf>,

    /// Optional main branch name, evaluates to 'main' or 'master' if not provided
    #[arg(long)]
    main: Option<String>,

    /// The subcommand to run
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(subcommand)]
    Files(FilesCommands),

    #[command(subcommand)]
    Projects(ProjectsCommands),
}

#[derive(Subcommand)]
enum FilesCommands {
    List,
}

#[derive(Subcommand)]
enum ProjectsCommands {
    List,
}

fn main() -> Result<()> {
    let env = Env::default()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "always");

    // env_logger::init_from_env(env);

    Builder::from_env(env)
        .format(|buf, record| {
            let level = record.level();
            let info_style = buf.default_level_style(record.level());
            // let timestamp = buf.timestamp();
            // writeln!(buf, "{level}: {info_style}{}{info_style:#}", record.args())
            writeln!(buf, "{info_style}{level}: {info_style:#}{}", record.args())
        })
        .init();

    let cli = Cli::parse();

    let workspace_root = cli
        .repo
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get the repository path"));
    debug!("Using repository: {:?}", workspace_root);

    let repo = Repository::open(workspace_root).expect("Could not open the repository");

    match &cli.command {
        Commands::Files(subcommand) => match subcommand {
            FilesCommands::List => {
                list_all_targets(&repo, cli.main)?;
            }
        },
        Commands::Projects(subcommand) => match subcommand {
            ProjectsCommands::List => {
                list_projects(&repo, cli.main)?;
            }
        },
    }

    Ok(())
}
