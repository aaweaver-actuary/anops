# AO CLI - Project Action Plan

**Goal:** Develop the `ao` CLI into a functional Analytics Ops orchestrator as envisioned.

**Current Status (as of May 5, 2025):**
*   Basic CLI structure using `clap`.
*   `init` subcommand implemented with directory/file creation.
*   `check` subcommand implemented with structure validation and tool execution via config.
*   `run` subcommand implemented to execute tasks defined in config.
*   Configuration loading (`ao.toml`) implemented using `serde` and `toml`.
*   Helper functions for finding project root and running external tools.
*   Unit tests exist for `init`, `check`, `config`, and `run` modules.
*   Core dependencies: `clap`, `anyhow`, `toml`, `serde`.
*   Dev dependencies: `tempfile`.

**Phase 1: Core Feature Implementation (`init` and `check`)**

1.  **Define Project Structure:**
    *   [x] Decide on a default project structure `ao init` will create (e.g., directories for models, tests, configuration, notebooks). (Decision: `models`, `tests`, `notebooks`, `ao.toml`)
    *   [ ] Consider variations based on project type or language (using `opts::language`). Start with one simple template (e.g., a generic structure or Python-focused).
2.  **Implement `ao init`:**
    *   [x] Replace placeholder in `src/init.rs`.
    *   [x] Add logic to create the defined directory structure.
    *   [x] Add logic to create basic configuration files (e.g., a placeholder `ao.toml`).
    *   [x] Add basic unit tests for `init`.
3.  **Define `check` Scope:**
    *   [x] Determine *what* `ao check` should verify initially. Start simple, e.g., check for required files/directories defined by the template.
    *   [x] Plan integration points for actual linting and testing tools (e.g., how will `ao` know *which* linter/tester to run?). (Via `ao.toml` `[check]` section)
4.  **Implement `ao check` (Initial Version):**
    *   [x] Replace placeholder in `src/check.rs`.
    *   [x] Implement basic structure validation based on the defined template.
    *   [x] Add logic to locate the project root (using the `--path` argument or searching upwards for a marker file like `ao.toml`).
    *   [x] Add basic unit tests for `check`.
5.  **Refactor/Clarify `lint.rs`:**
    *   [x] Decide if `lint` is a separate command or part of `check`. (Decision: Part of `check`)
    *   [x] If separate, define its distinct purpose and implement basic logic (or remove if redundant). (Removed)
    *   [x] If part of `check`, remove `src/lint.rs` and integrate its intended logic into `src/check.rs`. (Removed `lint.rs`, integration done in Phase 2)

**Phase 2: Enhancing Core Features & Configuration**

1.  **Configuration:**
    *   [x] Design a configuration file format (e.g., `ao.toml`) for project-specific settings. (Initial design with `[project]` table)
    *   [x] Define settings for `check` (e.g., tools to run, paths to include/exclude). (Added `[check]` table with `linters`, `testers`)
    *   [x] Implement config file loading and parsing (consider crates like `serde` and `toml`). (Basic loading implemented)
    *   [x] Add unit tests for config loading. (Tests updated for `[check]` and `[tasks]` sections)
2.  **Enhance `ao check`:**
    *   [x] Integrate with at least one linter (e.g., invoke `ruff` for Python if found). (Mechanism implemented via `run_tool`)
    *   [x] Integrate with at least one testing tool (e.g., invoke `pytest` for Python if found). (Mechanism implemented via `run_tool`)
    *   [x] Use the configuration file to drive which tools are run. (Config is loaded and used to run tools)

**Phase 3: Orchestration & Advanced Features**

1.  **Define Orchestration Tasks:**
    *   [ ] Identify common workflows in analytics/modeling (e.g., `data-load -> transform -> train -> validate`).
    *   [x] Design how these workflows/tasks are defined (e.g., in the `ao.toml` file). (Added `[tasks]` table with `task_name = ["cmd1", "cmd2"]`)
2.  **Implement `ao run <task>`:**
    *   [x] Add a new `run` subcommand.
    *   [x] Implement logic to parse task definitions from the config.
    *   [x] Implement logic to execute the steps defined in a task (e.g., running shell commands, other `ao` commands). (Using `run_tool` helper)
    *   [ ] Consider task dependencies. (Not yet implemented)
3.  **Add More Subcommands (Potential):**
    *   [ ] `ao build`: For compiling models or assets.
    *   [ ] `ao deploy`: For deploying models or reports.
    *   [ ] `ao clean`: For removing build artifacts.
4.  **Plugin System (Optional/Future):**
    *   [ ] Design an architecture to allow extending `ao` with plugins for specific tools or platforms (dbt, specific ML frameworks, cloud providers).

**Phase 4: Polish & Release**

1.  **Error Handling & Logging:**
    *   [ ] Improve user-facing error messages. (Using `anyhow` provides context, but needs refinement)
    *   [ ] Add configurable logging levels (e.g., using `log` and `env_logger` crates). (Currently uses `println!`)
2.  **Documentation:**
    *   [ ] Write comprehensive README.
    *   [ ] Add usage examples.
    *   [ ] Document the configuration file format.
    *   [x] Generate CLI help text effectively (`clap` features).
3.  **Testing:**
    *   [ ] Add integration tests covering CLI commands and workflows.
    *   [ ] Ensure good unit test coverage. (Unit tests exist, but coverage not measured)
4.  **Packaging & Distribution:**
    *   [ ] Prepare for publishing to crates.io.
    *   [ ] Consider binary releases (e.g., using `cargo-dist` or GitHub Actions).

**Ongoing:**
*   Refactor code for clarity and maintainability. (Helper functions `find_project_root` and `run_tool` are duplicated in `check.rs` and `run.rs` - potential refactor target)
*   Keep dependencies updated.
*   Address bugs and user feedback.