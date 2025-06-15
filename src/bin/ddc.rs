use clap::{Parser, Subcommand};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use dialoguer::{Input, Select, Confirm, MultiSelect, theme::ColorfulTheme};
use console::style;
use toml;

const API_URL: &str = "http://127.0.0.1:3000";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Interactive setup wizard for DarkDev configuration
    Setup {
        /// Skip interactive prompts and use defaults
        #[arg(long)]
        non_interactive: bool,

        /// Output directory for configuration (default: current directory)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

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

    /// Validate watch-config.toml configuration
    Validate {
        /// Path to configuration file (default: watch-config.toml)
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Show detailed validation information
        #[arg(short, long)]
        verbose: bool,
    },
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

#[derive(Debug, Serialize, Deserialize)]
struct WatchConfig {
    global: Option<GlobalConfig>,
    projects: Option<HashMap<String, ProjectConfig>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GlobalConfig {
    channel_capacity: Option<u32>,
    default_modes: Option<Vec<String>>,
    debounce_delay: Option<u64>,
    logging_level: Option<String>,
    watch_paths: Option<Vec<String>>,
    watch_extensions: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectConfig {
    project_dir: String,
    watch_paths: Option<Vec<String>>,
    watch_extensions: Option<Vec<String>>,
    dependencies: Option<Vec<String>>,
    modes: Option<Vec<String>>,
    commands: Option<HashMap<String, CommandConfig>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CommandConfig {
    program: String,
    args: Vec<String>,
    run_on_startup: Option<bool>,
}

#[derive(Debug, Clone)]
struct ProjectTemplate {
    name: String,
    description: String,
    watch_paths: Vec<String>,
    watch_extensions: Vec<String>,
    default_modes: Vec<String>,
    commands: std::collections::HashMap<String, CommandTemplate>,
}

#[derive(Debug, Clone)]
struct CommandTemplate {
    program: String,
    args: Vec<String>,
    run_on_startup: bool,
}

#[derive(Debug, Clone)]
struct DetectedProject {
    name: String,
    path: PathBuf,
    project_type: String,
}

#[derive(Debug)]
struct ValidationError {
    field: String,
    message: String,
    severity: ValidationSeverity,
}

#[derive(Debug, PartialEq)]
enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug)]
struct ValidationResult {
    errors: Vec<ValidationError>,
    warnings: Vec<ValidationError>,
    info: Vec<ValidationError>,
}

impl ValidationResult {
    fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
        }
    }

    fn add_error(&mut self, field: String, message: String) {
        self.errors.push(ValidationError {
            field,
            message,
            severity: ValidationSeverity::Error,
        });
    }

    fn add_warning(&mut self, field: String, message: String) {
        self.warnings.push(ValidationError {
            field,
            message,
            severity: ValidationSeverity::Warning,
        });
    }

    fn add_info(&mut self, field: String, message: String) {
        self.info.push(ValidationError {
            field,
            message,
            severity: ValidationSeverity::Info,
        });
    }

    fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

fn get_project_templates() -> Vec<ProjectTemplate> {
    vec![
        ProjectTemplate {
            name: "java-maven".to_string(),
            description: "Java project with Maven build system".to_string(),
            watch_paths: vec!["src/main/java".to_string(), "src/test/java".to_string(), "pom.xml".to_string()],
            watch_extensions: vec!["java".to_string(), "xml".to_string()],
            default_modes: vec!["compile".to_string()],
            commands: {
                let mut commands = std::collections::HashMap::new();
                commands.insert("compile".to_string(), CommandTemplate {
                    program: "mvn".to_string(),
                    args: vec!["-q".to_string(), "compile".to_string()],
                    run_on_startup: false,
                });
                commands.insert("test".to_string(), CommandTemplate {
                    program: "mvn".to_string(),
                    args: vec!["-q".to_string(), "test".to_string()],
                    run_on_startup: false,
                });
                commands.insert("run".to_string(), CommandTemplate {
                    program: "mvn".to_string(),
                    args: vec!["-q".to_string(), "compile".to_string(), "exec:java".to_string()],
                    run_on_startup: false,
                });
                commands
            },
        },
        ProjectTemplate {
            name: "rust-cargo".to_string(),
            description: "Rust project with Cargo build system".to_string(),
            watch_paths: vec!["src".to_string(), "Cargo.toml".to_string()],
            watch_extensions: vec!["rs".to_string(), "toml".to_string()],
            default_modes: vec!["check".to_string()],
            commands: {
                let mut commands = std::collections::HashMap::new();
                commands.insert("check".to_string(), CommandTemplate {
                    program: "cargo".to_string(),
                    args: vec!["check".to_string()],
                    run_on_startup: false,
                });
                commands.insert("build".to_string(), CommandTemplate {
                    program: "cargo".to_string(),
                    args: vec!["build".to_string()],
                    run_on_startup: false,
                });
                commands.insert("test".to_string(), CommandTemplate {
                    program: "cargo".to_string(),
                    args: vec!["test".to_string()],
                    run_on_startup: false,
                });
                commands.insert("run".to_string(), CommandTemplate {
                    program: "cargo".to_string(),
                    args: vec!["run".to_string()],
                    run_on_startup: false,
                });
                commands
            },
        },
        ProjectTemplate {
            name: "node-npm".to_string(),
            description: "Node.js project with npm".to_string(),
            watch_paths: vec!["src".to_string(), "package.json".to_string()],
            watch_extensions: vec!["js".to_string(), "ts".to_string(), "json".to_string()],
            default_modes: vec!["dev".to_string()],
            commands: {
                let mut commands = std::collections::HashMap::new();
                commands.insert("dev".to_string(), CommandTemplate {
                    program: "npm".to_string(),
                    args: vec!["run".to_string(), "dev".to_string()],
                    run_on_startup: false,
                });
                commands.insert("build".to_string(), CommandTemplate {
                    program: "npm".to_string(),
                    args: vec!["run".to_string(), "build".to_string()],
                    run_on_startup: false,
                });
                commands.insert("test".to_string(), CommandTemplate {
                    program: "npm".to_string(),
                    args: vec!["test".to_string()],
                    run_on_startup: false,
                });
                commands
            },
        },
        ProjectTemplate {
            name: "python".to_string(),
            description: "Python project".to_string(),
            watch_paths: vec!["src".to_string(), "*.py".to_string(), "requirements.txt".to_string()],
            watch_extensions: vec!["py".to_string(), "txt".to_string()],
            default_modes: vec!["run".to_string()],
            commands: {
                let mut commands = std::collections::HashMap::new();
                commands.insert("run".to_string(), CommandTemplate {
                    program: "python".to_string(),
                    args: vec!["main.py".to_string()],
                    run_on_startup: false,
                });
                commands.insert("test".to_string(), CommandTemplate {
                    program: "python".to_string(),
                    args: vec!["-m".to_string(), "pytest".to_string()],
                    run_on_startup: false,
                });
                commands
            },
        },
    ]
}

fn detect_projects(base_path: &Path) -> Result<Vec<DetectedProject>, Box<dyn Error>> {
    let mut detected = Vec::new();

    // Check current directory
    if let Some(project) = detect_single_project(base_path)? {
        detected.push(project);
    }

    // Check subdirectories
    if let Ok(entries) = fs::read_dir(base_path) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                if let Some(project) = detect_single_project(&entry.path())? {
                    detected.push(project);
                }
            }
        }
    }

    Ok(detected)
}

