use affected::logger::init_logger;
use affected::tasks;
use affected::workspace::Workspace;
use affected::{get_affected_files, get_affected_projects, Config};
use anyhow::Result;
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use git2::Repository;
use log::debug;
use std::collections::HashSet;
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

#[tokio::main]
async fn main() -> Result<()> {
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

    let workspace = Workspace::with_config(&workspace_root, config);
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
            let config = workspace.config().expect("No configuration found");
            config.to_file(&config_path)?;
            println!("Config file created at {:?}", &config_path);
        }

        Commands::View(subcommand) => match subcommand {
            ViewCommands::Files => {
                if let Some(config) = workspace.config() {
                    let files = get_affected_files(&repo, config)?;
                    for file in files {
                        println!("{}", file);
                    }
                }
            }
            ViewCommands::Projects => {
                let config = workspace.config().expect("No configuration found");
                let project_paths = get_affected_projects(&workspace_root, &repo, config)?;
                if project_paths.is_empty() {
                    println!("No projects affected");
                    return Ok(());
                }

                let graph = affected::graph::build_graph(&workspace_root, &project_paths)?;

                if graph.node_count() == 0 {
                    println!("No projects affected");
                    return Ok(());
                }

                let mut printed_nodes = HashSet::new();

                for node_index in graph.node_indices() {
                    let project_name = &graph[node_index];
                    printed_nodes.insert(project_name);
                    debug!("{}", project_name);
                }

                for edge in graph.edge_indices() {
                    let (source, target) = graph.edge_endpoints(edge).unwrap();
                    let source_name = &graph[source];
                    let target_name = &graph[target];
                    debug!("{} -> (implicit) -> {}", source_name, target_name);
                    printed_nodes.insert(target_name);
                }

                for node in printed_nodes {
                    println!("{}", node);
                }

                // println!("{:?}", graph);
            }
            ViewCommands::Tasks => {
                if let Some(config) = workspace.config() {
                    if let Some(tasks) = &config.tasks {
                        for task in tasks {
                            println!("{}", task);
                        }
                    } else {
                        println!("No tasks defined");
                    }
                }
            }
        },
        Commands::Run { task } => {
            if let Some(config) = workspace.config() {
                match tasks::run_task_by_name(&workspace_root, &repo, config, task).await {
                    Ok(_) => println!("Done"),
                    Err(err) => log::error!("Failed to run task: {}", err),
                }
            } else {
                log::error!("No configuration found");
            }
        }
    }

    Ok(())
}
