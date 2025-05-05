use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tracing::{info, warn, error};

// Basic placeholder content for generated files
const DEFAULT_AO_TOML_CONTENT: &str = r#"[project]
name = "{}" # Replaced by project name

# Example [check] configuration (optional)
# [check]
# linters = ["ruff check ."]
# testers = ["pytest"]

# Example [tasks] configuration (optional)
# [tasks]
# build = ["echo Building..."]
"#;

const DEFAULT_GITIGNORE_CONTENT: &str = r#"# Python
__pycache__/
*.pyc
*.pyo
*.pyd
.Python
env/
venv/
ENV/
pip-log.txt
pip-delete-this-directory.txt
.tox/
.nox/
.coverage
.coverage.*
.cache
nosetests.xml
coverage.xml
*.cover
*.log
.hypothesis/
.pytest_cache/

# R (if used)
.Rproj.user/
.Rhistory
.RData
.Ruserdata

# Docker
docker-compose.override.yml

# AnOps specific
/api-service/app/__pycache__/
/model-service/app/__pycache__/
/model-service/generated/ # Example for generated gRPC code
"#;

const DEFAULT_API_DOCKERFILE: &str = r#"# Use an official Python runtime as a parent image
FROM python:3.11-slim

WORKDIR /app

# TODO: Add requirements.txt generation/copying
# COPY requirements.txt .
# RUN pip install --no-cache-dir -r requirements.txt

COPY . .

EXPOSE 8000

# TODO: Replace with actual command e.g., uvicorn main:app
CMD ["echo", "API Service Placeholder - Implement main:app and uncomment CMD"]
"#;

const DEFAULT_MODEL_DOCKERFILE: &str = r#"# Use an official Python runtime as a parent image
FROM python:3.11-slim

WORKDIR /app

# TODO: Add requirements.txt generation/copying (including grpcio, grpcio-tools)
# COPY requirements.txt .
# RUN pip install --no-cache-dir -r requirements.txt

# TODO: Copy generated gRPC code and model implementation
COPY . .

EXPOSE 50051

# TODO: Replace with actual command e.g., python server.py
CMD ["echo", "Model Service Placeholder - Implement server.py and uncomment CMD"]
"#;

const DEFAULT_DOCKER_COMPOSE: &str = r#"version: '3.8'

services:
  api-service:
    build: ./api-service
    ports:
      - "8000:8000"
    environment:
      MODEL_SERVICE_URL: model-service:50051
    depends_on:
      - model-service
    networks:
      - anops-net
    # Add volumes if needed for development hot-reloading
    # volumes:
    #   - ./api-service:/app

  model-service:
    build: ./model-service
    ports:
      - "50051:50051"
    networks:
      - anops-net
    # Add volumes if needed for development hot-reloading
    # volumes:
    #   - ./model-service:/app
    #   - ./model-interface:/app/interface # Example: mount proto if needed at runtime

networks:
  anops-net:
    driver: bridge
"#;

const DEFAULT_ANOP_PROTO: &str = r#"syntax = "proto3";

package anops;

// The AnOps service definition.
service AnOps {
  // Sends input data for prediction.
  rpc Predict (PredictRequest) returns (PredictResponse) {}
}

// The request message containing the input data.
message PredictRequest {
  string input_data = 1; // Placeholder: Use appropriate type (bytes, struct, etc.)
}

// The response message containing the prediction result.
message PredictResponse {
  string output_data = 1; // Placeholder: Use appropriate type
}
"#;

const DEFAULT_API_README: &str = r#"# AnOps API Service

## Overview
Acts as the RESTful entry point, receiving HTTP requests and communicating with the `model-service` via gRPC.

**Technology:** Python/FastAPI (default)

See root README and `ACTIONPLAN.md` for more details.
"#;

const DEFAULT_MODEL_README: &str = r#"# AnOps Model Service

## Overview
Hosts the actual model code and implements the gRPC server defined in `model-interface`.

