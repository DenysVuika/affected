use affected::{
    get_project, list_affected_files, list_affected_projects, list_all_projects, Config,
};
use anyhow::{bail, Result};
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

    /// Base of the current branch (usually main). Falls back to 'main' or 'master' if not provided.
    #[arg(long)]
    base: Option<String>,

    /// The subcommand to run
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,

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
    All,
    List,
}

fn main() -> Result<()> {
    let env = Env::default()
        .filter_or("LOG_LEVEL", "error")
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
    debug!("Using repository: {:?}", &workspace_root);

    let config_path = workspace_root.join("affected.yml");
    if config_path.exists() {
        let config = Config::from_file(&config_path)?;
        debug!("Using config file: {:?}", &config_path);
        println!("{:?}", config);
    }

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
            // let config = Config::from_env();
            let config = Config {
                base: cli.base.clone().unwrap_or_else(|| "main".to_string()),
            };
            config.to_file(&config_path)?;
            println!("Config file created at {:?}", &config_path);
        }
        Commands::Files(subcommand) => match subcommand {
            FilesCommands::List => {
                let files = list_affected_files(&repo, cli.base)?;
                for file in files {
                    println!("{}", file);
                }
            }
        },
        Commands::Projects(subcommand) => match subcommand {
            ProjectsCommands::All => {
                list_all_projects(&workspace_root, &repo, cli.base)?;
            }
            ProjectsCommands::List => {
                let project_paths = list_affected_projects(&workspace_root, &repo, cli.base)?;
                for project_path in project_paths {
                    let project = get_project(&workspace_root, &project_path)?;
                    let name = match project.name() {
                        Some(name) => name,
                        None => bail!("Project name is not defined"),
                    };
                    println!("{}", name);
                }
            }
        },
    }

    Ok(())
}
