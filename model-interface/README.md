# AnOps Model Interface (gRPC)

This directory contains the Protocol Buffer (`.proto`) definitions for the gRPC interface used for communication between the `api-service` and the `model-service`.

## Overview

The `api-service` acts as a gRPC client, sending requests to the `model-service`, which acts as a gRPC server implementing the defined service.

## Service Definition

See `anops.proto` for the formal service and message definitions.

## Generating Code Stubs

The necessary Python client and server code stubs (`*_pb2.py`, `*_pb2_grpc.py`, `*_pb2.pyi`) are **automatically generated** when you run the `ao build` command.

The generated files are placed into both the `api-service` and `model-service` directories. You do not need to run `protoc` manually. Ensure you have `grpcio-tools` installed in the Python environment where you run `ao build` (or ensure `python -m grpc_tools.protoc` is runnable).
