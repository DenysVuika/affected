use affected::logger::init_logger;
use affected::workspace::Workspace;
use affected::Config;
use anyhow::Result;
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use log::debug;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "affected")]
#[command(about = "A tool to find affected files or projects in a git repository and run commands on them.", long_about = None)]
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

    let mut workspace = Workspace::with_config(&workspace_root, config);

    match &cli.command {
        Commands::Init => {
            let config = workspace.config().expect("No configuration found");
            config.to_file(&config_path)?;
            println!("Config file created at {:?}", &config_path);
        }

        Commands::View(subcommand) => match subcommand {
            ViewCommands::Files => {
                workspace.load().await?;

                let files = workspace.affected_files()?;
                for file in files {
                    println!("{}", file);
                }
            }
            ViewCommands::Projects => {
                workspace.load().await?;

                let projects = workspace.affected_projects()?;

                if projects.is_empty() {
                    println!("No projects affected");
                    return Ok(());
                }

                for project in projects {
                    println!("{}", project);
                }

                /*
                   let graph = affected::graph::build_graph(&workspace)?;

                   if graph.node_count() == 0 {
                       println!("No projects affected");
                       return Ok(());
                   }

                   let mut printed_nodes: HashSet<String> = HashSet::new();

                   for node_index in graph.node_indices() {
                       let node = &graph[node_index];

                       match node {
                           NodeType::Project(project_node) => {
                               printed_nodes.insert(project_node.name.clone());
                               debug!("{}", project_node.name);
                           }
                           _ => {}
                       }
                   }

                   for edge in graph.edge_indices() {
                       let (source, target) = graph.edge_endpoints(edge).unwrap();
                       let source_node = &graph[source];
                       let target_node = &graph[target];
                       if let (NodeType::Project(source_project), NodeType::Project(target_project)) =
                           (source_node, target_node)
                       {
                           debug!(
                               "{} -> (implicit) -> {}",
                               &source_project.name, &target_project.name
                           );
                           printed_nodes.insert(target_project.name.clone());
                       }
                   }

                   for node in printed_nodes {
                       println!("{}", node);
                   }
                */

                // println!("{:?}", graph);
            }
            ViewCommands::Tasks => {
                let tasks = workspace.tasks();
                if tasks.is_empty() {
                    println!("No tasks defined");
                    return Ok(());
                }

                for task in tasks {
                    println!("{}", task);
                }
            }
        },
        Commands::Run { task } => {
            workspace.load().await?;
            match workspace.run_task(task).await {
                Ok(_) => println!("Done"),
                Err(err) => log::error!("Failed to run task: {}", err),
            }
        }
    }

    Ok(())
}
