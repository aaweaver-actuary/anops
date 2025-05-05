# Use an official Python runtime as a parent image
FROM python:3.11-slim

# Set the working directory in the container
WORKDIR /app

# Install dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy the gRPC generated code and the service implementation
# Assumes generated code (_pb2.py, _pb2_grpc.py) is in the build context
COPY . .

# Expose the port the gRPC server listens on
EXPOSE 50051

# Command to run the gRPC server
CMD ["python", "server.py"]