fn detect_single_project(path: &Path) -> Result<Option<DetectedProject>, Box<dyn Error>> {
    let name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Java Maven project
    let pom_xml = path.join("pom.xml");
    if pom_xml.exists() {
        return Ok(Some(DetectedProject {
            name,
            path: path.to_path_buf(),
            project_type: "java-maven".to_string(),
        }));
    }

    // Rust Cargo project
    let cargo_toml = path.join("Cargo.toml");
    if cargo_toml.exists() {
        return Ok(Some(DetectedProject {
            name,
            path: path.to_path_buf(),
            project_type: "rust-cargo".to_string(),
        }));
    }

    // Node.js project
    let package_json = path.join("package.json");
    if package_json.exists() {
        return Ok(Some(DetectedProject {
            name,
            path: path.to_path_buf(),
            project_type: "node-npm".to_string(),
        }));
    }

    // Python project
    let setup_py = path.join("setup.py");
    let pyproject_toml = path.join("pyproject.toml");
    let requirements_txt = path.join("requirements.txt");
    if setup_py.exists() || pyproject_toml.exists() || requirements_txt.exists() {
        return Ok(Some(DetectedProject {
            name,
            path: path.to_path_buf(),
            project_type: "python".to_string(),
        }));
    }

    Ok(None)
}

