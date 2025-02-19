# darkdev

A development tool written in Rust that provides real-time monitoring and automated actions for multiple projects. It enables fast feedback loops without manual intervention by watching your projects and executing configured commands when changes are detected.

## Features

* **Multi-Project Support**: Watch and manage multiple projects simultaneously
* **Flexible Configuration**: All settings are externalized in `watch-config.toml`
* **Project Modes**: Support for multiple operational modes (compile, test, run, etc.) with configurable commands
* **Smart Debouncing**: Prevents rapid-fire triggers when multiple files change
* **Dependency Management**: Define project dependencies to ensure correct build order
* **Customizable File Watching**:
  * Configure specific paths to watch per project
  * Set file extensions to monitor
  * Global defaults with per-project overrides

## Configuration

Configuration is managed through `watch-config.toml` with the following structure:

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

## Usage

1. Configure your projects in `watch-config.toml`

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Run the watcher:
   ```bash
   RUST_LOG=info ./target/release/darkdev
   ```

4. Make changes to your watched files and observe the automated actions

## Logging

Set the `RUST_LOG` environment variable to control log levels:
- `error`: Only errors
- `warn`: Warnings and errors
- `info`: General information (recommended)
- `debug`: Detailed debugging information
- `trace`: Very verbose output

## Roadmap

* Project bootstrapping tools
* Infrastructure automation
* OpenTelemetry integration
* Git integration
* Watch pattern improvements
* Cross-platform testing