**Technology:** Python (default), R (planned)

See root README and `ACTIONPLAN.md` for more details.
"#;

const DEFAULT_INTERFACE_README: &str = r#"# AnOps Model Interface (gRPC)

Contains the Protocol Buffer (`.proto`) definitions for the gRPC interface between `api-service` and `model-service`.

See `anops.proto` and the root README for more details.
"#;


/// Handler for `ao init`.
/// Creates the basic project directory structure and configuration file.
///
/// # Arguments
///
/// * `name` - The name of the project directory to initialize.
///
/// # Errors
///
/// Returns an error if initialization fails (e.g., directory creation, file creation).
pub fn run(name: String) -> Result<()> {
    let project_path = Path::new(&name);

    info!("Initializing project '{}' at {:?}", name, project_path);

    // Create base directory
    fs::create_dir_all(project_path)
        .with_context(|| format!("Failed to create project directory: {}", project_path.display()))?;
    info!("Created directory: {:?}", project_path);

    // Create standard subdirectories
    let subdirs = ["api-service", "model-service", "model-interface", "tests", "notebooks"];
    for subdir in subdirs.iter() {
        let dir_path = project_path.join(subdir);
        fs::create_dir_all(&dir_path)
            .with_context(|| format!("Failed to create subdirectory: {}", dir_path.display()))?;
        info!("Created directory: {:?}", dir_path);
    }

    // Create ao.toml configuration file
    let config_path = project_path.join("ao.toml");
    let config_content = DEFAULT_AO_TOML_CONTENT.replace("{}", &name);
    fs::write(&config_path, config_content)
        .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;
    info!("Created config file: {:?}", config_path);

    // Create .gitignore
    let gitignore_path = project_path.join(".gitignore");
    fs::write(&gitignore_path, DEFAULT_GITIGNORE_CONTENT)
        .with_context(|| format!("Failed to write .gitignore file: {}", gitignore_path.display()))?;
    info!("Created file: {:?}", gitignore_path);

    // Create Dockerfiles
    let api_dockerfile_path = project_path.join("api-service/Dockerfile");
    fs::write(&api_dockerfile_path, DEFAULT_API_DOCKERFILE)
        .with_context(|| format!("Failed to write api-service Dockerfile: {}", api_dockerfile_path.display()))?;
    info!("Created file: {:?}", api_dockerfile_path);

    let model_dockerfile_path = project_path.join("model-service/Dockerfile");
    fs::write(&model_dockerfile_path, DEFAULT_MODEL_DOCKERFILE)
        .with_context(|| format!("Failed to write model-service Dockerfile: {}", model_dockerfile_path.display()))?;
    info!("Created file: {:?}", model_dockerfile_path);

    // Create docker-compose.yml
    let compose_path = project_path.join("docker-compose.yml");
    fs::write(&compose_path, DEFAULT_DOCKER_COMPOSE)
        .with_context(|| format!("Failed to write docker-compose.yml: {}", compose_path.display()))?;
    info!("Created file: {:?}", compose_path);

    // Create model-interface proto file
    let proto_path = project_path.join("model-interface/anops.proto");
    fs::write(&proto_path, DEFAULT_ANOP_PROTO)
        .with_context(|| format!("Failed to write anops.proto: {}", proto_path.display()))?;
    info!("Created file: {:?}", proto_path);

    // Create READMEs
    let api_readme_path = project_path.join("api-service/README.md");
     fs::write(&api_readme_path, DEFAULT_API_README)
        .with_context(|| format!("Failed to write api-service README: {}", api_readme_path.display()))?;
    info!("Created file: {:?}", api_readme_path);

    let model_readme_path = project_path.join("model-service/README.md");
     fs::write(&model_readme_path, DEFAULT_MODEL_README)
        .with_context(|| format!("Failed to write model-service README: {}", model_readme_path.display()))?;
    info!("Created file: {:?}", model_readme_path);

    let interface_readme_path = project_path.join("model-interface/README.md");
     fs::write(&interface_readme_path, DEFAULT_INTERFACE_README)
        .with_context(|| format!("Failed to write model-interface README: {}", interface_readme_path.display()))?;
    info!("Created file: {:?}", interface_readme_path);

    // Create placeholder files in services (optional, but good practice)
    // e.g., api-service/main.py, model-service/server.py
    // fs::write(project_path.join("api-service/main.py"), "# FastAPI app placeholder")?;
    // fs::write(project_path.join("model-service/server.py"), "# gRPC server placeholder")?;

    info!("Project '{}' initialized successfully.", name);
    info!("Next steps for '{}':", name);
    info!("  - cd {}", name);
    info!("  - Review READMEs in api-service, model-service, model-interface.");
    info!("  - Implement your model in model-service.");
    info!("  - Implement the API endpoints in api-service.");
    info!("  - Generate gRPC code (see model-interface/README.md).");
    info!("  - Configure dependencies (e.g., requirements.txt).");
    info!("  - Run 'ao build' to build the service images.");
    info!("  - Run 'docker-compose up' to start the services.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn run_succeeds_and_creates_structure() {
        let tmp_dir = tempdir().unwrap();
        let project_name = "test_init_project";
        // Run the init command relative to the temp dir
        let result = run(tmp_dir.path().join(project_name).to_str().unwrap().to_string());
        assert!(result.is_ok());

        let project_path = tmp_dir.path().join(project_name);

        // Check if base directory exists
        assert!(project_path.exists());
        assert!(project_path.is_dir());

        // Check if standard subdirectories exist
        let subdirs = ["api-service", "model-service", "model-interface", "tests", "notebooks"];
        for subdir in subdirs.iter() {
            let dir_path = project_path.join(subdir);
            assert!(dir_path.exists(), "Directory missing: {}", subdir);
            assert!(dir_path.is_dir(), "Path is not a directory: {}", subdir);
        }

        // Check if core files exist
        let core_files = [
            "ao.toml",
            ".gitignore",
            "docker-compose.yml",
            "api-service/Dockerfile",
            "api-service/README.md",
            "model-service/Dockerfile",
            "model-service/README.md",
            "model-interface/anops.proto",
            "model-interface/README.md",
        ];
        for file in core_files.iter() {
            let file_path = project_path.join(file);
            assert!(file_path.exists(), "File missing: {}", file);
            assert!(file_path.is_file(), "Path is not a file: {}", file);
        }

        // Check if config file has basic content (loosened: just check for [project] and project_name)
        let config_path = project_path.join("ao.toml");
        let content = fs::read_to_string(config_path).unwrap();
        assert!(content.contains("[project]"));
        assert!(content.contains(project_name));

        // Check .gitignore content (basic check)
        let gitignore_path = project_path.join(".gitignore");
        let gitignore_content = fs::read_to_string(gitignore_path).unwrap();
        assert!(gitignore_content.contains("__pycache__/"));
        assert!(gitignore_content.contains("*.pyc"));
        // Clean up is handled by tempdir dropping
    }

    #[test]
    fn run_fails_if_cannot_create_dir() {
        // This test is tricky because permissions are hard to simulate reliably
        // across platforms in a unit test without running as root or modifying
        // system state. We rely on the OS preventing creation in restricted areas.
        // Trying to create directly in `/` (on Unix-like systems) might fail
        // without root privileges.
        if cfg!(unix) {
             let project_name = "/ao_init_fail_test";
             // Attempt to run the init command in a restricted path
             let result = run(project_name.to_string());
             // We expect this to fail, likely with a permission error context.
             assert!(result.is_err());
             assert!(result.unwrap_err().to_string().contains("Failed to create project directory"));
        } else {
            // Skip this specific scenario on non-Unix platforms where
            // root directory permissions might behave differently.
            println!("Skipping root directory creation test on non-Unix platform.");
        }
    }
}