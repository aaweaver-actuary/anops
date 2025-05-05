# Use an official Python runtime as a parent image
FROM python:3.11-slim

# Create a non-root user
RUN useradd -m appuser

# Set the working directory in the container
WORKDIR /app

# Install dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy only necessary files
COPY server.py .
COPY anops_pb2.py .
COPY anops_pb2_grpc.py .
COPY anops_pb2.pyi .

# Switch to non-root user
USER appuser

# Expose the port the gRPC server listens on
EXPOSE 50051

# Command to run the gRPC server
CMD ["python", "server.py"]
