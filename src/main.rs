use affected::logger::init_logger;
use affected::tasks;
use affected::{get_affected_files, get_project, list_affected_projects, Config};
use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use git2::Repository;
use log::debug;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "affected")]
#[command(about = "A tool to find affected files or projects in a git repository", long_about = None)]
struct Cli {
    /// Optional repo path, defaults to current directory
    #[arg(long)]
    repo: Option<PathBuf>,

    /// Base of the current branch (usually main). Falls back to 'main' or 'master' if not provided.
    #[arg(long)]
    base: Option<String>,

    /// The subcommand to run
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize the configuration file
    Init,

    /// View affected files or projects
    #[command(subcommand)]
    View(ViewCommands),

    /// Run a specific task
    #[command(arg_required_else_help = true)]
    Run {
        /// The task to run
        task: String,
    },
}

#[derive(Subcommand)]
enum ViewCommands {
    Files,
    Projects,
    Tasks,
}

fn main() -> Result<()> {
    // load environment variables from .env file
    let _ = dotenv();

    init_logger();

    let cli = Cli::parse();

    let workspace_root = cli
        .repo
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get the repository path"));
    debug!("Using repository: {:?}", &workspace_root);

    // let config = Config::from_env();

    let config_path = workspace_root.join(".affected.yml");
    let config = if config_path.exists() {
        debug!("Config file found at {:?}", &config_path);
        Config::from_file(&config_path)?
    } else {
        debug!("Config file not found, using a default one");
        Config {
            base: cli.base.clone().or(Some("main".to_string())),
            ..Default::default()
        }
    };

    let repo = Repository::open(&workspace_root).expect("Could not open the repository");

    // TODO: introduce flag to fetch from remote
    // Fetch the latest changes from the remote repository
    // let mut remote = repo
    //     .find_remote("origin")
    //     .context("Could not find remote 'origin'")?;
    // remote
    //     .fetch(&["refs/heads/*:refs/remotes/origin/*"], None, None)
    //     .context("Failed to fetch from remote repository")?;

    match &cli.command {
        Commands::Init => {
            config.to_file(&config_path)?;
            println!("Config file created at {:?}", &config_path);
        }

        Commands::View(subcommand) => match subcommand {
            ViewCommands::Files => {
                let files = get_affected_files(&repo, &config)?;
                for file in files {
                    println!("{}", file);
                }
            }
            ViewCommands::Projects => {
                let project_paths = list_affected_projects(&workspace_root, &repo, &config)?;
                for project_path in project_paths {
                    let project = get_project(&workspace_root, &project_path)?;
                    let name = match project.name() {
                        Some(name) => name,
                        None => bail!("Project name is not defined"),
                    };
                    println!("{}", name);
                }
            }
            ViewCommands::Tasks => {
                if let Some(tasks) = &config.tasks {
                    for task in tasks {
                        println!("{}", task);
                    }
                } else {
                    println!("No tasks defined");
                }
            }
        },
        Commands::Run { task } => {
            tasks::run_task_by_name(&workspace_root, &repo, &config, task)?;
            println!("Done");
        }
    }

    Ok(())
}
