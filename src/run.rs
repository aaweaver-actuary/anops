use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use crate::config;

// --- Helper Functions (Consider moving to a shared utils module later) --- //

/// Searches upwards from the starting path for a file named `ao.toml`.
/// Returns the path to the directory containing `ao.toml` if found.
// TODO: Move this to a shared location (e.g., `src/project.rs` or `src/utils.rs`)
fn find_project_root(start_path: &Path) -> Result<PathBuf> {
    let mut current_path = start_path.canonicalize()
        .with_context(|| format!("Failed to canonicalize path: {}", start_path.display()))?;

    loop {
        let config_path = current_path.join("ao.toml");
        if config_path.exists() && config_path.is_file() {
            return Ok(current_path);
        }

        if let Some(parent) = current_path.parent() {
            current_path = parent.to_path_buf();
        } else {
            bail!("Could not find project root (ao.toml) starting from {}", start_path.display());
        }
    }
}

/// Executes an external tool/command within the project directory.
// TODO: Move this to a shared location (e.g., `src/cmd.rs` or `src/utils.rs`)
fn run_tool(command_str: &str, project_root: &Path) -> Result<()> {
    println!("Running tool: '{}' in {}", command_str, project_root.display());

    if command_str.is_empty() {
        bail!("Cannot run an empty command string.");
    }

    let parts: Vec<&str> = command_str.split_whitespace().collect();
    if parts.is_empty() {
        bail!("Command string '{}' resulted in no executable parts.", command_str);
    }
    let executable = parts[0];
    let args = &parts[1..];

    let mut command = Command::new(executable);
    command.args(args);
    command.current_dir(project_root);
    command.stdout(Stdio::inherit());
    command.stderr(Stdio::inherit());

    let status = command.status()
        .with_context(|| format!("Failed to execute command: '{}'", command_str))?;

    if status.success() {
        println!("Tool '{}' finished successfully.", command_str);
        Ok(())
    } else {
        bail!("Tool '{}' failed with status: {}", command_str, status);
    }
}

// --- Main `run` function --- //

