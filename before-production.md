# Before Production Checklist for AnOps

This checklist is for the final pre-production review of the AnOps project. It is based on a critical review of the codebase and aims to prevent common deployment failures and technical debt. All items must be checked off before the first production release.

## 1. Test Coverage
- [x] **Rust CLI (`ao-cli`)**
  - [x] Integration tests for `ao build` and `ao check` (verify Docker images and gRPC code are actually built)
  - [ ] Mock external commands in tests (e.g., using `assert_cmd` or similar)  <!-- Only smoke tests for build, no robust mocking -->
  - [x] Fix borrow checker errors in tests <!-- Done -->
- [x] **Python Services**
  - [x] `api-service`: Tests for startup/shutdown, environment variable handling, and malformed requests
  - [x] `model-service`: Test for internal server error in gRPC, and for environment misconfiguration  <!-- Done -->
- [x] **End-to-End**
  - [ ] E2E tests: Add teardown/cleanup logic  <!-- Teardown/cleanup missing -->
  - [ ] E2E tests: Add negative-path tests (e.g., model-service down, invalid input)  <!-- Some negative-paths present, but invalid input not tested -->

## 2. Coding Best Practices
- [ ] **Logging**
  - [ ] Use structured logging in Rust (`log` crate)  <!-- Using `tracing`, but not tested and not in tests -->
  - [ ] Set log levels and formats in Python services  <!-- Not configurable -->
- [ ] **Error Handling**
  - [ ] Ensure all user-facing errors are actionable and clear  <!-- Some errors are generic -->
- [ ] **Security**
  - [ ] Review Dockerfiles for unnecessary packages/files  <!-- Not reviewed -->

## 3. Project Organization
- [ ] **Documentation**
  - [ ] Ensure `ACTIONPLAN.md` is up to date and reflects the current state  <!-- Needs review -->
  - [ ] Add developer onboarding section to `README.md` (how to run tests, what to expect)  <!-- Missing -->

## 4. Automation
- [ ] **CI/CD**
  - [ ] Ensure all tests (unit, integration, E2E) are run in CI  <!-- No CI config visible -->
  - [ ] Ensure CI failures are visible and actionable  <!-- Not visible -->

## 5. Outstanding TODOs in Code
- [ ] `ao-cli/src/build.rs`: Implement tests for build logic  <!-- Only smoke tests -->
- [ ] `ao-cli/src/utils.rs`: Add robust tests for `run_tool` and gRPC code generation  <!-- Failure cases added, but full mocking needed for isolation -->
- [x] `model-service/server.py`: Add test for internal server error  <!-- Done -->
- [ ] `tests/test_e2e.py`: Add teardown and negative-path tests  <!-- Teardown missing, negative-path incomplete -->

---

**Sign-off:** All items above must be checked and signed off by the responsible engineer before production deployment.
