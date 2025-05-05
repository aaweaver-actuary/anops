import grpc
from concurrent import futures
import time
import logging  # Added logging

# Import the generated classes
import anops_pb2
import anops_pb2_grpc

# Configure basic logging
logging.basicConfig(level=logging.INFO)


# --- Simple Model Logic --- #
def run_model(input_str: str) -> str:
    """Placeholder for actual model execution."""
    logging.info(f"Running model with input: '{input_str}'")
    # Example: Reverse the input string
    output_str = input_str[::-1]
    logging.info(f"Model output: '{output_str}'")
    return output_str


# ------------------------ #


# Implementation of the AnOps service
class AnOpsServicer(anops_pb2_grpc.AnOpsServicer):
    """Provides methods that implement functionality of the AnOps server."""

    def Predict(self, request, context):
        """Handles the Predict RPC call."""
        logging.info(f"Received Predict request with data: '{request.input_data}'")
        # --- Call the model logic --- #
        try:
            output = run_model(request.input_data)
            return anops_pb2.PredictResponse(output_data=output)
        except Exception as e:
            logging.error(f"Model execution failed: {e}", exc_info=True)
            context.set_code(grpc.StatusCode.INTERNAL)
            context.set_details(f"Model execution failed: {e}")
            return anops_pb2.PredictResponse()  # Return empty response on error
        # -------------------------- #


# Function to start the server
def serve():
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    anops_pb2_grpc.add_AnOpsServicer_to_server(AnOpsServicer(), server)

    port = "[::]:50051"  # Listen on all interfaces, port 50051
    server.add_insecure_port(port)

    print(f"Starting gRPC server on {port}")
    server.start()
    print("Server started.")

    # Keep the server running
    try:
        while True:
            time.sleep(86400)  # Keep alive for one day
    except KeyboardInterrupt:
        print("Stopping server...")
        server.stop(0)
        print("Server stopped.")


if __name__ == "__main__":
    # Note: Ensure you have generated the _pb2.py and _pb2_grpc.py files
    # using grpc_tools.protoc before running this server.
    # Example command (run from model-interface dir):
    # python -m grpc_tools.protoc -I. --python_out=../model-service --pyi_out=../model-service --grpc_python_out=../model-service anops.proto
    serve()