/// Handler for `ao run <task_name>`.
/// Finds the project root, loads config, and executes the steps for the specified task.
///
/// # Arguments
///
/// * `task_name` - The name of the task defined in `ao.toml` to execute.
/// * `path_str` - Path within the project directory to start searching from.
///
/// # Errors
///
/// Returns an error if the project root is not found, config loading fails,
/// the task is not found, or any command within the task fails.
pub fn run(task_name: String, path_str: String) -> Result<()> {
    let start_path = Path::new(&path_str);
    println!("Attempting to run task '{}' starting from '{}'", task_name, start_path.display());

    let project_path = find_project_root(start_path)
        .with_context(|| format!("Failed to find project root starting from '{}'", start_path.display()))?;
    println!("Found project root at '{}'", project_path.display());

    // Load configuration
    let config = config::load_config(&project_path)
        .context("Failed to load project configuration")?;
    println!("Project name from config: {}", config.project.name);

    // Find the requested task
    match config.tasks.get(&task_name) {
        Some(commands) => {
            println!("--- Running task '{}' ---", task_name);
            if commands.is_empty() {
                println!("Task '{}' has no commands defined.", task_name);
            } else {
                for command_str in commands {
                    run_tool(command_str, &project_path)
                        .with_context(|| format!("Command '{}' in task '{}' failed", command_str, task_name))?;
                }
            }
            println!("--- Task '{}' finished successfully ---", task_name);
            Ok(())
        }
        None => {
            bail!("Task '{}' not found in ao.toml", task_name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init; // To set up a project structure
    use std::fs;
    use tempfile::tempdir;

    // Helper to create a project with a specific ao.toml content
    fn setup_project_with_config(base_path: &Path, config_content: &str) -> Result<PathBuf> {
        let project_dir = base_path.join("test_run_project");
        // Run init first to get base structure (it creates a basic ao.toml)
        init::run(project_dir.to_str().unwrap().to_string())?;
        // Overwrite ao.toml with specific content
        let config_path = project_dir.join("ao.toml");
        fs::write(config_path, config_content).context("Failed to write test config")?;
        Ok(project_dir)
    }

    #[test]
    fn run_succeeds_with_valid_task() {
        let tmp_dir = tempdir().unwrap();
        let config_content = r#"
[project]
name = "run-test"

[tasks]
build = ["echo building...", "mkdir build_output"]
"#;
        let project_path = setup_project_with_config(tmp_dir.path(), config_content).unwrap();

        let result = run("build".to_string(), project_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
        // Check side effect of the command
        assert!(project_path.join("build_output").exists());
        assert!(project_path.join("build_output").is_dir());
    }

    #[test]
    fn run_succeeds_with_empty_task() {
        let tmp_dir = tempdir().unwrap();
        let config_content = r#"
[project]
name = "empty-task-test"

[tasks]
empty = []
"#;
        let project_path = setup_project_with_config(tmp_dir.path(), config_content).unwrap();

        let result = run("empty".to_string(), project_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
        // No side effects to check
    }

    #[test]
    fn run_fails_if_task_not_found() {
        let tmp_dir = tempdir().unwrap();
        let config_content = r#"
[project]
name = "no-such-task-test"

[tasks]
build = ["echo build"]
"#;
        let project_path = setup_project_with_config(tmp_dir.path(), config_content).unwrap();

        let result = run("deploy".to_string(), project_path.to_str().unwrap().to_string()); // Task 'deploy' doesn't exist

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Task 'deploy' not found in ao.toml"));
    }

    #[test]
    fn run_fails_if_command_in_task_fails() {
        let tmp_dir = tempdir().unwrap();
        let config_content = r#"
[project]
name = "failing-task-test"

[tasks]
build = ["echo starting...", "ls non_existent_file_in_task", "echo finished?"]
"#;
        let project_path = setup_project_with_config(tmp_dir.path(), config_content).unwrap();

        let result = run("build".to_string(), project_path.to_str().unwrap().to_string());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        // Check that the error context includes the failing command and task name
        assert!(err_msg.contains("Command 'ls non_existent_file_in_task' in task 'build' failed"));
        // Also check the underlying error from run_tool
        assert!(err_msg.contains("Tool 'ls non_existent_file_in_task' failed with status"));
    }

    #[test]
    fn run_fails_if_project_root_not_found() {
        let tmp_dir = tempdir().unwrap();
        // Don't create any project or config
        let non_project_path = tmp_dir.path().join("not_a_project");
        fs::create_dir(&non_project_path).unwrap();

        let result = run("build".to_string(), non_project_path.to_str().unwrap().to_string());

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Could not find project root"));
    }

    #[test]
    fn run_fails_if_config_is_malformed() {
        let tmp_dir = tempdir().unwrap();
        // Create a project but with invalid TOML
        let project_path = tmp_dir.path().join("malformed_config_project");
        init::run(project_path.to_str().unwrap().to_string()).unwrap();
        fs::write(project_path.join("ao.toml"), "[project]name=").unwrap(); // Malformed

        let result = run("build".to_string(), project_path.to_str().unwrap().to_string());

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to parse TOML config file"));
    }

    #[test]
    fn run_works_when_called_from_subdir() {
         let tmp_dir = tempdir().unwrap();
        let config_content = r#"
[project]
name = "run-from-subdir-test"

[tasks]
build = ["echo building...", "mkdir ../build_output_subdir"] # Create output relative to root
"#;
        let project_path = setup_project_with_config(tmp_dir.path(), config_content).unwrap();
        let models_path = project_path.join("models"); // Subdir created by init

        // Run from the 'models' subdirectory
        let result = run("build".to_string(), models_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
        // Check side effect relative to the project root
        assert!(project_path.join("build_output_subdir").exists());
        assert!(project_path.join("build_output_subdir").is_dir());
    }
}
