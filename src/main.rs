#[macro_use]
extern crate log;
use notify::{EventKind, RecursiveMode, Result, Watcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult};
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;


#[derive(Debug, Deserialize, Clone)]
struct GlobalConfig {
    default_mode: String,
    logging_level: String,
    watch_paths: Vec<String>,
    watch_extensions: Vec<String>,
    channel_capacity: Option<usize>,
    debounce_delay: Option<u64>,
}

#[derive(Debug, Deserialize, Clone)]
struct WatchConfig {
    global: GlobalConfig,
    projects: HashMap<String, ProjectConfig>,
}

#[derive(Debug, Deserialize, Clone)]
struct ProjectConfig {
    project_dir: String,
    watch_paths: Option<Vec<String>>,
    watch_extensions: Option<Vec<String>>,
    dependencies: Vec<String>,
    mode: Option<String>,
    commands: HashMap<String, ModeConfig>,
}

#[derive(Debug, Deserialize, Clone)]
struct ModeConfig {
    program: String,
    args: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = load_config("watch-config.toml").expect("Failed to load config file.");

    // logging setup
    let mut builder = pretty_env_logger::formatted_builder();
    builder.filter_level(match config.global.logging_level.as_str() {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Info,
    });
    builder.init();

    info!("Loaded config");

    let (tx, mut rx) = mpsc::channel(config.global.channel_capacity.unwrap_or(10));

    let mut watchers = Vec::new();

    for (project_name, project_config) in &config.projects {
        let project_name = project_name.clone();
        let project_config = project_config.clone();
        let watch_paths = project_config
            .watch_paths
            .as_ref()
            .unwrap_or(&config.global.watch_paths);

        let project_dir = PathBuf::from(&project_config.project_dir);

        let tx_clone = tx.clone();
        let mut debouncer = new_debouncer(
            Duration::from_millis(config.global.debounce_delay.unwrap_or(1000)),
            None,
            move |res: DebounceEventResult| {
                if let Ok(events) = res {
                    for event in events {
                        if let Err(e) = tx_clone.try_send((project_name.clone(), event.event)) {
                            error!(target: &project_name, "Failed to send event: {}", e);
                        }
                    }
                } else if let Err(e) = res {
                    error!(target: &project_name, "Error in debouncer: {:?}", e);
                }
            },
        )?;

        for path in watch_paths {
            let full_path = project_dir.join(path);
            if !full_path.exists() {
                warn!("Path does not exist: {:?}", full_path);
                continue;
            }
            debouncer.watcher().watch(&full_path, determine_mode(&full_path))?;
        }

        watchers.push(debouncer);
    }

    info!("Watching for file changes...");

    let running_processes: Arc<Mutex<HashMap<String, Option<Child>>>> = Arc::new(Mutex::new(HashMap::new()));

    while let Some((project_name, event)) = rx.recv().await {
        if let EventKind::Modify(_) = event.kind {
            if let Some(path) = event.paths.get(0) {
                let project_config = &config.projects[&project_name];
                let watch_extensions = project_config.watch_extensions.as_ref()
                    .unwrap_or(&config.global.watch_extensions);

                if should_trigger(path, watch_extensions) {
                    info!(target: &project_name, "Detected change in: {:?}", path);

                    for dep in &project_config.dependencies {
                        execute_command(
                            &config.global,
                            &config.projects[dep],
                            running_processes.clone(),
                        );
                    }

                    execute_command(
                        &config.global,
                        project_config,
                        running_processes.clone(),
                    );
                }
            }
        }
    }

    Ok(())
}

fn load_config(filename: &str) -> std::result::Result<WatchConfig, Box<dyn Error>> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let config: WatchConfig = toml::from_str(&contents)?;
    Ok(config)
}

fn should_trigger(path: &Path, extensions: &[String]) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map_or(false, |ext| extensions.iter().any(|e| e == ext))
}

fn execute_command(
    global_config: &GlobalConfig,
    project_config: &ProjectConfig,
    running_processes: Arc<Mutex<HashMap<String, Option<Child>>>>,
) {
    let mode = project_config.mode.as_deref().unwrap_or(&global_config.default_mode);

    if let Some(mode_config) = project_config.commands.get(mode) {
        let project_dir = PathBuf::from(&project_config.project_dir);

        terminate_running_process(&project_config.project_dir, &running_processes);

        info!(target: &project_dir.display().to_string(), "Executing: {}", mode);

        let child = Command::new(&mode_config.program)
            .args(&mode_config.args)
            .current_dir(&project_dir)
            .spawn()
            .map_err(|e| {
                error!(target: &project_dir.display().to_string(), "Failed to start process: {}", e);
            }).expect("Failed to start process");


        running_processes
            .lock()
            .unwrap()
            .insert(project_config.project_dir.clone(), Some(child));
    } else {
        info!(target: &project_config.project_dir, "No command defined for mode '{}'. Skipping...",mode);
    }
}

fn terminate_running_process(
    project_dir: &str,
    running_processes: &Arc<Mutex<HashMap<String, Option<Child>>>>,
) {
    if let Some(mut process) = running_processes.lock().unwrap().get_mut(project_dir).and_then(Option::take) {
        process.kill().expect("Failed to kill the running process.");
        info!(target: project_dir, "Terminated the running process");
    }
}

fn determine_mode(path: &PathBuf) -> RecursiveMode {
    if path.is_dir() {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    }
}
