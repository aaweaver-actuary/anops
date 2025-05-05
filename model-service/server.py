import grpc
from concurrent import futures
import time
import logging
import os  # Added os

# Import the generated classes
import anops_pb2
import anops_pb2_grpc

# Configure basic logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)


# --- Simple Model Logic --- #
def load_model_resource():
    """Placeholder for loading any model artifacts (files, connections, etc.)."""
    # Example: Load a prefix from an environment variable or a file
    prefix = os.getenv("MODEL_PREFIX", "MODEL_OUTPUT:")
    logger.info(f"Model resource loaded. Using prefix: '{prefix}'")
    return prefix


# Load resource once when the server starts (or lazily on first request)
MODEL_PREFIX = load_model_resource()


def run_model(input_str: str) -> str:
    """Placeholder for actual model execution."""
    logger.info(f"Running model with input: '{input_str}'")
    if not input_str:
        raise ValueError("Input data cannot be empty.")
    # Example: Add a prefix and convert to uppercase
    output_str = f"{MODEL_PREFIX} {input_str.upper()}"
    logger.info(f"Model output: '{output_str}'")
    return output_str


# ------------------------ #


# Implementation of the AnOps service
class AnOpsServicer(anops_pb2_grpc.AnOpsServicer):
    """Provides methods that implement functionality of the AnOps server."""

    def Predict(self, request, context):
        """Handles the Predict RPC call."""
        logger.info(f"Received Predict request with data: '{request.input_data}'")
        # --- Call the model logic --- #
        try:
            output = run_model(request.input_data)
            return anops_pb2.PredictResponse(output_data=output)
        except ValueError as ve:
            logger.warning(f"Invalid input data: {ve}")
            context.set_code(grpc.StatusCode.INVALID_ARGUMENT)
            context.set_details(f"Invalid input: {ve}")
            return anops_pb2.PredictResponse()  # Return empty response on error
        except Exception as e:
            logger.error(f"Model execution failed: {e}", exc_info=True)  # Log traceback
            context.set_code(grpc.StatusCode.INTERNAL)
            context.set_details(f"Internal model execution error: {e}")
            return anops_pb2.PredictResponse()  # Return empty response on error
        # -------------------------- #


# Function to start the server
def serve():
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    anops_pb2_grpc.add_AnOpsServicer_to_server(AnOpsServicer(), server)

    port = "[::]:50051"  # Listen on all interfaces, port 50051
    server.add_insecure_port(port)

    logger.info(f"Starting gRPC server on {port}")
    server.start()
    logger.info("Server started. Waiting for requests...")

    # Keep the server running
    try:
        while True:
            time.sleep(86400)  # Keep alive for one day
    except KeyboardInterrupt:
        logger.info("Stopping server...")
        server.stop(0)  # Graceful stop with no grace period
        logger.info("Server stopped.")


if __name__ == "__main__":
    serve()

# TODO: Add unit tests for the model logic (run_model, load_model_resource).
# TODO: Add integration tests for the gRPC server (AnOpsServicer.Predict).
