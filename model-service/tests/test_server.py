import pytest
import grpc
import os
from concurrent import futures
from unittest import mock

# Import the modules to be tested
import server
import anops_pb2
import anops_pb2_grpc

# --- Unit Tests for Model Logic --- #


def test_load_model_resource_default():
    """Test loading the default prefix."""
    # Unset env var if it exists to test default
    if "MODEL_PREFIX" in os.environ:
        del os.environ["MODEL_PREFIX"]
    prefix = server.load_model_resource()
    assert prefix == "MODEL_OUTPUT:"


def test_load_model_resource_env_var():
    """Test loading the prefix from environment variable."""
    test_prefix = "TEST_PREFIX:"
    os.environ["MODEL_PREFIX"] = test_prefix
    prefix = server.load_model_resource()
    assert prefix == test_prefix
    del os.environ["MODEL_PREFIX"]  # Clean up


def test_load_model_resource_empty_env():
    """Test loading the prefix when environment variable is empty."""
    os.environ["MODEL_PREFIX"] = ""
    prefix = server.load_model_resource()
    assert prefix == ""
    del os.environ["MODEL_PREFIX"]  # Clean up


def test_run_model_success():
    """Test successful model execution."""
    # Ensure default prefix is used for consistency
    if "MODEL_PREFIX" in os.environ:
        del os.environ["MODEL_PREFIX"]
    server.MODEL_PREFIX = server.load_model_resource()  # Reload prefix

    input_data = "hello world"
    expected_output = "MODEL_OUTPUT: HELLO WORLD"
    output = server.run_model(input_data)
    assert output == expected_output


def test_run_model_empty_input():
    """Test model execution with empty input."""
    with pytest.raises(ValueError, match="Input data cannot be empty."):
        server.run_model("")


# --- Integration Tests for gRPC Servicer --- #


# Fixture to set up and tear down the gRPC server for testing
@pytest.fixture(scope="module")
def grpc_server():
    test_server = grpc.server(futures.ThreadPoolExecutor(max_workers=1))
    anops_pb2_grpc.add_AnOpsServicer_to_server(server.AnOpsServicer(), test_server)
    port = test_server.add_insecure_port("[::]:0")  # Use random available port
    test_server.start()
    yield f"localhost:{port}"  # Provide the server address to tests
    test_server.stop(0)


def test_predict_success(grpc_server):
    """Test the Predict RPC call with valid input."""
    with grpc.insecure_channel(grpc_server) as channel:
        stub = anops_pb2_grpc.AnOpsStub(channel)
        request = anops_pb2.PredictRequest(input_data="test input")
        response = stub.Predict(request)

        # Assuming default prefix MODEL_OUTPUT:
        assert response.output_data == "MODEL_OUTPUT: TEST INPUT"


def test_predict_invalid_argument(grpc_server):
    """Test the Predict RPC call with invalid (empty) input."""
    with grpc.insecure_channel(grpc_server) as channel:
        stub = anops_pb2_grpc.AnOpsStub(channel)
        request = anops_pb2.PredictRequest(input_data="")

        with pytest.raises(grpc.RpcError) as rpc_error:
            stub.Predict(request)

        assert rpc_error.value.code() == grpc.StatusCode.INVALID_ARGUMENT
        assert "Invalid input: Input data cannot be empty." in rpc_error.value.details()


def test_predict_internal_error(grpc_server):
    """Test the Predict RPC call with internal server error."""
    # Patch run_model to raise a generic Exception
    with mock.patch("server.run_model", side_effect=Exception("Simulated failure")):
        with grpc.insecure_channel(grpc_server) as channel:
            stub = anops_pb2_grpc.AnOpsStub(channel)
            request = anops_pb2.PredictRequest(input_data="test input")
            with pytest.raises(grpc.RpcError) as rpc_error:
                stub.Predict(request)
            assert rpc_error.value.code() == grpc.StatusCode.INTERNAL
            assert "Internal server error" in rpc_error.value.details()


@pytest.mark.asyncio
async def test_predict_internal_error(grpc_server, monkeypatch):
    """Test the Predict RPC call when run_model raises an unexpected Exception."""

    # Patch run_model to raise a generic Exception
    def mock_run_model_error(input_data):
        raise Exception("Simulated internal model error")

    monkeypatch.setattr(server, "run_model", mock_run_model_error)

    async with grpc.aio.insecure_channel(grpc_server) as channel:
        stub = anops_pb2_grpc.AnOpsStub(channel)
        request = anops_pb2.PredictRequest(input_data="trigger error")

        with pytest.raises(grpc.aio.AioRpcError) as excinfo:
            await stub.Predict(request)

        assert excinfo.value.code() == grpc.StatusCode.INTERNAL
        assert "Simulated internal model error" in excinfo.value.details()

    # Restore original function if necessary (though monkeypatch handles scope)
    monkeypatch.undo()


# TODO: Add test for internal server error (e.g., by mocking run_model to raise Exception) - DONE
