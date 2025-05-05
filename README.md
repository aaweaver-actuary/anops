# AnOps

AnOps is an orchestration tool for deploying and managing machine learning models in production. It is designed to simplify the process of deploying models by providing a consistent interface for all models, regardless of their underlying technology or implementation. AnOps is built on top of Docker and gRPC, making it easy to deploy and manage models in any environment that supports these technologies.

## Components

- **ao CLI**: Command-line tool for project management, validation, and automation.
- **api-service**: REST API (FastAPI) that exposes model predictions via HTTP and forwards requests to model-service via gRPC.
- **model-service**: Python (or R, future) service that implements the gRPC interface and runs the actual model logic.
- **model-interface**: Protocol Buffers definition for the gRPC contract between api-service and model-service.

## Quickstart

1. **Initialize a new project**
   ```sh
   ao init myproject
   cd myproject
   ```
2. **Implement your model** in `model-service/server.py` and update the gRPC interface in `model-interface/anops.proto` if needed.
3. **Build and generate code**
   ```sh
   ao build
   ```
   This will generate gRPC code and build Docker images for both services.
4. **Run the stack**
   ```sh
   docker-compose up --build
   ```
5. **Test the API**
   ```sh
   curl -X POST http://localhost:8000/predict -H 'Content-Type: application/json' -d '{"input_data": "hello world"}'
   ```

## Testing

- Run all tests (unit, integration, end-to-end):
  ```sh
  ao check
  ```
- Tests are located in `api-service/tests/`, `model-service/tests/`, and `tests/` (end-to-end).

## Best Practices

- All code is tested and linted before build and deploy.
- Docker images run as non-root users and only include necessary files.
- gRPC code generation is automated via `ao build`.
- Project structure and configuration are validated with `ao check`.

## Contributing

See `ACTIONPLAN.md` for the current development roadmap and checklist.

## License

MIT