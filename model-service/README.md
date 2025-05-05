# AnOps Model Service

## Overview
The AnOps Model Service is responsible for hosting and executing the actual machine learning or statistical model. It exposes a gRPC interface defined in `../model-interface` and listens for requests from the `api-service`.

**Technology Choice:** This service is primarily intended to be implemented in **Python**, leveraging common data science libraries (e.g., scikit-learn, statsmodels, pandas, numpy) and the `grpcio` library for the gRPC server. Support for R (e.g., using Plumber with a gRPC bridge) is a future consideration.

## Functionality
*   Implements the gRPC server interface defined by `../model-interface/anops.proto`.
*   Loads the user-provided model code and any necessary artifacts (e.g., trained model files).
*   Receives prediction requests from the `api-service` via gRPC.
*   Processes the input data using the loaded model.
*   Returns the prediction results back to the `api-service` via gRPC.

## Running the Service
This service is designed to be run as a Docker container. See the `Dockerfile` and the root `docker-compose.yml`. The user's model code and dependencies will be packaged into this container image during the `ao build` process.