fn run_interactive_setup(output_dir: &Path) -> Result<(), Box<dyn Error>> {
    let theme = ColorfulTheme::default();

    // Welcome message
    println!("{}", style("🚀 Welcome to DarkDev Interactive Setup!").bold().cyan());
    println!("{}", style("This wizard will help you configure DarkDev for your development environment.\n").dim());

    // Detect existing projects
    println!("{}", style("🔍 Scanning for projects...").yellow());
    let detected_projects = detect_projects(output_dir)?;

    if !detected_projects.is_empty() {
        println!("{}", style(format!("Found {} project(s):", detected_projects.len())).green());
        for project in &detected_projects {
            println!("  {} {} ({})",
                style("•").green(),
                style(&project.name).bold(),
                style(&project.project_type).dim()
            );
        }
        println!();
    }

    // Ask which projects to include
    let selected_projects = if !detected_projects.is_empty() {
        let project_names: Vec<String> = detected_projects.iter()
            .map(|p| format!("{} ({})", p.name, p.project_type))
            .collect();

        let selected_indices = MultiSelect::with_theme(&theme)
            .with_prompt("Select projects to include in DarkDev configuration:")
            .items(&project_names)
            .defaults(&vec![true; project_names.len()])
            .interact()?;

        selected_indices.into_iter()
            .map(|i| detected_projects[i].clone())
            .collect()
    } else {
        println!("{}", style("No projects detected automatically.").yellow());

        if Confirm::with_theme(&theme)
            .with_prompt("Would you like to manually add a project?")
            .default(true)
            .interact()? {

            let name: String = Input::with_theme(&theme)
                .with_prompt("Project name")
                .interact()?;

            let path: String = Input::with_theme(&theme)
                .with_prompt("Project path (relative to current directory)")
                .default(".".to_string())
                .interact()?;

            let templates = get_project_templates();
            let template_names: Vec<String> = templates.iter()
                .map(|t| format!("{} - {}", t.name, t.description))
                .collect();

            let template_index = Select::with_theme(&theme)
                .with_prompt("Select project type")
                .items(&template_names)
                .default(0)
                .interact()?;

            vec![DetectedProject {
                name,
                path: PathBuf::from(path),
                project_type: templates[template_index].name.clone(),
            }]
        } else {
            vec![]
        }
    };

    if selected_projects.is_empty() {
        println!("{}", style("No projects selected. Exiting setup.").yellow());
        return Ok(());
    }

    // Global configuration
    println!("\n{}", style("⚙️  Global Configuration").bold().cyan());

    let debounce_delay: u64 = Input::with_theme(&theme)
        .with_prompt("Debounce delay (milliseconds)")
        .default(1000)
        .interact()?;

    let logging_levels = vec!["error", "warn", "info", "debug", "trace"];
    let logging_level_index = Select::with_theme(&theme)
        .with_prompt("Logging level")
        .items(&logging_levels)
        .default(2) // info
        .interact()?;

    // Generate configuration
    println!("\n{}", style("📝 Generating configuration...").yellow());

    let config = generate_watch_config(&selected_projects, debounce_delay, logging_levels[logging_level_index])?;

    // Write configuration file
    let config_path = output_dir.join("watch-config.toml");
    fs::write(&config_path, config)?;

    println!("{}", style("✅ Configuration generated successfully!").green().bold());
    println!("📁 Configuration saved to: {}", style(config_path.display()).bold());

    // Next steps
    println!("\n{}", style("🎉 Next Steps:").bold().cyan());
    println!("1. Review the generated configuration in {}", style("watch-config.toml").bold());
    println!("2. Start the file watcher with: {}", style("ddw").bold().green());
    println!("3. Manage projects with: {}", style("ddc list").bold().green());

    Ok(())
}

