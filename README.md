# darkdev

A development toolkit written in Rust that provides real-time monitoring and project management capabilities. It consists of three main components:

- `ddw` (DarkDev Watch): A file watcher that monitors multiple projects and executes commands when changes are detected
- `ddc` (DarkDev Command): A CLI tool for managing projects and their configurations
- `dda` (DarkDev API): HTTP API server for project management operations

## Features

### Interactive Setup Wizard 
* **Smart Project Detection**: Automatically detects Java (Maven), Rust (Cargo), Node.js, and Python projects
* **Guided Configuration**: Step-by-step wizard to configure DarkDev for your environment
* **Template-Based Setup**: Pre-configured templates for common project types
* **Beautiful CLI**: Colorful, interactive prompts with sensible defaults

### File Watcher (ddw)
* **Multi-Project Support**: Watch and manage multiple projects simultaneously
* **Smart Debouncing**: Prevents rapid-fire triggers when multiple files change
* **Dependency Management**: Define project dependencies to ensure correct build order
* **Customizable File Watching**:
  * Configure specific paths to watch per project
  * Set file extensions to monitor
  * Global defaults with per-project overrides

### Project Manager (ddc)
* **Interactive Setup**: `ddc setup` - guided configuration wizard
* **Project Initialization**: Bootstrap new projects with predefined templates
* **Configuration Management**: Add, remove, and list projects in watch-config.toml
* **Project Modes**: Configure different operational modes (compile, test, run) per project

### API Server (dda)
* **REST API**: HTTP endpoints for project management
* **Real-time Operations**: Add, remove, and list projects via API
* **Health Monitoring**: Built-in health check endpoints

## Installation

```bash
cargo build --release
```

The binaries will be available in `target/release/`:
- `ddw`: The file watcher
- `ddc`: The project management CLI
- `dda`: The API server

## Quick Start

### 1. Interactive Setup (Recommended)

The easiest way to get started is with the interactive setup wizard:

```bash
# Build DarkDev
cargo build --release

# Run the interactive setup wizard
./target/release/ddc setup
```

The wizard will:
- Scan your directory for projects (Java, Rust, Node.js, Python)
- Let you select which projects to include
- Configure global settings (debounce delay, logging level)
- Generate a complete `watch-config.toml` configuration
- Provide next steps to get you running

### 2. Manual Setup

If you prefer manual configuration:

## Usage

### Project Management (ddc)

#### Interactive Setup
```bash
# Run the setup wizard in current directory
ddc setup

# Run setup wizard with custom output directory
ddc setup --output /path/to/project

# Non-interactive setup (coming soon)
ddc setup --non-interactive
```

#### Manual Project Management
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
