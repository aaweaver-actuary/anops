# Contributing to AnOps

Thank you for your interest in contributing to AnOps!

## How to Contribute

1. **Fork the repository** and create a new branch for your feature or bugfix.
2. **Write clear, well-documented code** that follows the style of the project.
3. **Add or update tests** for your changes. All code must be covered by unit, integration, or end-to-end tests.
4. **Run `ao check`** to ensure all tests and linters pass before submitting a pull request.
5. **Document your changes** in the relevant README or code comments if needed.
6. **Submit a pull request** with a clear description of your changes and why they are needed.

## Code Style
- Python: Use [black](https://black.readthedocs.io/en/stable/) for formatting and [ruff](https://beta.ruff.rs/docs/) for linting.
- Rust: Use `cargo fmt` and `cargo clippy`.
- All code must be tested and pass CI before merging.

## Project Structure
- `ao-cli/`: Rust CLI tool
- `api-service/`: FastAPI REST API
- `model-service/`: Python gRPC model server
- `model-interface/`: gRPC proto definitions
- `tests/`: End-to-end tests

## Reporting Issues
- Please use GitHub Issues to report bugs or request features.

## Code of Conduct
- Be respectful and constructive. See [Contributor Covenant](https://www.contributor-covenant.org/) for guidance.
