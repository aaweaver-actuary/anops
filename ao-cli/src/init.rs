use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

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

const API_SERVICE_REQUIREMENTS: &str = r#"fastapi>=0.100.0,<1.0.0
uvicorn[standard]>=0.20.0,<1.0.0
grpcio>=1.50.0,<2.0.0
python-json-logger>=2.0.0,<3.0.0
"#;

const MODEL_SERVICE_REQUIREMENTS: &str = r#"grpcio>=1.50.0,<2.0.0
python-json-logger>=2.0.0,<3.0.0
# Add other model dependencies here, e.g.:
# pandas
# scikit-learn
"#;

const API_SERVICE_MAIN_PY: &str = r#"# Placeholder main.py for api-service
from fastapi import FastAPI

app = FastAPI()

@app.get("/health")
def health_check():
    return {"status": "ok"}

# TODO: Implement /predict endpoint
"#;

const MODEL_SERVICE_SERVER_PY: &str = r#"# Placeholder server.py for model-service
import time
from concurrent import futures
import grpc

# TODO: Import generated gRPC code
# import anops_pb2
# import anops_pb2_grpc

_ONE_DAY_IN_SECONDS = 60 * 60 * 24

# TODO: Implement the AnOpsServicer class
# class AnOpsServicer(anops_pb2_grpc.AnOpsServicer):
#     def Predict(self, request, context):
#         # Implement prediction logic here
#         return anops_pb2.PredictResponse(output_data=f"Processed: {request.input_data}")

def serve():
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    # TODO: Add servicer to server
    # anops_pb2_grpc.add_AnOpsServicer_to_server(AnOpsServicer(), server)
    print("Starting server. Listening on port 50051.")
    server.add_insecure_port("[::]:50051")
    server.start()
    try:
        while True:
            time.sleep(_ONE_DAY_IN_SECONDS)
    except KeyboardInterrupt:
        server.stop(0)

if __name__ == "__main__":
    serve()
"#;

const API_SERVICE_TEST_MAIN_PY: &str = r#"# Placeholder test_main.py for api-service
# TODO: Add tests using pytest and httpx

def test_placeholder():
    assert True
"#;

const MODEL_SERVICE_TEST_SERVER_PY: &str = r#"# Placeholder test_server.py for model-service
# TODO: Add tests using pytest and grpcio-testing

