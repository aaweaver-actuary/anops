# AnOps

`AnOps` is an orchestration tool for deploying and managing machine learning models in production. It is designed to simplify the process of deploying models by providing a consistent interface for all models, regardless of their underlying technology or implementation. `AnOps` is built on top of Docker and gRPC, making it easy to deploy and manage models in any environment that supports these technologies.

`AnOps` is specifically designed to be as simple to use as possible, and usable for statistical modelling experts who may not have a strong background in software engineering. It is intended to be used by data scientists, analysts, and other professionals who need to deploy and manage machine learning models in production.

`AnOps` is designed to be extensible and flexible, allowing users to easily add new models or modify existing ones. It is also designed to be easy to use, with a simple command-line interface and a RESTful API that provides access to all of the functionality of the tool.

`AnOps` consists of four components:
1. `ao` CLI
2. `api-service` (REST API)
3. `model-service` (Python- or R-based)
4. `model-interface` (gRPC interface connecting `api-service` and `model-service`)


## `ao` CLI
The `ao` CLI is a convenience tool for managing an AnOps modeling project. It has the following subcommands:
* `init`: Initializes a new AnOps project by creating the necessary directory structure and configuration files, linking the user's local model to the `model-service`, and setting up the `api-service` to use the model.
* `check`: Validates the project structure and configuration files, ensuring that all required files and directories are present and correctly configured. If the model is python-based, it also runs `ruff` and `pytest` to check for linting errors and run unit tests.
* `run`: Executes a task defined in the configuration file, which can include running shell commands or other `ao` commands. This is useful for orchestrating complex workflows in an analytics or modeling project.
* `build`: Builds the `model-service` Docker image using the provided Dockerfile and configuration files. After building, it will lint and run tests/coverage on the model code one last time before pushing the image to a Docker registry.
* `config`: Displays the current configuration settings for the project, including the project name, model type, and any other relevant settings. This is useful for verifying that the project is set up correctly and that all required settings are in place.

## `api-service`
The `api-service` is a RESTful API that provides access to a single endpoint for using models that fulfill the `model-interface` in production. It is implemented as a Docker container so that it can be deployed in any environment that supports Docker. The API is designed to be simple and easy to use, with a focus on providing a consistent interface for all models.

## `model-service`
The `model-service` is a Docker container that runs a model in production. It is designed to be used with the `model-interface` and can be deployed in any environment that supports Docker. The `model-service` is responsible for running the model and providing predictions to the `api-service`.

## `model-interface`
The `model-interface` is a gRPC interface that connects the `api-service` and the `model-service`. It is designed to be simple and easy to use, with a focus on providing a consistent interface for all models. The `model-interface` is responsible for handling requests from the `api-service` and forwarding them to the `model-service`.