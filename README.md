# darkdev

A development toolkit written in Rust that provides real-time monitoring and project management capabilities. It consists of two main components:

- `ddw` (DarkDev Watch): A file watcher that monitors multiple projects and executes commands when changes are detected
- `ddc` (DarkDev Command): A CLI tool for managing projects and their configurations

## Features

### File Watcher (ddw)
* **Multi-Project Support**: Watch and manage multiple projects simultaneously
* **Smart Debouncing**: Prevents rapid-fire triggers when multiple files change
* **Dependency Management**: Define project dependencies to ensure correct build order
* **Customizable File Watching**:
  * Configure specific paths to watch per project
  * Set file extensions to monitor
  * Global defaults with per-project overrides

### Project Manager (ddc)
* **Project Initialization**: Bootstrap new projects with predefined templates
* **Configuration Management**: Add, remove, and list projects in watch-config.toml
* **Project Modes**: Configure different operational modes (compile, test, run) per project

## Installation

```bash
cargo build --release
```

The binaries will be available in `target/release/`:
- `ddw`: The file watcher
- `ddc`: The project management CLI

## Usage

### Project Management (ddc)

Use the `ddc` command to manage your projects:

```bash
# Initialize a new project
ddc init --name my-project --project-type java

# Add an existing project to watch
ddc add --name my-project --path /path/to/project --mode run

# List all watched projects
ddc list

# Remove a project from watch
ddc remove --name my-project
```

### File Watching (ddw)

Use the `ddw` command to start the file watcher:

```bash
# Start watching with default log level (info)
ddw

# Start watching with debug logging
RUST_LOG=debug ddw
```

## Configuration

Configuration is managed through `watch-config.toml`:

```toml
[global]
default_mode = "compile"          # Default mode for all projects
watch_paths = ["src", "pom.xml"]  # Default paths to watch
watch_extensions = [".java", ".xml"] # File extensions to monitor
debounce_delay = 1000             # Delay in ms before triggering actions

[projects.your_project]
project_dir = "path/to/project"
watch_paths = ["src", "config"]   # Override global paths
mode = "run"                      # Override default mode
dependencies = ["other_project"]  # Projects that must build first

[projects.your_project.commands]
compile = { program = "mvn", args = ["-q", "compile"] }
run = { program = "mvn", args = ["-q", "exec:java"] }
```

## Logging

Both tools support the following log levels via the `RUST_LOG` environment variable:
- `error`: Only errors
- `warn`: Warnings and errors
- `info`: General information (recommended)
- `debug`: Detailed debugging information
- `trace`: Very verbose output

## Roadmap

* Project Templates
  * Java/Maven project templates
  * Rust project templates
  * Custom template support
* Infrastructure
  * Docker integration
  * Kubernetes support
* Monitoring
  * OpenTelemetry integration
  * Metrics collection
* Version Control
  * Git integration
  * Branch-specific configurations
