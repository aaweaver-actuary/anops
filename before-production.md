# Before Production Checklist for AnOps

This checklist is for the final pre-production review of the AnOps project. It is based on a critical review of the codebase and aims to prevent common deployment failures and technical debt. All items must be checked off before the first production release.

## 1. Test Coverage
- [ ] **Rust CLI (`ao-cli`)**
  - [ ] Integration tests for `ao build` and `ao check` (verify Docker images and gRPC code are actually built)
  - [ ] Mock external commands in tests (e.g., using `assert_cmd` or similar)
- [ ] **Python Services**
  - [ ] `api-service`: Tests for startup/shutdown, environment variable handling, and malformed requests
  - [ ] `model-service`: Test for internal server error in gRPC, and for environment misconfiguration
- [ ] **End-to-End**
  - [ ] E2E tests: Add teardown/cleanup logic
  - [ ] E2E tests: Add negative-path tests (e.g., model-service down, invalid input)

## 2. Coding Best Practices
- [ ] **Logging**
  - [ ] Use structured logging in Rust (`log` crate)
  - [ ] Set log levels and formats in Python services
- [ ] **Error Handling**
  - [ ] Ensure all user-facing errors are actionable and clear
- [ ] **Security**
  - [ ] Review Dockerfiles for unnecessary packages/files

## 3. Project Organization
- [ ] **Documentation**
  - [ ] Ensure `ACTIONPLAN.md` is up to date and reflects the current state
  - [ ] Add developer onboarding section to `README.md` (how to run tests, what to expect)

## 4. Automation
- [ ] **CI/CD**
  - [ ] Ensure all tests (unit, integration, E2E) are run in CI
  - [ ] Ensure CI failures are visible and actionable

## 5. Outstanding TODOs in Code
- [ ] `ao-cli/src/build.rs`: Implement tests for build logic
- [ ] `ao-cli/src/utils.rs`: Add robust tests for `run_tool` and gRPC code generation
- [ ] `model-service/server.py`: Add test for internal server error
- [ ] `tests/test_e2e.py`: Add teardown and negative-path tests

---

**Sign-off:** All items above must be checked and signed off by the responsible engineer before production deployment.
