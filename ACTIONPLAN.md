# AnOps - Project Action Plan

**Overall Goal:** Develop AnOps into a simple, usable orchestration tool for deploying and managing ML models, primarily targeting users with strong statistical modeling skills but potentially less software engineering background.

**Current Status (as of May 5, 2025):**
*   Project structure defined with four main components: `ao-cli`, `api-service`, `model-service`, `model-interface`.
*   `ao-cli` (Rust):
    *   Basic CLI structure using `clap`.
    *   `init`, `check`, `run` subcommands implemented with basic functionality (structure creation, config loading, task execution).
    *   `init` now creates the full project structure including Dockerfiles, compose file, and gRPC proto.
    *   Configuration loading (`ao.toml`) implemented.
    *   Unit tests exist for core CLI modules.
    *   Detailed action plan exists within `ao-cli/ACTIONPLAN.md`.
*   `api-service`, `model-service`, `model-interface`: Basic structure, Dockerfiles, READMEs, and `.proto` file created by `ao init`. No functional code yet.

**Minimum Viable Product (MVP) / Prototype Goal:**
Achieve an end-to-end workflow where a user can:
1.  Initialize a new AnOps project (`ao init`).
2.  Define a simple Python-based model within the `model-service` structure.
3.  Define the gRPC contract in `model-interface`.
4.  Build Docker images for `api-service` and `model-service` (`ao build`).
5.  Run the `api-service` and `model-service` (e.g., using Docker Compose or manual Docker commands).
6.  Send a request to the `api-service` REST endpoint.
7.  Have the `api-service` forward the request via gRPC to the `model-service`.
8.  Have the `model-service` execute the simple Python model and return a result.
9.  Receive the result back through the `api-service`.
10. Use `ao check` to validate the basic project setup and configuration.

---

## Phased Development Plan (Prioritizing MVP)

**Phase 1: Foundation & Interface Definition (MVP)**

1.  **Define `model-interface` (gRPC):**
    *   [x] Create `.proto` file(s) defining the service(s) and messages for communication between `api-service` and `model-service`. Start with a simple request/response structure (e.g., `PredictRequest`, `PredictResponse`). (`model-interface/anops.proto` created by `ao init`)
    *   [ ] Generate gRPC code stubs for Python (`model-service`) and the chosen language for `api-service`. (Manual step for now, potentially automated by `ao build` or a dedicated command later)
    *   [x] Store `.proto` files in the `model-interface` directory.
    *   [x] Create a README in `model-interface` explaining the gRPC interface and how to use it. (`model-interface/README.md` created by `ao init`)
2.  **Choose `api-service` Language/Framework:**
    *   [x] Decide on the technology stack (e.g., Python/FastAPI, Go/Gin, Node.js/Express). (Decision: Python/FastAPI)
    *   [x] Consider ease of use. (FastAPI is user-friendly)
    *   [x] Consider gRPC integration. (Python has good gRPC support)
    *   [x] Consider performance and scalability. (FastAPI is performant)
    *   [x] Consider how to keep the `api-service` and `model-service` loosely coupled, allowing the `api-service` to be shipped separately from the `model-service`. (gRPC interface achieves this)
    *   [x] Document the decision in the `ao-service` README. (`api-service/README.md` updated)
3.  **Basic Project Structure Setup:**
    *   [x] Update `ao init` to create placeholder directories and files for `api-service`, `model-service`, and `model-interface` according to the chosen structure.
    *   [x] Add basic Dockerfiles for `api-service` and `model-service`. (Created by `ao init`)
    *   [x] Add a basic `docker-compose.yml` (or similar orchestration) for running the services together. (Created by `ao init`)
4.  **Update README files to reflect current state and decisions:**
    *   [x] Update root README.md (Implicitly updated by context, review recommended)
    *   [x] Update ao-cli/README.md (Implicitly updated by context, review recommended)
    *   [x] Update api-service/README.md (Created/Updated by `ao init`)
    *   [x] Update model-service/README.md (Created by `ao init`)
    *   [x] Update model-interface/README.md (Created by `ao init`)

**Phase 2: Core Service Implementation (Barebones MVP)**

1.  **Implement `model-service` (Barebones):**
    *   [x] Create a simple Python gRPC server based on the generated stubs. (`server.py` created)
    *   [x] Implement the `Predict` method (or equivalent) to initially just echo the request or return a fixed response.
    *   [ ] Ensure the model service uses ruff, pytest, and coverage for linting and testing. The project should fail if any of these tools fail, or if the test coverage is below 95%. (TODO: Add in Phase 3/5)
    *   [x] Ensure it can be built into a Docker image using the Dockerfile from Phase 1. (Dockerfile updated)
