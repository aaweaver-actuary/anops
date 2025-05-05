use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Searches upwards from the starting path for a file named `ao.toml`.
/// Returns the path to the directory containing `ao.toml` if found.
pub fn find_project_root(start_path: &Path) -> Result<PathBuf> {
    println!("find_project_root: Starting search from '{}'", start_path.display()); // Added log
    // Canonicalize the starting path to resolve symlinks and relative components
    let mut current_path = start_path
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize path: {}", start_path.display()))?;
    println!("find_project_root: Canonical path is '{}'", current_path.display()); // Added log

    loop {
        let config_path = current_path.join("ao.toml");
        println!("find_project_root: Checking for config at '{}'", config_path.display()); // Added log
        if config_path.exists() && config_path.is_file() {
            println!("find_project_root: Found config at '{}'", config_path.display()); // Added log
            return Ok(current_path);
        }

        // Move up to the parent directory
        if let Some(parent) = current_path.parent() {
            // Check if we are already at the root to prevent infinite loop
            if parent == current_path {
                println!("find_project_root: Reached filesystem root, config not found."); // Added log
                break;
            }
            println!("find_project_root: Moving up to parent '{}'", parent.display()); // Added log
            current_path = parent.to_path_buf();
        } else {
            // Should not happen if parent == current_path check works, but as a safeguard
            println!("find_project_root: No parent found, config not found."); // Added log
            break;
        }
    }

    // If loop finishes without returning, the file was not found
    bail!(
        "Could not find project root (ao.toml) starting from {}",
        start_path.display()
    );
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
pub fn run_tool(command_str: &str, project_root: &Path) -> Result<()> {
    println!(
        "Running tool: '{}' in {}",
        command_str,
        project_root.display()
    );

    if command_str.is_empty() {
        bail!("Cannot run an empty command string.");
    }

    // Basic command parsing (split by space, handle potential quotes later if needed)
    let parts: Vec<&str> = command_str.split_whitespace().collect();
    if parts.is_empty() {
        bail!(
            "Command string '{}' resulted in no executable parts.",
            command_str
        );
    }
    let cmd_name = parts[0];
    let args = &parts[1..];

    let mut command = Command::new(cmd_name);
    command.args(args);
    command.current_dir(project_root);
    command.stdout(Stdio::inherit()); // Stream stdout directly
    command.stderr(Stdio::inherit()); // Stream stderr directly

    let status = command
        .status()
        .with_context(|| format!("Failed to execute command: '{}'", command_str))?;

    if status.success() {
        println!("Tool '{}' finished successfully.", command_str);
        Ok(())
    } else {
        bail!("Tool '{}' failed with status: {}", command_str, status);
    }
}
