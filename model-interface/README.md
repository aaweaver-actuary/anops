# AnOps Model Interface (gRPC)

This directory contains the Protocol Buffer (`.proto`) definitions for the gRPC interface used for communication between the `api-service` and the `model-service`.

## Overview

The `api-service` acts as a gRPC client, sending requests to the `model-service`, which acts as a gRPC server implementing the defined service.

## Service Definition

See `anops.proto` for the formal service and message definitions.

## Generating Code Stubs

You will need the `protoc` compiler and the appropriate language-specific plugins (e.g., `grpcio-tools` for Python) to generate client and server code stubs from the `.proto` file.

**Example (Python):**

```bash
pip install grpcio grpcio-tools
python -m grpc_tools.protoc -I. --python_out=../model-service --pyi_out=../model-service --grpc_python_out=../model-service anops.proto
# Adjust output paths as needed
```

This generated code will be used by the `api-service` (client stubs) and `model-service` (server base classes and stubs).
