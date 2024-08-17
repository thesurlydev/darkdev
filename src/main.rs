use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Result, Watcher};
use std::path::Path;
use std::process::{Command, Child};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    // Define the path to watch and the Maven project directory
    let maven_project_dir = "java-project"; // The root directory of your Maven project
    let src_path = Path::new("java-project/src/main/java");
    let test_path = Path::new("java-project/src/test/java");
    let pom_path = Path::new("java-project/pom.xml");

    // Create a channel to receive file change events
    let (tx, mut rx) = mpsc::channel(1);

    // Create a watcher object
    let mut watcher = RecommendedWatcher::new(move |res| {
        if let Ok(event) = res {
            tx.blocking_send(event).expect("Failed to send event.");
        }
    }, Config::default())?;

    // Add a path to be watched
    watcher.watch(src_path, RecursiveMode::Recursive)?;
    watcher.watch(test_path, RecursiveMode::Recursive)?;
    watcher.watch(pom_path, RecursiveMode::NonRecursive)?;

    println!("Watching for file changes...");

    // Keep track of the running Java process
    let running_process: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));

    while let Some(event) = rx.recv().await {
        if let EventKind::Modify(_) = event.kind {
            if let Some(path) = event.paths.get(0) {
                if path.extension() == Some(std::ffi::OsStr::new("java")) || path.ends_with("pom.xml") {
                    println!("Detected change in: {:?}", path);

                    // Terminate the running Java process if any
                    if let Some(mut process) = running_process.lock().unwrap().take() {
                        process.kill().expect("Failed to kill the running Java process.");
                        println!("Terminated the running Java process.");
                    }

                    // Compile and run the new process
                    compile_and_run(maven_project_dir, running_process.clone());
                }
            }
        }
    }


    Ok(())
}

fn compile_and_run(maven_project_dir: &str, running_process: Arc<Mutex<Option<Child>>>) {
    // Start the timer to measure how long compilation and execution take
    let start_time = Instant::now();

    // Run the Maven command to compile and run the Java project in a single step
    println!("Compiling and running the Maven project in {}...", maven_project_dir);

    let child = Command::new("mvn")
        .arg("-q")
        // .arg("-X")
        .arg("clean")
        .arg("compile")
        .arg("exec:java")
        .arg("-Dexec.mainClass=dev.surly.HelloWorld") // Replace with the fully qualified name of your main class
        .arg("-Dexec.cleanupDaemonThreads=false") // Ensure long-running processes aren't prematurely stopped
        .current_dir(maven_project_dir)
        .spawn()
        .expect("Failed to start Maven process");

    // Stop the timer and calculate the duration
    let duration = start_time.elapsed();

    // Print the time it took for the compilation and execution
    println!(
        "Compilation and execution started in {:.2?}. Running the main class...",
        duration
    );

    // Store the child process handle so it can be terminated later
    *running_process.lock().unwrap() = Some(child);
}


/*fn compile_and_run(maven_project_dir: &str, running_process: Arc<Mutex<Option<Child>>>) {
    // Start the timer
    let start_time = Instant::now();

    // Compile the Maven project
    println!("Compiling Maven project in {}...", maven_project_dir);
    let status = Command::new("mvn")
        .arg("compile")
        .arg("-P").arg("fast")
        .arg("-q")
        .current_dir(maven_project_dir)
        .status()
        .expect("Failed to compile Maven project");

    // Stop the timer and calculate the duration
    let duration = start_time.elapsed();

    if status.success() {
        println!(
            "Compilation successful in {:.2?}. Running the main class...",
            duration
        );

        // Run the main class (e.g., App) in the background
        let child = Command::new("mvn")
            .arg("-q")
            .arg("exec:java")
            .arg("-Dexec.mainClass=dev.surly.HelloWorld") // Update this to your main class
            .current_dir(maven_project_dir)
            .spawn()
            .expect("Failed to start Java process");

        // Store the child process handle
        *running_process.lock().unwrap() = Some(child);
    } else {
        println!("Compilation failed.");
    }
}*/

/*fn compile_and_run(maven_project_dir: &str) {
    // Start the timer
    let start_time = Instant::now();

    // Compile the Maven project
    println!("Compiling Maven project in {}...", maven_project_dir);
    let status = Command::new("mvn")
        .arg("compile")
        .current_dir(maven_project_dir)
        .status()
        .expect("Failed to compile Maven project");

    // Stop the timer and calculate the duration
    let duration = start_time.elapsed();

    if status.success() {
        println!(
            "Compilation successful in {:.2?}. Running the main class...",
            duration
        );

        // Run the main class (e.g., App)
        Command::new("mvn")
            .arg("exec:java")
            .arg("-Dexec.mainClass=dev.surly.HelloWorld") // Update this to your main class
            .current_dir(maven_project_dir)
            .status()
            .expect("Failed to execute Java program");
    } else {
        println!("Compilation failed.");
    }
}*/
