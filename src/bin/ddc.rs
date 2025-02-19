use clap::{Parser, Subcommand};
use std::error::Error;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new project
    Init {
        /// Name of the project
        #[arg(short, long)]
        name: String,
        
        /// Project type (java, rust, etc.)
        #[arg(short, long)]
        project_type: String,
    },
    
    /// Add a new project to watch-config.toml
    Add {
        /// Name of the project
        #[arg(short, long)]
        name: String,
        
        /// Path to the project directory
        #[arg(short, long)]
        path: String,
        
        /// Project mode (compile, test, run, etc.)
        #[arg(short, long, default_value = "compile")]
        mode: String,
    },
    
    /// Remove a project from watch-config.toml
    Remove {
        /// Name of the project
        #[arg(short, long)]
        name: String,
    },
    
    /// List all projects in watch-config.toml
    List,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { name, project_type } => {
            println!("Initializing new {} project: {}", project_type, name);
            // TODO: Implement project initialization
        }
        Commands::Add { name, path, mode } => {
            println!("Adding project {} at {} with mode {}", name, path, mode);
            // TODO: Implement project addition to watch-config.toml
        }
        Commands::Remove { name } => {
            println!("Removing project {}", name);
            // TODO: Implement project removal from watch-config.toml
        }
        Commands::List => {
            println!("Listing all projects");
            // TODO: Implement project listing
        }
    }

    Ok(())
}