fn generate_watch_config(projects: &[DetectedProject], debounce_delay: u64, logging_level: &str) -> Result<String, Box<dyn Error>> {
    let templates = get_project_templates();
    let template_map: std::collections::HashMap<String, &ProjectTemplate> = templates.iter()
        .map(|t| (t.name.clone(), t))
        .collect();

    let mut config = String::new();

    // Global configuration
    config.push_str("# DarkDev Configuration\n");
    config.push_str("# Generated by interactive setup wizard\n\n");
    config.push_str("[global]\n");
    config.push_str(&format!("debounce_delay = {}\n", debounce_delay));
    config.push_str(&format!("logging_level = \"{}\"\n", logging_level));
    config.push_str("channel_capacity = 100\n");
    config.push_str("default_modes = [\"compile\"]\n");
    config.push_str("watch_paths = [\"src\"]\n");
    config.push_str("watch_extensions = [\"rs\", \"java\", \"js\", \"ts\", \"py\", \"toml\", \"xml\", \"json\"]\n\n");

    // Projects section
    config.push_str("[projects]\n\n");

    for project in projects {
        if let Some(template) = template_map.get(&project.project_type) {
            config.push_str(&format!("# {} configuration\n", project.name));
            config.push_str(&format!("[projects.{}]\n", project.name.replace("-", "_")));
            config.push_str(&format!("project_dir = \"{}\"\n", project.path.display()));

            if !template.watch_paths.is_empty() {
                config.push_str(&format!("watch_paths = {:?}\n", template.watch_paths));
            }

            if !template.watch_extensions.is_empty() {
                config.push_str(&format!("watch_extensions = {:?}\n", template.watch_extensions));
            }

            config.push_str("dependencies = []\n");
            config.push_str(&format!("modes = {:?}\n", template.default_modes));
            config.push_str("\n");

            // Commands
            config.push_str(&format!("[projects.{}.commands]\n", project.name.replace("-", "_")));
            for (mode, command) in &template.commands {
                config.push_str(&format!("{} = {{ program = \"{}\", args = {:?}, run_on_startup = {} }}\n",
                    mode, command.program, command.args, command.run_on_startup));
            }
            config.push_str("\n");
        }
    }

    Ok(config)
}

fn read_projects_from_config(config_path: &Path) -> Result<HashMap<String, ProjectConfig>, Box<dyn Error>> {
    let config_str = fs::read_to_string(config_path)?;
    let config: WatchConfig = toml::from_str(&config_str)?;

    if let Some(projects) = config.projects {
        Ok(projects)
    } else {
        Ok(HashMap::new())
    }
}

fn validate_config(config_path: &Path, verbose: bool) -> Result<ValidationResult, Box<dyn Error>> {
    let config_str = fs::read_to_string(config_path)?;
    let config: WatchConfig = toml::from_str(&config_str)?;

    let mut result = ValidationResult::new();

    // Validate global configuration
    validate_global_config(&config.global, &mut result);

    // Validate project configurations
    validate_projects_config(&config.projects, &mut result, config_path.parent().unwrap_or(Path::new(".")));

    // Print validation results if verbose
    if verbose {
        print_validation_results(&result);
    }

    Ok(result)
}

