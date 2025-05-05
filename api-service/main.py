import os
import grpc
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
import logging

# Import the generated gRPC classes
# Assumes the generated files (_pb2.py, _pb2_grpc.py) are accessible.
# This might require adjusting PYTHONPATH or copying files during build.
# For simplicity, we'll assume they are in the same directory or PYTHONPATH.
import anops_pb2
import anops_pb2_grpc

# Configure logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)

app = FastAPI()

# Get the model service URL from environment variable
MODEL_SERVICE_URL = os.getenv("MODEL_SERVICE_URL", "localhost:50051")


# Pydantic model for request body
class PredictRequestData(BaseModel):
    input_data: str


# Pydantic model for response body
class PredictResponseData(BaseModel):
    output_data: str


@app.post("/predict", response_model=PredictResponseData)
async def predict(request_data: PredictRequestData):
    logger.info(f"Received /predict request: {request_data.input_data}")
    try:
        # Establish insecure gRPC channel to the model service
        # In production, use secure channels (grpc.secure_channel)
        async with grpc.aio.insecure_channel(MODEL_SERVICE_URL) as channel:
            stub = anops_pb2_grpc.AnOpsStub(channel)
            logger.info(f"Sending request to model service at {MODEL_SERVICE_URL}")

            # Create the gRPC request message
            grpc_request = anops_pb2.PredictRequest(input_data=request_data.input_data)

            # Make the asynchronous gRPC call
            grpc_response = await stub.Predict(grpc_request)

            logger.info(
                f"Received response from model service: {grpc_response.output_data}"
            )

            # Return the response data
            return PredictResponseData(output_data=grpc_response.output_data)

    except grpc.aio.AioRpcError as e:
        logger.error(f"gRPC error: {e}", exc_info=True)
        if e.code() == grpc.StatusCode.INVALID_ARGUMENT:
            raise HTTPException(status_code=400, detail=f"Invalid input: {e.details()}")
        elif e.code() == grpc.StatusCode.UNAVAILABLE:
            raise HTTPException(status_code=503, detail="Model service unavailable.")
        else:
            raise HTTPException(status_code=500, detail=f"gRPC error: {e.details()}")
    except Exception as e:
        logger.error(f"Unexpected error: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=f"Internal server error: {str(e)}")


@app.get("/health")
async def health_check():
    # Basic health check endpoint
    return {"status": "ok"}


if __name__ == "__main__":
    # This block is mainly for local development testing if needed,
    # Uvicorn will run the app in the Docker container.
    import uvicorn

    uvicorn.run(app, host="0.0.0.0", port=8000)
