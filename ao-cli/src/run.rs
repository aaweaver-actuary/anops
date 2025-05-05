use crate::config;
use anyhow::{bail, Context, Result};
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::utils::{find_project_root, run_tool}; // Import from utils

// --- Helper Functions removed, now in utils.rs --- //

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
    println!(
        "Running task '{}' starting from '{}'",
        task_name,
        start_path.display()
    );

    // Find project root using the utility function
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
                    // Use the utility function to run the command
                    run_tool(command_str, &project_path).with_context(|| {
                        format!("Command '{}' in task '{}' failed", command_str, task_name)
                    })?;
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
        let project_name = "test_run_project"; // Name used inside config content
        let config_content = format!(
            r#"[project]
name = "{}"

[tasks]
build = ["mkdir build_output"] # Simple command
"#,
            project_name
        );
        let project_path = setup_project_with_config(tmp_dir.path(), &config_content).unwrap();

        let result = run("build".to_string(), project_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
        // Check side effect of the command
        assert!(project_path.join("build_output").exists());
        assert!(project_path.join("build_output").is_dir());
    }

    #[test]
    fn run_succeeds_with_empty_task() {
        let tmp_dir = tempdir().unwrap();
        let project_name = "test_run_project";
        let config_content = format!(
            r#"[project]
name = "{}"

[tasks]
empty = [] # Empty command list
"#,
            project_name
        );
        let project_path = setup_project_with_config(tmp_dir.path(), &config_content).unwrap();

        let result = run("empty".to_string(), project_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
        // No side effects to check
    }

    #[test]
    fn run_fails_if_task_not_found() {
        let tmp_dir = tempdir().unwrap();
        let project_name = "test_run_project";
        let config_content = format!(
            r#"[project]
name = "{}"

[tasks]
build = ["echo hello"]
"#,
            project_name
        );
        let project_path = setup_project_with_config(tmp_dir.path(), &config_content).unwrap();

        let result = run("deploy".to_string(), project_path.to_str().unwrap().to_string()); // Task 'deploy' doesn't exist

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Task 'deploy' not found in ao.toml"));
    }

    #[test]
    fn run_fails_if_command_in_task_fails() {
        let tmp_dir = tempdir().unwrap();
        let project_name = "test_run_project";
        // Use a command that will fail (ls on a non-existent file)
        let config_content = format!(
            r#"[project]
name = "{}"

[tasks]
build = ["ls non_existent_file_in_task"]
"#,
            project_name
        );
        let project_path = setup_project_with_config(tmp_dir.path(), &config_content).unwrap();

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

        let result = run(
            "build".to_string(),
            non_project_path.to_str().unwrap().to_string(),
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Could not find project root"));
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
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse TOML config file"));
    }

    #[test]
    fn run_works_when_called_from_subdir() {
        let tmp_dir = tempdir().unwrap();
        let project_name = "test_run_project";
        let config_content = format!(
            r#"[project]
name = "{}"

[tasks]
build = ["mkdir build_output_subdir"]
"#,
            project_name
        );
        let project_path = setup_project_with_config(tmp_dir.path(), &config_content).unwrap();
        let models_path = project_path.join("models"); // Subdir created by init

        // Run from the 'models' subdirectory
        let result = run("build".to_string(), models_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
        // Check side effect relative to the project root
        assert!(project_path.join("build_output_subdir").exists());
        assert!(project_path.join("build_output_subdir").is_dir());
    }
}