fn validate_global_config(global: &Option<GlobalConfig>, result: &mut ValidationResult) {
    if let Some(global) = global {
        // Validate channel capacity
        if let Some(capacity) = global.channel_capacity {
            if capacity == 0 {
                result.add_error("global.channel_capacity".to_string(), "Channel capacity must be greater than 0".to_string());
            } else if capacity > 10000 {
                result.add_warning("global.channel_capacity".to_string(), "Channel capacity is very high, consider reducing it".to_string());
            }
        } else {
            result.add_warning("global.channel_capacity".to_string(), "Channel capacity not specified, using default".to_string());
        }

        // Validate default modes
        if let Some(modes) = &global.default_modes {
            if modes.is_empty() {
                result.add_error("global.default_modes".to_string(), "Default modes cannot be empty".to_string());
            }
            for mode in modes {
                if mode.trim().is_empty() {
                    result.add_error("global.default_modes".to_string(), "Mode names cannot be empty".to_string());
                }
            }
        } else {
            result.add_error("global.default_modes".to_string(), "Default modes are required".to_string());
        }

        // Validate debounce delay
        if let Some(delay) = global.debounce_delay {
            if delay == 0 {
                result.add_warning("global.debounce_delay".to_string(), "Debounce delay of 0 may cause excessive rebuilds".to_string());
            } else if delay > 10000 {
                result.add_warning("global.debounce_delay".to_string(), "Debounce delay is very high, may slow down development".to_string());
            }
        } else {
            result.add_error("global.debounce_delay".to_string(), "Debounce delay is required".to_string());
        }

        // Validate logging level
        if let Some(level) = &global.logging_level {
            let valid_levels = ["trace", "debug", "info", "warn", "error"];
            if !valid_levels.contains(&level.as_str()) {
                result.add_error("global.logging_level".to_string(), 
                    format!("Invalid logging level '{}'. Valid levels: {}", level, valid_levels.join(", ")));
            }
        } else {
            result.add_error("global.logging_level".to_string(), "Logging level is required".to_string());
        }

        // Validate watch paths
        if let Some(paths) = &global.watch_paths {
            if paths.is_empty() {
                result.add_error("global.watch_paths".to_string(), "Watch paths cannot be empty".to_string());
            }
            for path in paths {
                if path.trim().is_empty() {
                    result.add_error("global.watch_paths".to_string(), "Watch path cannot be empty".to_string());
                }
            }
        } else {
            result.add_error("global.watch_paths".to_string(), "Watch paths are required".to_string());
        }

        // Validate watch extensions
        if let Some(extensions) = &global.watch_extensions {
            if extensions.is_empty() {
                result.add_error("global.watch_extensions".to_string(), "Watch extensions cannot be empty".to_string());
            }
            for ext in extensions {
                if ext.trim().is_empty() {
                    result.add_error("global.watch_extensions".to_string(), "Watch extension cannot be empty".to_string());
                } else if ext.starts_with('.') {
                    result.add_warning("global.watch_extensions".to_string(), 
                        format!("Extension '{}' starts with '.', this may be unintended", ext));
                }
            }
        } else {
            result.add_error("global.watch_extensions".to_string(), "Watch extensions are required".to_string());
        }
    } else {
        result.add_error("global".to_string(), "Global configuration is required".to_string());
    }
}

fn validate_projects_config(projects: &Option<HashMap<String, ProjectConfig>>, result: &mut ValidationResult, base_path: &Path) {
    if let Some(projects) = projects {
        if projects.is_empty() {
            result.add_warning("projects".to_string(), "No projects configured".to_string());
            return;
        }

        let project_names: Vec<String> = projects.keys().cloned().collect();

        for (name, project) in projects {
            validate_single_project(name, project, result, base_path, &project_names);
        }

        // Validate dependency cycles
        validate_dependency_cycles(projects, result);
    } else {
        result.add_error("projects".to_string(), "Projects configuration is required".to_string());
    }
}

