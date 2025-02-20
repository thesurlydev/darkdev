use clap::{Parser, Subcommand};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

const API_URL: &str = "http://127.0.0.1:3000";

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

#[derive(Debug, Serialize, Deserialize)]
struct Project {
    name: String,
    path: String,
    mode: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct InitProject {
    name: String,
    project_type: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let client = Client::new();

    match &cli.command {
        Commands::Init { name, project_type } => {
            let init_project = InitProject {
                name: name.clone(),
                project_type: project_type.clone(),
            };
            
            let response = client
                .post(format!("{}/projects/init", API_URL))
                .json(&init_project)
                .send()
                .await?
                .json::<Project>()
                .await?;
                
            println!("Initialized project: {:?}", response);
        }
        Commands::Add { name, path, mode } => {
            let project = Project {
                name: name.clone(),
                path: path.clone(),
                mode: mode.clone(),
            };
            
            let response = client
                .post(format!("{}/projects", API_URL))
                .json(&project)
                .send()
                .await?
                .json::<Project>()
                .await?;
                
            println!("Added project: {:?}", response);
        }
        Commands::Remove { name } => {
            let project = Project {
                name: name.clone(),
                path: String::new(), // Not needed for removal
                mode: String::new(),
            };
            
            client
                .post(format!("{}/projects/remove", API_URL))
                .json(&project)
                .send()
                .await?;
                
            println!("Removed project: {}", name);
        }
        Commands::List => {
            let projects = client
                .get(format!("{}/projects", API_URL))
                .send()
                .await?
                .json::<Vec<Project>>()
                .await?;
                
            if projects.is_empty() {
                println!("No projects found");
            } else {
                println!("Projects:");
                for project in projects {
                    println!("  {} ({}): {}", project.name, project.mode, project.path);
                }
            }
        }
    }

    Ok(())
}
