use affected::logger::init_logger;
use affected::reports;
use affected::ts;
use affected::workspace::Workspace;
use affected::{find_git_root, Config, OutputFormat};
use anyhow::Result;
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use log::{debug, error, info};
use std::path::PathBuf;
use std::time::Instant;

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
    Init {
        /// Overwrite the existing configuration file
        #[arg(long)]
        force: bool,
    },

    /// View affected files or projects
    #[command(subcommand)]
    View(ViewCommands),

    /// Run a specific task.
    /// Supports glob patterns to filter tasks.
    #[command(arg_required_else_help = true)]
    Run {
        /// The task to run (supports glob patterns)
        tasks: Vec<String>,
    },

    Test,
}

#[derive(Subcommand)]
enum ViewCommands {
    /// View affected files
    Files {
        /// Output format
        #[arg(long, default_value = "text")]
        format: OutputFormat,
    },
    /// View affected projects
    Projects {
        /// Output format
        #[arg(long, default_value = "text")]
        format: OutputFormat,
    },
    /// View tasks defined in the configuration.
    Tasks,
}

#[tokio::main]
async fn main() -> Result<()> {
    // load environment variables from .env file
    let _ = dotenv();

    init_logger();

    let cli = Cli::parse();

    let starting_dir = cli.repo.unwrap_or_else(|| {
        std::env::current_dir().expect("The command should be run in a git repository")
    });
    let workspace_root =
        find_git_root(&starting_dir).expect("The command should be run in a git repository");

    debug!("Using repository: {:?}", &workspace_root);

    let base = cli.base.clone().or(Some("main".to_string()));

    let config_path = workspace_root.join(".affected.yml");
    let config = if config_path.exists() {
        debug!("Config file found at {:?}", &config_path);
        Config::from_file(&config_path)?
    } else {
        debug!("Config file not found, using a default one");
        Config {
            base: base.clone(),
            ..Default::default()
        }
    };

    let mut workspace = Workspace::with_config(&workspace_root, config);

    match &cli.command {
        Commands::Init { force } => {
            if config_path.exists() && !force {
                error!("Config file already exists. Remove it to reinitialize, or use --force to overwrite.");
                return Ok(());
            }
            let config = Config {
                base: base.clone(),
                ..Default::default()
            };
            config.to_file(&config_path)?;
            println!("Config file created at {:?}", &config_path);
        }

        Commands::View(subcommand) => match subcommand {
            ViewCommands::Files { format } => {
                if let Err(err) = workspace.load().await {
                    log::error!("Failed to load workspace: {}", err);
                    return Ok(());
                }

                reports::display_affected_files(&workspace, format)?;
            }
            ViewCommands::Projects { format } => {
                workspace.load().await?;
                reports::display_affected_projects(&workspace, format)?;

                // let projects = workspace.affected_projects()?;
                //
                // if projects.is_empty() {
                //     println!("No projects affected");
                //     return Ok(());
                // }
                //
                // print_lines(&projects, format, "Path")?;
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
        Commands::Run { tasks } => {
            workspace.load().await?;

            let now = Instant::now();

            // TODO: support running tasks in parallel (with extra `--parallel` flag)
            for task in tasks {
                match workspace.run_task(task).await {
                    Ok(_) => debug!("Task '{}' completed successfully", task),
                    Err(err) => {
                        error!("Failed to run task '{}': {}", task, err);
                        std::process::exit(1);
                    }
                }
            }

            let elapsed = now.elapsed();
            info!("Done ({:.2?})", elapsed);
        }
        Commands::Test => {
            if let Err(err) = workspace.load().await {
                log::error!("Failed to load workspace: {}", err);
                return Ok(());
            }

            let files = workspace.affected_files()?;
            if files.is_empty() {
                println!("No files affected");
                return Ok(());
            }

            let first_file = files.iter().next().unwrap();
            println!("First file: {}", first_file);

            ts::execute_common_js();
            ts::resolver::demo();
        }
    }

    Ok(())
}