fn validate_single_project(name: &str, project: &ProjectConfig, result: &mut ValidationResult, base_path: &Path, all_project_names: &[String]) {
    // Validate project directory
    if project.project_dir.is_empty() {
        result.add_error(format!("projects.{}.project_dir", name), "Project directory is required".to_string());
    } else {
        let project_path = base_path.join(&project.project_dir);
        if !project_path.exists() {
            result.add_error(format!("projects.{}.project_dir", name), 
                format!("Project directory '{}' does not exist", project.project_dir));
        } else if !project_path.is_dir() {
            result.add_error(format!("projects.{}.project_dir", name), 
                format!("Project directory '{}' is not a directory", project.project_dir));
        }
    }

    // Validate watch paths
    if let Some(paths) = &project.watch_paths {
        if paths.is_empty() {
            result.add_warning(format!("projects.{}.watch_paths", name), "Watch paths are empty".to_string());
        }
        for path in paths {
            if path.trim().is_empty() {
                result.add_error(format!("projects.{}.watch_paths", name), "Watch path cannot be empty".to_string());
            }
        }
    } else {
        result.add_info(format!("projects.{}.watch_paths", name), "Using global watch paths".to_string());
    }

    // Validate watch extensions
    if let Some(extensions) = &project.watch_extensions {
        if extensions.is_empty() {
            result.add_warning(format!("projects.{}.watch_extensions", name), "Watch extensions are empty".to_string());
        }
        for ext in extensions {
            if ext.trim().is_empty() {
                result.add_error(format!("projects.{}.watch_extensions", name), "Watch extension cannot be empty".to_string());
            } else if ext.starts_with('.') {
                result.add_warning(format!("projects.{}.watch_extensions", name), 
                    format!("Extension '{}' starts with '.', this may be unintended", ext));
            }
        }
    } else {
        result.add_info(format!("projects.{}.watch_extensions", name), "Using global watch extensions".to_string());
    }

    // Validate dependencies
    if let Some(dependencies) = &project.dependencies {
        for dep in dependencies {
            if dep.trim().is_empty() {
                result.add_error(format!("projects.{}.dependencies", name), "Dependency name cannot be empty".to_string());
            } else if dep == name {
                result.add_error(format!("projects.{}.dependencies", name), "Project cannot depend on itself".to_string());
            } else if !all_project_names.contains(dep) {
                result.add_error(format!("projects.{}.dependencies", name), 
                    format!("Dependency '{}' does not exist in configuration", dep));
            }
        }
    }

    // Validate modes
    if let Some(modes) = &project.modes {
        if modes.is_empty() {
            result.add_error(format!("projects.{}.modes", name), "Modes cannot be empty".to_string());
        }
        for mode in modes {
            if mode.trim().is_empty() {
                result.add_error(format!("projects.{}.modes", name), "Mode name cannot be empty".to_string());
            }
        }
    } else {
        result.add_info(format!("projects.{}.modes", name), "Using global default modes".to_string());
    }

    // Validate commands
    if let Some(commands) = &project.commands {
        if commands.is_empty() {
            result.add_warning(format!("projects.{}.commands", name), "No commands configured".to_string());
        }
        for (mode, command) in commands {
            if command.program.trim().is_empty() {
                result.add_error(format!("projects.{}.commands.{}.program", name, mode), "Command program cannot be empty".to_string());
            }
            if command.args.is_empty() {
                result.add_info(format!("projects.{}.commands.{}.args", name, mode), "No arguments specified for command".to_string());
            }
        }
    } else {
        result.add_warning(format!("projects.{}.commands", name), "No commands configured".to_string());
    }
}

fn validate_dependency_cycles(projects: &HashMap<String, ProjectConfig>, result: &mut ValidationResult) {
    fn has_cycle(
        current: &str,
        target: &str,
        projects: &HashMap<String, ProjectConfig>,
        visited: &mut std::collections::HashSet<String>,
        path: &mut Vec<String>,
    ) -> bool {
        if current == target && !path.is_empty() {
            return true;
        }

        if visited.contains(current) {
            return false;
        }

        visited.insert(current.to_string());
        path.push(current.to_string());

        if let Some(project) = projects.get(current) {
            if let Some(dependencies) = &project.dependencies {
                for dep in dependencies {
                    if has_cycle(dep, target, projects, visited, path) {
                        return true;
                    }
                }
            }
        }

        path.pop();
        false
    }

    for project_name in projects.keys() {
        let mut visited = std::collections::HashSet::new();
        let mut path = Vec::new();
        
        if has_cycle(project_name, project_name, projects, &mut visited, &mut path) {
            result.add_error(format!("projects.{}.dependencies", project_name), 
                "Circular dependency detected".to_string());
        }
    }
}