2.  **Implement `api-service` (Barebones):**
    *   [x] Create a simple REST API server (e.g., FastAPI) with one endpoint (e.g., `/predict`). (`main.py` created)
    *   [x] Implement the endpoint handler to:
        *   [x] Receive a JSON request.
        *   [x] Create a gRPC client connection to the `model-service`.
        *   [x] Send the request data via gRPC using the generated stubs.
        *   [x] Receive the gRPC response.
        *   [x] Return the response as JSON.
    *   [x] Ensure it can be built into a Docker image using the Dockerfile from Phase 1. (Dockerfile updated)

**Phase 3: CLI Integration & Build Process (MVP)**

1.  **Implement `ao build`:**
    *   [ ] Add the `build` subcommand to `ao-cli`.
    *   [ ] Implement logic to find the `api-service` and `model-service` directories.
    *   [ ] Implement logic to run `docker build` for both services using their respective Dockerfiles and context.
    *   [ ] Add configuration options in `ao.toml` if needed (e.g., image names, build args).
    *   [ ] Integrate basic pre-build checks (linting/testing) as described in the README, driven by `ao.toml` config.
2.  **Enhance `ao check`:**
    *   [x] Add checks for the existence of required files/directories for `api-service`, `model-service`, and `model-interface` (e.g., Dockerfiles, `.proto` file).
    *   [ ] Add check for the presence of gRPC generated code (optional but helpful).
3.  **Refactor Shared CLI Code:**
    *   [ ] Address the duplication of `find_project_root` and `run_tool` noted in `ao-cli/ACTIONPLAN.md`. Create a shared utility module (e.g., `ao-cli/src/utils.rs` or `ao-cli/src/project.rs`).

**Phase 4: End-to-End Test & Basic Model (MVP)**

1.  **Integrate Services:**
    *   [ ] Use `docker-compose up` (or equivalent) to run the built `api-service` and `model-service` containers.
    *   [ ] Verify network connectivity between the containers.
2.  **Manual End-to-End Test:**
    *   [ ] Send a request to the `api-service` endpoint (e.g., using `curl` or Postman).
    *   [ ] Verify that the request flows through the `model-service` and a response is returned successfully.
3.  **Implement Simple Python Model:**
    *   [ ] Replace the barebones `model-service` logic with a simple Python function (e.g., adds two numbers, basic scikit-learn model prediction).
    *   [ ] Ensure the model code is correctly packaged into the `model-service` Docker image.
    *   [ ] Update the gRPC server to call this function.
4.  **Re-test End-to-End Flow:**
    *   [ ] Rebuild (`ao build`) and re-run the services.
    *   [ ] Test the `/predict` endpoint again and verify the actual model logic is executed.

**Phase 5: Refinement & Feature Expansion (Post-MVP)**

1.  **Enhance `ao run`:**
    *   [ ] Add tasks to `ao.toml` for common workflows like running services (`docker-compose up`), stopping services, running tests within containers.
2.  **Implement `ao config`:**
    *   [ ] Add the `config` subcommand to display resolved configuration.
3.  **Add R Support:**
    *   [ ] Investigate and implement support for R-based models in `model-service` (e.g., using Plumber).
    *   [ ] Update `ao init` and `ao build` to handle R projects.
4.  **Improve Error Handling & Logging:**
    *   [ ] Implement structured logging in all services and the CLI.
    *   [ ] Improve user-facing error messages in the CLI.
5.  **Add Testing:**
    *   [ ] Implement unit and integration tests for `api-service` and `model-service`.
    *   [ ] Add end-to-end tests for the whole system.
6.  **Documentation:**
    *   [ ] Write comprehensive user guides and tutorials.
    *   [ ] Document the `ao.toml` configuration options thoroughly.
    *   [ ] Document the `model-interface` gRPC API.
7.  **Packaging & Distribution:**
    *   [ ] Prepare `ao-cli` for publishing to crates.io.
    *   [ ] Consider binary releases for `ao-cli`.
    *   [ ] Publish standard Docker images for `api-service` and `model-service` bases.

---

## Open Questions / Decisions

*   Language/Framework for `api-service`? (Needs decision in Phase 1)
*   Specific structure within `model-service` for Python/R code, dependencies, etc.?
*   How will secrets/credentials be managed (e.g., for Docker registries)?
*   Initial authentication/authorization strategy for the `api-service`? (Likely none for MVP)

---

## Future Considerations

*   Plugin system for `ao-cli`?
*   Support for other model types/languages?
*   Integration with cloud provider services (deployment, logging, monitoring)?
*   More sophisticated orchestration (e.g., multi-model support, versioning)?
*   UI/Dashboard?
