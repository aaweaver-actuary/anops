use anyhow::{bail, Context, Result, anyhow};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::fs;
use shlex;
use tracing::{info, warn, error};

/// Searches upwards from the starting path for a file named `ao.toml`.
/// Returns the path to the directory containing `ao.toml` if found.
pub fn find_project_root(start_path: &Path) -> Result<PathBuf> {
    info!("find_project_root: Starting search from '{}'", start_path.display());
    // Canonicalize the starting path to resolve symlinks and relative components
    let mut current_path = start_path
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize path: {}", start_path.display()))?;
    info!("find_project_root: Canonical path is '{}'", current_path.display());

    loop {
        let config_path = current_path.join("ao.toml");
        info!("find_project_root: Checking for config at '{}'", config_path.display());
        if config_path.exists() && config_path.is_file() {
            info!("find_project_root: Found config at '{}'", config_path.display());
            return Ok(current_path);
        }

        // Move up to the parent directory
        if let Some(parent) = current_path.parent() {
            // Check if we are already at the root to prevent infinite loop
            if parent == current_path {
                warn!("find_project_root: Reached filesystem root, config not found.");
                break;
            }
            info!("find_project_root: Moving up to parent '{}'", parent.display());
            current_path = parent.to_path_buf();
        } else {
            // Should not happen if parent == current_path check works, but as a safeguard
            warn!("find_project_root: No parent found, config not found.");
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
    // Use shlex for robust shell-like parsing
    let parts: Vec<String> = shlex::split(command_str)
        .ok_or_else(|| anyhow!("Failed to parse command string with shlex: '{}'", command_str))?;
    if parts.is_empty() {
        bail!("Command string '{}' resulted in no executable parts.", command_str);
    }
    let cmd_name = &parts[0];
    let args = &parts[1..];
    let mut command = Command::new(cmd_name);
    command.args(args);
    command.current_dir(project_root);
    command.stdout(Stdio::inherit());
    command.stderr(Stdio::inherit());
    let status = command
        .status()
        .with_context(|| format!("Failed to execute command: '{}'", command_str))?;
    if status.success() {
        info!("Tool '{}' finished successfully.", command_str);
        return Ok(());
    }
    let cmd_name = &parts[0];
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
        info!("Tool '{}' finished successfully.", command_str);
        return Ok(());
    } else {
        error!("Tool '{}' failed with status: {}", command_str, status);
        bail!("Tool '{}' failed with status: {}", command_str, status);
    }
}

/// Generates gRPC code using python -m grpc_tools.protoc
/// Assumes proto files are in model-interface and outputs to api-service and model-service.
pub fn generate_grpc_code(project_root: &Path) -> Result<()> {
    info!("--- Generating gRPC Code ---");
    let interface_dir = project_root.join("model-interface");
    let api_service_dir = project_root.join("api-service");
    let model_service_dir = project_root.join("model-service");
    let proto_file = interface_dir.join("anops.proto");

    if !proto_file.exists() {
        bail!("Proto file not found at {}", proto_file.display());
    }

    // Ensure output directories exist
    fs::create_dir_all(&api_service_dir)
        .with_context(|| format!("Failed to ensure api-service directory exists: {}", api_service_dir.display()))?;
    fs::create_dir_all(&model_service_dir)
        .with_context(|| format!("Failed to ensure model-service directory exists: {}", model_service_dir.display()))?;


    // Construct the command. Using Command directly to avoid run_tool's parsing issues for now.
    // We run it from the project_root context.
    // Note: Assumes 'python' and 'grpc_tools.protoc' are available in the PATH.
    let mut command = Command::new("python");
    command.arg("-m")
           .arg("grpc_tools.protoc")
           .arg(format!("-I{}", interface_dir.display())) // Include path for proto file
           // Output to api-service
           .arg(format!("--python_out={}", api_service_dir.display()))
           .arg(format!("--pyi_out={}", api_service_dir.display()))
           .arg(format!("--grpc_python_out={}", api_service_dir.display()))
           // Output to model-service
           .arg(format!("--python_out={}", model_service_dir.display()))
           .arg(format!("--pyi_out={}", model_service_dir.display()))
           .arg(format!("--grpc_python_out={}", model_service_dir.display()))
           // The proto file itself (relative to include path)
           .arg(proto_file.file_name().unwrap().to_str().unwrap()); // Use just the filename relative to -I

    command.current_dir(project_root); // Run from project root
    command.stdout(Stdio::inherit());
    command.stderr(Stdio::inherit());

    info!("Executing: {:?}", command);

    let status = command
        .status()
        .context("Failed to execute python -m grpc_tools.protoc command. Is grpcio-tools installed and python in PATH?")?;

    if status.success() {
        info!("gRPC code generated successfully.");
        info!("--- gRPC Code Generation Finished ---");
        Ok(())
    } else {
        error!("gRPC code generation failed with status: {}", status);
        bail!("gRPC code generation failed with status: {}", status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init; // To setup project structure
    use std::fs;
    use tempfile::tempdir;

    // TODO: Add tests for find_project_root
    // TODO: Add tests for run_tool (mocking Command)

    #[test]
    fn generate_grpc_code_runs_without_panic_on_valid_structure() {
        // This is a basic test to ensure the function can be called,
        // finds paths, and attempts to run the command without panicking.
        // It DOES NOT verify the command executes correctly or files are generated,
        // as that requires python/grpcio-tools and filesystem changes.
        // TODO: Implement proper mocking of std::process::Command for robust testing.

        let tmp_dir = tempdir().unwrap();
        let project_name = "test_grpc_gen_project";
        let project_path = tmp_dir.path().join(project_name);

        // Use init::run to create the necessary structure
        init::run(project_path.to_str().unwrap().to_string()).unwrap();

        // Ensure the proto file exists (created by init::run)
        assert!(project_path.join("model-interface/anops.proto").exists());

        // Call the function - we expect Ok(()) if it constructs the command,
        // even if the command itself fails externally.
        // If python/grpcio-tools are not installed, this might return Err,
        // but the test aims to catch panics within generate_grpc_code itself.
        let result = generate_grpc_code(&project_path);

        // Basic assertion: Check if the function completed its logic.
        // If python/grpcio-tools aren't installed, it will likely return Err here.
        // If they ARE installed, it should return Ok.
        // We accept either Ok or an Err containing the execution failure message.
        match result {
            Ok(_) => info!("generate_grpc_code returned Ok (python/grpcio-tools likely found)"),
            Err(e) => {
                let msg = e.to_string();
                warn!("generate_grpc_code returned Err: {} (python/grpcio-tools likely not found or failed)", msg);
                // Check it's the expected execution error, not a setup error
                assert!(msg.contains("Failed to execute") || msg.contains("gRPC code generation failed"));
            }
        }
    }

    #[test]
    fn generate_grpc_code_fails_if_proto_missing() {
        let tmp_dir = tempdir().unwrap();
        let project_name = "test_grpc_gen_no_proto";
        let project_path = tmp_dir.path().join(project_name);

        // Create partial structure WITHOUT the proto file
        fs::create_dir_all(project_path.join("model-interface")).unwrap();
        fs::create_dir_all(project_path.join("api-service")).unwrap();
        fs::create_dir_all(project_path.join("model-service")).unwrap();
        // Create ao.toml so find_project_root works if called implicitly later
        fs::write(project_path.join("ao.toml"), "[project]\nname=\"test\"").unwrap();

        let result = generate_grpc_code(&project_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Proto file not found"));
    }

    #[test]
    fn run_tool_succeeds_with_echo() {
        use std::env;
        use tempfile::tempdir;
        let tmp_dir = tempdir().unwrap();
        // Use a harmless command that works on all platforms
        let result = run_tool("echo hello", tmp_dir.path());
        assert!(result.is_ok());
    }

    // Note: For more robust mocking of external commands, consider using the 'assert_cmd' crate or similar in the future.
}