def test_placeholder():
    assert True
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
pub fn run(path_str: String) -> Result<()> {
    let project_path = PathBuf::from(path_str);
    info!("Initializing new project at: {}", project_path.display());

    // Extract the directory name to use as the default project name
    let project_name = project_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("anops-project"); // Default if path ends in .. or /

    // Create root project directory
    fs::create_dir_all(&project_path)
        .with_context(|| format!("Failed to create project directory: {}", project_path.display()))?;

    // Create ao.toml configuration file
    let config_path = project_path.join("ao.toml");
    let config_content = format!(
        r#"[project]
name = "{}"

[check]
linters = []
testers = [
    # Example: Add commands to run tests
    # "pytest api-service/tests",
    # "pytest model-service/tests",
]

# [tasks]
# Define custom tasks here, e.g.:
# build = ["echo Building project..."]
# deploy = ["echo Deploying project..."]
"#,
        project_name // Use the extracted project name
    );
    fs::write(&config_path, config_content)
        .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;
    info!("Created config file: {}", config_path.display());

    // Create service directories
    let services = ["api-service", "model-service", "model-interface"];
    for service in services.iter() {
        let service_path = project_path.join(service);
        fs::create_dir_all(&service_path)
            .with_context(|| format!("Failed to create directory: {}", service_path.display()))?;
        info!("Created directory: {}", service_path.display());

        // Add placeholder files/READMEs using the correct constant names
        match *service {
            "api-service" => {
                fs::write(service_path.join("README.md"), DEFAULT_API_README)?;
                fs::write(service_path.join("Dockerfile"), DEFAULT_API_DOCKERFILE)?;
                fs::write(service_path.join("requirements.txt"), API_SERVICE_REQUIREMENTS)?;
                fs::write(service_path.join("main.py"), API_SERVICE_MAIN_PY)?;
                // Create tests directory and placeholder test file
                let test_dir = service_path.join("tests");
                fs::create_dir_all(&test_dir)?;
                fs::write(test_dir.join("test_main.py"), API_SERVICE_TEST_MAIN_PY)?;
            }
            "model-service" => {
                fs::write(service_path.join("README.md"), DEFAULT_MODEL_README)?;
                fs::write(service_path.join("Dockerfile"), DEFAULT_MODEL_DOCKERFILE)?;
                fs::write(service_path.join("requirements.txt"), MODEL_SERVICE_REQUIREMENTS)?;
                fs::write(service_path.join("server.py"), MODEL_SERVICE_SERVER_PY)?;
                // Create tests directory and placeholder test file
                let test_dir = service_path.join("tests");
                fs::create_dir_all(&test_dir)?;
                fs::write(test_dir.join("test_server.py"), MODEL_SERVICE_TEST_SERVER_PY)?;
            }
            "model-interface" => {
                fs::write(service_path.join("README.md"), DEFAULT_INTERFACE_README)?;
                fs::write(service_path.join("anops.proto"), DEFAULT_ANOP_PROTO)?;
            }
            _ => {}
        }
    }

    // Create root README.md
    let readme_path = project_path.join("README.md");
    fs::write(&readme_path, "# AnOps Project\n\nThis is the root README for the AnOps project.")
        .with_context(|| format!("Failed to write README.md: {}", readme_path.display()))?;
    info!("Created README.md: {}", readme_path.display());

    // Create .gitignore
    let gitignore_path = project_path.join(".gitignore");
    fs::write(&gitignore_path, DEFAULT_GITIGNORE_CONTENT)
        .with_context(|| format!("Failed to write .gitignore: {}", gitignore_path.display()))?;
    info!("Created .gitignore: {}", gitignore_path.display());

    info!("Project '{}' initialized successfully.", project_name);
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
        let project_path = tmp_dir.path().join(project_name);
        // Run the init command relative to the temp dir
        let result = run(project_path.to_str().unwrap().to_string());
        assert!(result.is_ok(), "init::run failed: {:?}", result.err());

        // Check if base directory exists
        assert!(project_path.exists());
        assert!(project_path.is_dir());

        // Check if standard subdirectories exist (as created by the current init::run)
        let subdirs = ["api-service", "model-service", "model-interface", "api-service/tests", "model-service/tests"];
        for subdir in subdirs.iter() {
            let dir_path = project_path.join(subdir);
            assert!(dir_path.exists(), "Directory missing: {}", subdir);
            assert!(dir_path.is_dir(), "Path is not a directory: {}", subdir);
        }

        // Check if core files exist (as created by the current init::run)
        let core_files = [
            "ao.toml",
            ".gitignore",
            "README.md", // Root README
            "api-service/Dockerfile",
            "api-service/README.md",
            "api-service/requirements.txt",
            "api-service/main.py",
            "api-service/tests/test_main.py",
            "model-service/Dockerfile",
            "model-service/README.md",
            "model-service/requirements.txt",
            "model-service/server.py",
            "model-service/tests/test_server.py",
            "model-interface/anops.proto",
            "model-interface/README.md",
        ];
        for file in core_files.iter() {
            let file_path = project_path.join(file);
            assert!(file_path.exists(), "File missing: {}", file);
            assert!(file_path.is_file(), "Path is not a file: {}", file);
        }

        // Check if config file has the correct project name
        let config_path = project_path.join("ao.toml");
        let content = fs::read_to_string(config_path).unwrap();
        assert!(content.contains(&format!("name = \"{}\"", project_name)), "Project name mismatch in ao.toml");

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