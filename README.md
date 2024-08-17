# darkdev

A proof of concept project to demonstrate how to use Rust as a wrapper for rapid development of a Java application.

The idea is to enable fast feedback loops without a lot of manual steps.


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

### Configurable Behavior

Allow for arbitrary commands and feedback loops to be defined in a configuration file.

### Watch Modes

Allow for different watch modes to be used such as:

* watch and compile
* watch and test
* watch, compile, test and deploy (if tests pass)

### Infrastructure

Tight integration with infrastructure management tools.

### Observability

### Git Integration

