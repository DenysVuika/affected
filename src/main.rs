use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "affected")]
#[command(about = "A tool to find affected files or projects in a git repository", long_about = None)]
struct Cli {
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

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::List(subcommand) => match subcommand {
            ListTargets::All => {
                println!("All targets");
            }
            ListTargets::Projects => {
                println!("Projects");
            }
        },
    }
}
