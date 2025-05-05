# AnOps - Project Action Plan

**Overall Goal:** Develop AnOps into a simple, usable orchestration tool for deploying and managing ML models, primarily targeting users with strong statistical modeling skills but potentially less software engineering background.

**Current Status (as of May 5, 2025):**
*   Project structure defined with four main components: `ao-cli`, `api-service`, `model-service`, `model-interface`.
*   `ao-cli` (Rust): Basic commands (`init`, `check`, `build`, `run`) exist but lack comprehensive testing and robustness in some areas (e.g., `run_tool`).
*   `api-service`, `model-service`: Basic structure, Dockerfiles, READMEs, and placeholder code exist. gRPC code is manually generated and copied. **Testing, robust error handling, and logging are missing.**
*   `model-interface`: Basic `.proto` file exists.

**Minimum Viable Product (MVP) / Prototype Goal:**
Achieve a **robust and tested** end-to-end workflow where a user can:
1.  Initialize a new AnOps project (`ao init`).
2.  Define a simple Python-based model within the `model-service` structure.
3.  Define the gRPC contract in `model-interface`.
4.  **Automatically generate gRPC code** and build Docker images for `api-service` and `model-service` (`ao build`).
5.  Run the `api-service` and `model-service` (e.g., using Docker Compose).
6.  Send a request to the `api-service` REST endpoint.
7.  Have the `api-service` forward the request via gRPC to the `model-service` **with proper error handling**.
8.  Have the `model-service` execute the simple Python model and return a result **with proper error handling**.
9.  Receive the result back through the `api-service`.
10. Use `ao check` to validate the project setup, configuration, and **run basic checks/tests**.

---

## Detailed Development Checklist (Prioritizing MVP & Critical Fixes)

**Phase 1: Foundation & Interface Definition**

1.  **Define `model-interface` (gRPC):**
    *   [x] Create `.proto` file (`model-interface/anops.proto`).
    *   [x] Create README (`model-interface/README.md`).
    *   [x] **Automate gRPC Code Generation (Critical - High Priority):**
        *   [x] Implement logic in `ao-cli` (Rust) to generate code on `ao build`.
        *   [x] Update `model-interface/README.md` to document automation.
        *   [x] Add tests for gRPC code generation logic.
2.  **Choose `api-service` Language/Framework:**
    *   [x] Decision: Python/FastAPI.
    *   [x] Documented in `api-service/README.md`.
3.  **Basic Project Structure Setup:**
    *   [x] `ao init` creates directories and placeholder files.
    *   [x] Basic Dockerfiles exist (now refined for best practices).
    *   [x] Basic `docker-compose.yml` exists.
4.  **Update README files:**
    *   [x] Initial READMEs created by `ao init`. (Reviewed and updated.)

**Phase 2: Core Service Implementation & Testing**

1.  **Implement `model-service` (Python):**
    *   [x] gRPC server structure (`server.py`) exists.
    *   [x] Simple model logic and error handling implemented.
    *   [x] Logging uses Python's `logging` module.
    *   [x] Unit and integration tests in `model-service/tests/`.
    *   [x] Dockerfile copies only necessary files and runs as non-root user.
2.  **Implement `api-service` (Python/FastAPI):**
    *   [x] FastAPI app with `/predict` and `/health` endpoints.
    *   [x] gRPC client call structure.
    *   [x] Robust error handling and logging.
    *   [x] Unit and integration tests in `api-service/tests/`.
    *   [x] Dockerfile copies only necessary files and runs as non-root user.

**Phase 3: CLI Integration & Build Process**

1.  **Enhance `ao build` (Rust):**
    *   [x] Runs gRPC code generation before Docker build.
    *   [x] Runs pre-build checks and fails on errors.
    *   [x] Tests for build logic.
2.  **Enhance `ao check` (Rust):**
    *   [x] Validates structure, config, and required files.
    *   [x] Runs all tests (unit, integration, e2e) via pytest in each service and root tests/.
    *   [x] Tests for check logic.
3.  **Refactor & Strengthen Shared CLI Code (Rust):**
    *   [x] Shared utilities in `utils.rs`.
    *   [x] Strengthen `run_tool` parsing with shlex.
    *   [x] Tests for shared utilities.

**Phase 4: End-to-End Test & Integration**

1.  **Integrate Services:**
    *   [x] `docker-compose.yml` runs both services and networks them.
    *   [x] End-to-end test in `tests/test_e2e.py` verifies the full stack.
    *   [x] All tests are pytest-discoverable and run with `ao check`.

**Phase 5: Refinement & Feature Expansion**

*   [x] Add more documentation (contribution guide, advanced usage).
*   [x] Add pre-commit hooks or CI config for automated checks.
*   [ ] Add R support in model-service (future).
*   [ ] Expand test coverage and add more E2E scenarios.
*   [ ] Polish user/developer experience.

---

**All core MVP and best-practice items are now complete.**

---

## Open Questions / Decisions

*   How to manage Python dev dependencies (`requirements-dev.txt`?) vs runtime (`requirements.txt`)?
*   Specific structure within `model-service` for user model code vs. server code?
*   Secrets management (Post-MVP).
*   Authentication/Authorization (Post-MVP).

---

## Future Considerations

*   Plugin system for `ao-cli`?
*   Support for other model types/languages?
*   Cloud provider integrations?
*   Multi-model support, versioning?
*   UI/Dashboard?
