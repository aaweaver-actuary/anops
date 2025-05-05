use anyhow::{Context, Result};
use std::path::Path;

use crate::config;
use crate::utils::{find_project_root, run_tool};
use crate::check; // Import the check module to run pre-build checks

/// Handler for `ao build`.
/// Finds the project root, loads config, runs checks, and builds Docker images.
///
/// # Arguments
///
/// * `path_str` - Path within the project directory to start searching from.
///
/// # Errors
///
/// Returns an error if the project root is not found, config loading fails,
/// checks fail, or any Docker build command fails.
pub fn run(path_str: String) -> Result<()> {
    let start_path = Path::new(&path_str);
    println!("Starting build from '{}'", start_path.display());

    // Find project root
    let project_path = find_project_root(start_path)
        .with_context(|| format!("Failed to find project root starting from '{}'", start_path.display()))?;
    println!("Found project root at '{}'", project_path.display());

    // Load configuration
    let config = config::load_config(&project_path)
        .context("Failed to load project configuration")?;
    let project_name = &config.project.name;
    println!("Building project: {}", project_name);

    // --- Pre-build Checks --- //
    println!("--- Running Pre-Build Checks ---");
    // Use the existing check::run function
    check::run(path_str.clone()) // Pass the original path string
        .context("Pre-build checks failed")?;
    println!("--- Pre-Build Checks Passed ---");

    // --- Build Docker Images --- //
    println!("--- Building Docker Images ---");

    // Define image names (using project name from config)
    // TODO: Allow overriding tags/names via config or CLI args later
    let api_image_name = format!("{}-api-service:latest", project_name);
    let model_image_name = format!("{}-model-service:latest", project_name);

    // Build api-service
    let api_service_path = project_path.join("api-service");
    if api_service_path.exists() && api_service_path.is_dir() {
        println!("Building {}...", api_image_name);
        let build_cmd = format!(
            "docker build -t {} .",
            api_image_name
        );
        run_tool(&build_cmd, &api_service_path)
            .with_context(|| format!("Failed to build api-service image: {}", api_image_name))?;
        println!("Successfully built {}", api_image_name);
    } else {
        println!("Skipping api-service build: directory not found at {:?}", api_service_path);
    }

    // Build model-service
    let model_service_path = project_path.join("model-service");
     if model_service_path.exists() && model_service_path.is_dir() {
        println!("Building {}...", model_image_name);
        // Note: This assumes the Docker context is the model-service directory itself.
        // If generated gRPC code needs to be included from model-interface,
        // the Dockerfile or build process might need adjustment (e.g., copying files before build).
        let build_cmd = format!(
            "docker build -t {} .",
            model_image_name
        );
        run_tool(&build_cmd, &model_service_path)
            .with_context(|| format!("Failed to build model-service image: {}", model_image_name))?;
        println!("Successfully built {}", model_image_name);
    } else {
        println!("Skipping model-service build: directory not found at {:?}", model_service_path);
    }

    println!("--- Docker Images Built Successfully ---");

    Ok(())
}

// TODO: Add tests for build::run
// - Test success case (mocks docker build or requires docker)
// - Test failure if check::run fails
// - Test failure if docker build fails
// - Test finding root from subdir
