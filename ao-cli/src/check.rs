use std::path::Path;

use anyhow::{bail, Context, Result};
use tracing::info;

use crate::config; // Import the config module
use crate::utils::{find_project_root, run_tool}; // Import from utils

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
    info!("Starting check from {}", start_path.display());

    // Find project root
    let project_path = find_project_root(start_path)
        .with_context(|| format!("Failed to find project root starting from '{}'", start_path.display()))?;
    info!("Found project root at {}", project_path.display());

    // Load configuration
    let config = config::load_config(&project_path)
        .context("Failed to load project configuration")?;
    info!("Project name from config: {}", config.project.name);

    // --- Structure Checks --- //
    info!("Running structure checks in '{}'", project_path.display());

    // Check for required directories relative to the found project root
    // TODO: Enhance check to include service directories // Updated below
    let required_dirs = [
        "api-service",
        "model-service",
        "model-interface",
        // Keep original basic checks? Decide if they are still relevant
        // "models",
        // "tests",
        // "notebooks",
    ];
    for dir_name in required_dirs.iter() {
        let dir_path = project_path.join(dir_name);
        if !dir_path.exists() {
            bail!(
                "Required directory '{}' not found in project root.",
                dir_path.display()
            );
        }
        if !dir_path.is_dir() {
            bail!("Path '{}' is not a directory.", dir_path.display());
        }
        info!("Found directory: {:?}", dir_path);
    }

    // Check for specific required files within service directories
    let required_files = [
        ("api-service", "Dockerfile"),
        ("api-service", "requirements.txt"), // Good practice to check
        ("api-service", "anops_pb2.py"), // Check for generated gRPC file
        ("api-service", "anops_pb2_grpc.py"), // Check for generated gRPC file
        ("model-service", "Dockerfile"),
        ("model-service", "requirements.txt"), // Good practice to check
        ("model-service", "anops_pb2.py"), // Check for generated gRPC file
        ("model-service", "anops_pb2_grpc.py"), // Check for generated gRPC file
        ("model-interface", "anops.proto"),
    ];
    for (dir_name, file_name) in required_files.iter() {
        let file_path = project_path.join(dir_name).join(file_name);
        if !file_path.exists() {
            bail!(
                "Required file '{}' not found in directory '{}'.",
                file_name,
                dir_name
            );
        }
        if !file_path.is_file() {
            bail!("Path '{}' is not a file.", file_path.display());
        }
        info!("Found file: {:?}", file_path);
    }

    // Config file presence is already checked by find_project_root and load_config
    info!("Found config file: {:?}", project_path.join("ao.toml"));

    // --- Tool Execution --- //

    // Run configured linters
    if !config.check.linters.is_empty() {
        info!("--- Running Linters ---");
        for linter_cmd in &config.check.linters {
            run_tool(linter_cmd, &project_path)
                .with_context(|| format!("Linter command '{}' failed", linter_cmd))?;
        }
        info!("--- Linters Finished ---");
    } else {
        info!("No linters configured.");
    }

    // Run configured testers
    if !config.check.testers.is_empty() {
        info!("--- Running Testers ---");
        for tester_cmd in &config.check.testers {
            run_tool(tester_cmd, &project_path)
                .with_context(|| format!("Tester command '{}' failed", tester_cmd))?;
        }
        info!("--- Testers Finished ---");
    } else {
        info!("No testers configured.");
    }

    info!("All checks passed successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init;
    use anyhow::{Context, Result};
    use std::fs;
    use std::path::{Path, PathBuf}; // Import PathBuf
    use tempfile::tempdir;

    // Helper to create a valid project structure for testing check
    // Note: init::run already creates the required service structure
    fn setup_valid_project(base_path: &Path) -> Result<PathBuf> {
        let project_name = "check_test_project";
        let project_path = base_path.join(project_name);
        // Use init::run to create the structure
        init::run(project_path.to_str().unwrap().to_string())
            .context("init::run failed during test setup")?;
        Ok(project_path)
    }

    // Helper to add a [check] section to ao.toml
    fn add_check_config(project_path: &Path) {
        let config_path = project_path.join("ao.toml");
        let mut content = fs::read_to_string(&config_path).unwrap();
        content.push_str(
            "\n[check]\nlinters = [\"echo Linter OK\"]\ntesters = [\"echo Tester OK\"]\n",
        );
        fs::write(config_path, content).unwrap();
    }

    #[test]
    fn find_project_root_finds_root() {
        let tmp_dir = tempdir().unwrap();
        let project_path = setup_valid_project(tmp_dir.path()).unwrap();
        // Ensure 'models' directory exists for the test
        let models_path = project_path.join("models");
        std::fs::create_dir_all(&models_path).unwrap();

        // Search from root
        let found_root = crate::utils::find_project_root(&project_path).unwrap();
        // Compare canonicalized paths for robustness
        let expected = project_path.canonicalize().unwrap();
        let actual = found_root.canonicalize().unwrap();
        assert_eq!(expected, actual);

        // Search from subdir
        let found_root_from_subdir = crate::utils::find_project_root(&models_path).unwrap();
        let actual_subdir = found_root_from_subdir.canonicalize().unwrap();
        assert_eq!(expected, actual_subdir);
    }

    #[test]
    fn find_project_root_fails_if_no_root() {
        let tmp_dir = tempdir().unwrap();
        let project_path = tmp_dir.path().join("no_config_project");
        fs::create_dir(&project_path).unwrap();

        let result = crate::utils::find_project_root(&project_path); // Use utils::find_project_root
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Could not find project root"));
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
        let models_path = project_path.join("models"); // 'models' dir is created by setup_valid_project via init::run
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
        let err_str = result.unwrap_err().to_string();
        assert!(err_str.contains("Failed to find project root") || err_str.contains("Failed to canonicalize"));
    }

    #[test]
    fn run_fails_if_no_project_found() {
        let tmp_dir = tempdir().unwrap();
        let empty_dir = tmp_dir.path().join("empty_dir");
        fs::create_dir(&empty_dir).unwrap();
        let result = run(empty_dir.to_str().unwrap().to_string());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("project root"));
    }

    #[test]
    fn run_fails_if_config_is_malformed() {
        let tmp_dir = tempdir().unwrap();
        let project_path = setup_valid_project(tmp_dir.path()).unwrap();
        // Overwrite with malformed config
        fs::write(project_path.join("ao.toml"), "[project]name=").unwrap();

        let result = run(project_path.to_str().unwrap().to_string());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("parse") && err.contains("config"));
    }

    #[test]
    fn run_fails_if_structure_invalid_even_if_found() {
        let tmp_dir = tempdir().unwrap();
        let project_path = setup_valid_project(tmp_dir.path()).unwrap();

        // Test removing a required service directory
        fs::remove_dir_all(project_path.join("api-service")).unwrap();
        let result_dir = run(project_path.to_str().unwrap().to_string());
        assert!(result_dir.is_err());
        let err_msg_dir = result_dir.unwrap_err().to_string();
        assert!(err_msg_dir.contains("Required directory") && err_msg_dir.contains("api-service"));

        // Recreate the project for the next check
        let project_path = setup_valid_project(tmp_dir.path()).unwrap(); // Re-init

        // Test removing a required file within a service directory (proto)
        fs::remove_file(project_path.join("model-interface/anops.proto")).unwrap();
        let result_file_proto = run(project_path.to_str().unwrap().to_string());
        assert!(result_file_proto.is_err());
        let err_msg_proto = result_file_proto.unwrap_err().to_string();
        assert!(err_msg_proto.contains("Required file") && err_msg_proto.contains("anops.proto") && err_msg_proto.contains("model-interface"));

        // Recreate the project for the next check
        let project_path = setup_valid_project(tmp_dir.path()).unwrap(); // Re-init

        // Test removing a generated gRPC file
        fs::remove_file(project_path.join("api-service/anops_pb2.py")).unwrap();
        let result_file_grpc = run(project_path.to_str().unwrap().to_string());
        assert!(result_file_grpc.is_err());
        let err_msg_grpc = result_file_grpc.unwrap_err().to_string();
        assert!(err_msg_grpc.contains("Required file") && err_msg_grpc.contains("anops_pb2.py") && err_msg_grpc.contains("api-service"));
    }
}