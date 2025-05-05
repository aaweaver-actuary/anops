use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use crate::config; // Import the config module

/// Searches upwards from the starting path for a file named `ao.toml`.
/// Returns the path to the directory containing `ao.toml` if found.
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
///
/// # Arguments
///
/// * `command_str` - The command string to execute (e.g., "ruff check .").
/// * `project_root` - The path to the project root directory, used as the working directory.
///
/// # Errors
///
/// Returns an error if the command cannot be executed or if it exits with a non-zero status.
fn run_tool(command_str: &str, project_root: &Path) -> Result<()> {
    println!("Running tool: '{}' in {}", command_str, project_root.display());

    if command_str.is_empty() {
        bail!("Cannot run an empty command string.");
    }

    // Basic command parsing (split by space, handle potential quotes later if needed)
    let parts: Vec<&str> = command_str.split_whitespace().collect();
    if parts.is_empty() {
        bail!("Command string '{}' resulted in no executable parts.", command_str);
    }
    let executable = parts[0];
    let args = &parts[1..];

    let mut command = Command::new(executable);
    command.args(args);
    command.current_dir(project_root);
    command.stdout(Stdio::inherit()); // Stream stdout directly
    command.stderr(Stdio::inherit()); // Stream stderr directly

    let status = command.status()
        .with_context(|| format!("Failed to execute command: '{}'", command_str))?;

    if status.success() {
        println!("Tool '{}' finished successfully.", command_str);
        Ok(())
    } else {
        bail!("Tool '{}' failed with status: {}", command_str, status);
    }
}

/// Handler for `ao check`.
/// Verifies structure, loads config, and runs configured linters/testers.
///
/// # Arguments
///
/// * `path_str` - Path within the project directory to start searching from.
///
/// # Errors
///
/// Returns an error if any step (root finding, config load, structure check, tool execution) fails.
pub fn run(path_str: String) -> Result<()> {
    let start_path = Path::new(&path_str);
    println!("Starting check from '{}'", start_path.display());

    let project_path = find_project_root(start_path)
        .with_context(|| format!("Failed to find project root starting from '{}'", start_path.display()))?;
    println!("Found project root at '{}'", project_path.display());

    // Load configuration
    let config = config::load_config(&project_path)
        .context("Failed to load project configuration")?;
    println!("Project name from config: {}", config.project.name);

    println!("Running structure checks in '{}'", project_path.display());

    // Check for required directories relative to the found project root
    let required_dirs = ["models", "tests", "notebooks"];
    for dir_name in &required_dirs {
        let dir_path = project_path.join(dir_name);
        if !dir_path.exists() {
            bail!("Required directory '{}' not found in project root.", dir_path.display());
        }
        if !dir_path.is_dir() {
            bail!("Path '{}' is not a directory.", dir_path.display());
        }
        println!("Found directory: {:?}", dir_path);
    }

    // Config file presence is already checked by find_project_root and load_config
    println!("Found config file: {:?}", project_path.join("ao.toml"));

    // Run configured linters
    if !config.check.linters.is_empty() {
        println!("--- Running Linters ---");
        for linter_cmd in &config.check.linters {
            run_tool(linter_cmd, &project_path)
                .with_context(|| format!("Linter command '{}' failed", linter_cmd))?;
        }
        println!("--- Linters Finished ---");
    } else {
        println!("No linters configured.");
    }

    // Run configured testers
    if !config.check.testers.is_empty() {
        println!("--- Running Testers ---");
        for tester_cmd in &config.check.testers {
            run_tool(tester_cmd, &project_path)
                .with_context(|| format!("Tester command '{}' failed", tester_cmd))?;
        }
        println!("--- Testers Finished ---");
    } else {
        println!("No testers configured.");
    }

    println!("All checks passed successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init;
    use std::fs;
    use tempfile::tempdir;
        let project_path = tmp_dir.path().join("no_config_project");
        fs::create_dir_all(&project_path).unwrap();
        fs::create_dir_all(project_path.join("models")).unwrap();
        let result = find_project_root(&project_path);
        assert!(result.is_err());
    }

    #[test]
    fn run_succeeds_when_called_from_root() {
        let tmp_dir = tempdir().unwrap();
        let project_path = setup_valid_project(tmp_dir.path()).unwrap();
        let result = run(project_path.to_str().unwrap().to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn run_succeeds_when_called_from_subdir() {
        let tmp_dir = tempdir().unwrap();
        let project_path = setup_valid_project(tmp_dir.path()).unwrap();
        let models_path = project_path.join("models");
        let result = run(models_path.to_str().unwrap().to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn run_succeeds_with_check_config_present() {
        let tmp_dir = tempdir().unwrap();
        let project_path = setup_valid_project(tmp_dir.path()).unwrap();
        add_check_config(&project_path); // Add [check] section
        let result = run(project_path.to_str().unwrap().to_string());
        assert!(result.is_ok());
        // We could capture stdout here to verify the print messages if needed
    }

    #[test]
    fn run_fails_if_path_does_not_exist() {
        let tmp_dir = tempdir().unwrap();
        let project_path = tmp_dir.path().join("non_existent_project");
        let result = run(project_path.to_str().unwrap().to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to find project root") || result.unwrap_err().to_string().contains("Failed to canonicalize"));
    }

    #[test]
    fn run_fails_if_no_project_found() {
        let tmp_dir = tempdir().unwrap();
        let empty_dir = tmp_dir.path().join("empty_dir");
        fs::create_dir(&empty_dir).unwrap();
        let result = run(empty_dir.to_str().unwrap().to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Could not find project root"));
    }

    #[test]
    fn run_fails_if_config_is_malformed() {
        let tmp_dir = tempdir().unwrap();
        let project_path = setup_valid_project(tmp_dir.path()).unwrap();
        // Overwrite with malformed config
        fs::write(project_path.join("ao.toml"), "[project]name=").unwrap();
        let result = run(project_path.to_str().unwrap().to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to parse TOML config file"));
    }

    #[test]
    fn run_fails_if_structure_invalid_even_if_found() {
        let tmp_dir = tempdir().unwrap();
        let project_path = setup_valid_project(tmp_dir.path()).unwrap();
        fs::remove_dir_all(project_path.join("models")).unwrap();
        let result = run(project_path.to_str().unwrap().to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Required directory"));
        assert!(result.unwrap_err().to_string().contains("models' not found"));
    }
}