fn print_validation_results(result: &ValidationResult) {
    println!("\n{}", style("Validation Results:").bold().underlined());
    
    if !result.errors.is_empty() {
        println!("\n{}", style("Errors:").red().bold());
        for error in &result.errors {
            println!("  {} {}", style("✗").red(), style(&error.field).red().bold());
            println!("    {}", error.message);
        }
    }

    if !result.warnings.is_empty() {
        println!("\n{}", style("Warnings:").yellow().bold());
        for warning in &result.warnings {
            println!("  {} {}", style("⚠").yellow(), style(&warning.field).yellow().bold());
            println!("    {}", warning.message);
        }
    }

    if !result.info.is_empty() {
        println!("\n{}", style("Info:").blue().bold());
        for info in &result.info {
            println!("  {} {}", style("ℹ").blue(), style(&info.field).blue().bold());
            println!("    {}", info.message);
        }
    }

    println!("\n{}", style("Summary:").bold());
    println!("  Errors: {}", style(result.errors.len()).red());
    println!("  Warnings: {}", style(result.warnings.len()).yellow());
    println!("  Info: {}", style(result.info.len()).blue());
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Setup { non_interactive, output } => {
            let output_dir = output.as_ref()
                .map(|p| p.as_path())
                .unwrap_or_else(|| Path::new("."));

            if *non_interactive {
                println!("Non-interactive setup not yet implemented");
                return Ok(());
            }

            run_interactive_setup(output_dir)?;
        }
        Commands::Init { name, project_type } => {
            let init_project = InitProject {
                name: name.clone(),
                project_type: project_type.clone(),
            };

            let client = Client::new();
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

            let client = Client::new();
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

            let client = Client::new();
            client
                .post(format!("{}/projects/remove", API_URL))
                .json(&project)
                .send()
                .await?;

            println!("Removed project: {}", name);
        }
        Commands::List => {
            // Try to read from watch-config.toml first
            let config_path = Path::new("watch-config.toml");
            if config_path.exists() {
                match read_projects_from_config(config_path) {
                    Ok(projects) => {
                        if projects.is_empty() {
                            println!("No projects found in watch-config.toml");
                        } else {
                            println!("Projects from watch-config.toml:");
                            for (name, config) in projects {
                                let modes = config.modes.as_ref()
                                    .map(|m| m.join(", "))
                                    .unwrap_or_else(|| "compile".to_string());
                                println!("  {} ({}): {}", name, modes, config.project_dir);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading watch-config.toml: {}", e);
                        eprintln!("Falling back to API server...");
                        
                        // Fallback to API server
                        let client = Client::new();
                        match client
                            .get(format!("{}/projects", API_URL))
                            .send()
                            .await
                        {
                            Ok(response) => {
                                match response.json::<Vec<Project>>().await {
                                    Ok(projects) => {
                                        if projects.is_empty() {
                                            println!("No projects found");
                                        } else {
                                            println!("Projects from API server:");
                                            for project in projects {
                                                println!("  {} ({}): {}", project.name, project.mode, project.path);
                                            }
                                        }
                                    }
                                    Err(e) => eprintln!("Error parsing API response: {}", e),
                                }
                            }
                            Err(e) => eprintln!("Error connecting to API server: {}", e),
                        }
                    }
                }
            } else {
                println!("No watch-config.toml found in current directory");
                eprintln!("Run 'ddc setup' to create a configuration file");
            }
        }
        Commands::Validate { config, verbose } => {
            let config_path = config.as_ref()
                .map(|p| p.as_path())
                .unwrap_or_else(|| Path::new("watch-config.toml"));

            if !config_path.exists() {
                eprintln!("Error: Configuration file not found");
                return Ok(());
            }

            let result = validate_config(config_path, *verbose)?;

            if result.is_valid() {
                println!("Configuration is valid");
            } else {
                eprintln!("Configuration is invalid");
            }
        }
    }

    Ok(())
}
