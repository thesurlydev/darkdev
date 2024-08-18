# darkdev

A proof of concept project to demonstrate how to use Rust as a wrapper for rapid development of multiple distributed applications.

The idea is to enable fast feedback loops without a lot of manual steps.

## Features

* Watch multiple projects and their dependencies
* Externalized configuration
* Support for arbitrary "modes" (e.g. compile, test, run, etc.) with configurable commands

## Usage

### Build and Run the Rust Watcher

```bash
cargo build --release
```

```bash
./target/release/darkdev
```

Then, make changes to the Java code or `pom.xml` and observe the output of the feedback loop


## Roadmap

### Infrastructure

Tight integration with infrastructure management tools.

### OpenTelemetry Integration

### Git Integration

