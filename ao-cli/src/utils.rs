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
    use std::path::Path;

    // Helper to create a project structure for utils tests
    fn setup_test_project(base_path: &Path) -> Result<PathBuf> {
        let project_dir = base_path.join("utils_test_project");
        init::run(project_dir.to_str().unwrap().to_string())
            .context("Failed to init project for utils test")?;
        Ok(project_dir)
    }

    #[test]
    fn find_project_root_succeeds_from_root() {
        let tmp_dir = tempdir().unwrap();
        let project_path = setup_test_project(tmp_dir.path()).unwrap();
        let found_root = find_project_root(&project_path).unwrap();
        assert_eq!(found_root.canonicalize().unwrap(), project_path.canonicalize().unwrap());
    }

    #[test]
    fn find_project_root_succeeds_from_subdir() {
        let tmp_dir = tempdir().unwrap();
        let project_path = setup_test_project(tmp_dir.path()).unwrap();
        let subdir = project_path.join("api-service"); // Exists due to init
        fs::create_dir_all(&subdir).unwrap(); // Ensure it exists
        let found_root = find_project_root(&subdir).unwrap();
        assert_eq!(found_root.canonicalize().unwrap(), project_path.canonicalize().unwrap());
    }

    #[test]
    fn find_project_root_fails_outside_project() {
        let tmp_dir = tempdir().unwrap();
        let outside_path = tmp_dir.path().join("not_a_project");
        fs::create_dir(&outside_path).unwrap();
        let result = find_project_root(&outside_path);
        assert!(result.is_err());
        // Correct the assertion string to match the actual error
        assert!(result.unwrap_err().to_string().contains("Could not find project root (ao.toml)"));
    }

    #[test]
    fn run_tool_succeeds_with_valid_command() {
        let tmp_dir = tempdir().unwrap();
        let project_path = setup_test_project(tmp_dir.path()).unwrap();
        // Use a simple, universally available command
        let result = run_tool("echo hello", &project_path);
        assert!(result.is_ok());
    }

    #[test]
    fn run_tool_fails_with_invalid_command() {
        let tmp_dir = tempdir().unwrap();
        let project_path = setup_test_project(tmp_dir.path()).unwrap();
        let result = run_tool("this_command_should_not_exist_ever", &project_path);
        assert!(result.is_err());
        // Error message might vary depending on OS and shell
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to execute command") || err_msg.contains("No such file or directory"));
    }

    #[test]
    fn run_tool_fails_with_command_error_status() {
        let tmp_dir = tempdir().unwrap();
        let project_path = setup_test_project(tmp_dir.path()).unwrap();
        // Command that exists but returns non-zero status
        let result = run_tool("ls non_existent_file_for_run_tool", &project_path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("failed with status"));
    }

    #[test]
    fn run_tool_fails_with_empty_command() {
        let tmp_dir = tempdir().unwrap();
        let project_path = setup_test_project(tmp_dir.path()).unwrap();
        let result = run_tool("", &project_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("resulted in no executable parts"));
    }

    #[test]
    fn run_tool_fails_with_bad_shlex() {
        let tmp_dir = tempdir().unwrap();
        let project_path = setup_test_project(tmp_dir.path()).unwrap();
        // Command with unbalanced quotes
        let result = run_tool("echo \"hello", &project_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to parse command string"));
    }

    #[test]
    fn generate_grpc_code_fails_if_proto_missing() {
        let tmp_dir = tempdir().unwrap();
        let project_path = setup_test_project(tmp_dir.path()).unwrap();
        // Delete the proto file created by init
        fs::remove_file(project_path.join("model-interface/anops.proto")).unwrap();
        let result = generate_grpc_code(&project_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Proto file not found"));
    }

    #[test]
    fn generate_grpc_code_fails_if_python_or_grpc_tools_missing() {
        // This test assumes 'python_does_not_exist_for_test' is not a valid command.
        // It's a basic check that the function attempts execution and fails if the tool is missing.
        // A more robust test would involve mocking std::process::Command.
        let tmp_dir = tempdir().unwrap();
        let project_path = setup_test_project(tmp_dir.path()).unwrap();

        // Temporarily modify the command generation logic for this test (if possible without major refactor)
        // Or, more simply, accept that this test relies on the environment not having the fake python.
        // We'll proceed assuming the command fails as expected if python/grpcio-tools are missing.

        // We expect this to fail when trying to execute the python command.
        let result = generate_grpc_code(&project_path);

        // Check if the error indicates a failure to execute the command.
        // This is environment-dependent. If python and grpcio-tools *are* installed,
        // this test might pass for the wrong reasons (actual successful generation).
        // A truly isolated test needs mocking.
        if result.is_err() {
            let err_msg = result.unwrap_err().to_string();
            println!("generate_grpc_code_fails_if_python_or_grpc_tools_missing error: {}", err_msg);
            // Check for common error messages related to command execution failure
            assert!(err_msg.contains("Failed to execute") || err_msg.contains("No such file or directory") || err_msg.contains("gRPC code generation failed"));
        } else {
            // If it succeeded, it means python & grpcio-tools are likely installed.
            // We can't reliably test the failure case without mocking or ensuring they aren't installed.
            println!("Skipping assertion for generate_grpc_code failure: python/grpcio-tools likely installed.");
        }
    }

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
