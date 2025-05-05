# AnOps API Service

## Overview
The AnOps API Service is a RESTful API that provides access to the underlying `model-service` via a simple HTTP interface. It acts as a bridge between standard web requests and the gRPC-based `model-service`.

**Technology Choice:** This service is implemented using **Python** and the **FastAPI** framework. This choice was made for:
*   **Simplicity and Speed:** FastAPI allows for rapid development of robust APIs.
*   **Python Ecosystem:** Leverages the rich Python ecosystem, familiar to the target audience.
*   **Async Support:** Good support for asynchronous operations, beneficial for handling I/O like gRPC calls.
*   **Automatic Docs:** Built-in OpenAPI (Swagger) documentation generation.

## Functionality
*   Receives requests (e.g., JSON payloads) on defined REST endpoints (e.g., `/predict`).
*   Connects to the `model-service` using gRPC (acting as a gRPC client).
*   Forwards the request data to the `model-service` according to the `model-interface` definition.
*   Receives the response from the `model-service`.
*   Formats and returns the response to the original HTTP caller (e.g., as JSON).

## Running the Service
This service is designed to be run as a Docker container. See the `Dockerfile` and the root `docker-compose.yml`.