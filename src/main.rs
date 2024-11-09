use affected::{list_all_targets, list_projects};
use anyhow::Result;
use clap::{Parser, Subcommand};
use env_logger::{Builder, Env};
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

    /// Optional main branch name, defaults to "main"
    #[arg(long, default_value = "main")]
    main: Option<String>,

    /// The subcommand to run
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(subcommand)]
    List(ListTargets),
}

#[derive(Subcommand)]
enum ListTargets {
    All,
    Projects,
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

    let workspace_dir = cli
        .repo
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));
    debug!("Using working directory: {:?}", workspace_dir);

    let main = cli.main.unwrap();
    debug!("Using main branch: {}", main.clone());

    match &cli.command {
        Commands::List(subcommand) => match subcommand {
            ListTargets::All => {
                list_all_targets(workspace_dir, main.clone())?;
            }
            ListTargets::Projects => {
                list_projects()?;
            }
        },
    }

    Ok(())
}
