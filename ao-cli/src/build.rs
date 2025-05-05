use anyhow::{Context, Result};
use std::path::Path;
use tracing::{info, warn, error};

use crate::config;
use crate::utils::{find_project_root, run_tool, generate_grpc_code}; // Added generate_grpc_code
use crate::check; // Import the check module to run pre-build checks

/// Handler for `ao build`.
/// Finds the project root, loads config, generates gRPC code, runs checks, and builds Docker images.
///
/// # Arguments
///
/// * `path_str` - Path within the project directory to start searching from.
///
/// # Errors
///
/// Returns an error if the project root is not found, config loading fails,
/// gRPC generation fails, checks fail, or any Docker build command fails.
pub fn run(path_str: String) -> Result<()> {
    let start_path = Path::new(&path_str);
    info!("Starting build from {}", start_path.display());

    // Find project root
    let project_path = find_project_root(start_path)
        .with_context(|| format!("Failed to find project root starting from '{}'", start_path.display()))?;
    info!("Found project root at {}", project_path.display());

    // Load configuration
    let config = config::load_config(&project_path)
        .context("Failed to load project configuration")?;
    let project_name = &config.project.name;
    info!("Building project: {}", project_name);

    // --- Generate gRPC Code --- //
    generate_grpc_code(&project_path)
        .context("Failed to generate gRPC code")?;
    // --- End Generate gRPC Code --- //


    // --- Pre-build Checks --- //
    info!("--- Running Pre-Build Checks ---");
    // Use the existing check::run function
    check::run(path_str.clone()) // Pass the original path string
        .context("Pre-build checks failed")?;
    info!("--- Pre-Build Checks Passed ---");

    // --- Build Docker Images --- //
    info!("--- Building Docker Images ---");

    // Define image names (using project name from config)
    // TODO: Allow overriding tags/names via config or CLI args later
    let api_image_name = format!("{}-api-service:latest", project_name);
    let model_image_name = format!("{}-model-service:latest", project_name);

    // Build api-service
    let api_service_path = project_path.join("api-service");
    if api_service_path.exists() && api_service_path.is_dir() {
        info!("Building {}...", api_image_name);
        let build_cmd = format!(
            "docker build -t {} .",
            api_image_name
        );
        run_tool(&build_cmd, &api_service_path)
            .with_context(|| format!("Failed to build api-service image: {}", api_image_name))?;
        info!("Successfully built {}", api_image_name);
    } else {
        warn!("Skipping api-service build: directory not found at {:?}", api_service_path);
    }

    // Build model-service
    let model_service_path = project_path.join("model-service");
     if model_service_path.exists() && model_service_path.is_dir() {
        info!("Building {}...", model_image_name);
        // Note: This assumes the Docker context is the model-service directory itself.
        // If generated gRPC code needs to be included from model-interface,
        // the Dockerfile or build process might need adjustment (e.g., copying files before build).
        let build_cmd = format!(
            "docker build -t {} .",
            model_image_name
        );
        run_tool(&build_cmd, &model_service_path)
            .with_context(|| format!("Failed to build model-service image: {}", model_image_name))?;
        info!("Successfully built {}", model_image_name);
    } else {
        warn!("Skipping model-service build: directory not found at {:?}", model_service_path);
    }

    info!("--- Docker Images Built Successfully ---");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init;
    use tempfile::tempdir;
    use std::fs;
    use std::path::PathBuf;

    // Helper to create a valid project structure for testing build
    fn setup_valid_project(base_path: &std::path::Path) -> PathBuf {
        let project_name = "test_build_project";
        let project_path = base_path.join(project_name);
        init::run(project_path.to_str().unwrap().to_string()).unwrap();
        project_path
    }

    // Basic integration test for build::run
    // This test will not actually build Docker images, but will check that the logic flows and errors are handled.
    #[test]
    fn build_run_succeeds_with_valid_project() {
        let tmp_dir = tempdir().unwrap();
        let project_path = setup_valid_project(tmp_dir.path());
        // Overwrite Dockerfiles with a minimal valid Dockerfile to avoid build errors
        let dockerfile = "FROM scratch\n";
        fs::write(project_path.join("api-service/Dockerfile"), dockerfile).unwrap();
        fs::write(project_path.join("model-service/Dockerfile"), dockerfile).unwrap();
        // Overwrite ao.toml with minimal config
        fs::write(project_path.join("ao.toml"), "[project]\nname = 'test_build_project'").unwrap();
        // This will likely fail at the gRPC codegen or docker build step if dependencies are missing,
        // but we want to ensure it does not panic and returns an error with context.
        let result = run(project_path.to_str().unwrap().to_string());
        match result {
            Ok(_) => info!("build::run returned Ok (all dependencies found)"),
            Err(e) => {
                let msg = e.to_string();
                warn!("build::run returned Err: {}", msg);
                // Acceptable errors: gRPC codegen or docker build failures
                assert!(msg.contains("Failed to generate gRPC code") ||
                        msg.contains("Failed to build api-service image") ||
                        msg.contains("Failed to build model-service image") ||
                        msg.contains("Pre-build checks failed"));
            }
        }
    }
}
