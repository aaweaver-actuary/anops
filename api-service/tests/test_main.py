import pytest
from fastapi.testclient import TestClient
import main
from unittest.mock import patch, AsyncMock

client = TestClient(main.app)


def test_health_check():
    response = client.get("/health")
    assert response.status_code == 200
    assert response.json() == {"status": "ok"}


@pytest.mark.asyncio
@patch("main.anops_pb2_grpc.AnOpsStub")
async def test_predict_success(mock_stub):
    # Mock the gRPC stub's Predict method
    mock_predict = AsyncMock()
    mock_predict.return_value = main.anops_pb2.PredictResponse(
        output_data="MOCKED OUTPUT"
    )
    mock_stub.return_value.Predict = mock_predict

    response = await client.post("/predict", json={"input_data": "test input"})
    assert response.status_code == 200
    assert response.json() == {"output_data": "MOCKED OUTPUT"}
    mock_predict.assert_awaited_once()


@pytest.mark.asyncio
@patch("main.anops_pb2_grpc.AnOpsStub")
async def test_predict_invalid_argument(mock_stub):
    # Simulate gRPC INVALID_ARGUMENT error
    mock_predict = AsyncMock()
    error = main.grpc.aio.AioRpcError(
        main.grpc.StatusCode.INVALID_ARGUMENT, "Invalid input!"
    )
    mock_predict.side_effect = error
    mock_stub.return_value.Predict = mock_predict

    response = await client.post("/predict", json={"input_data": ""})
    assert response.status_code == 400
    assert "Invalid input" in response.json()["detail"]


@pytest.mark.asyncio
@patch("main.anops_pb2_grpc.AnOpsStub")
async def test_predict_service_unavailable(mock_stub):
    # Simulate gRPC UNAVAILABLE error
    mock_predict = AsyncMock()
    error = main.grpc.aio.AioRpcError(main.grpc.StatusCode.UNAVAILABLE, "Service down")
    mock_predict.side_effect = error
    mock_stub.return_value.Predict = mock_predict

    response = await client.post("/predict", json={"input_data": "test"})
    assert response.status_code == 503
    assert "unavailable" in response.json()["detail"].lower()


@pytest.mark.asyncio
@patch("main.anops_pb2_grpc.AnOpsStub")
async def test_predict_internal_error(mock_stub):
    # Simulate generic gRPC error
    mock_predict = AsyncMock()
    error = main.grpc.aio.AioRpcError(main.grpc.StatusCode.INTERNAL, "Internal error")
    mock_predict.side_effect = error
    mock_stub.return_value.Predict = mock_predict

    response = await client.post("/predict", json={"input_data": "test"})
    assert response.status_code == 500
    assert "gRPC error" in response.json()["detail"]


@pytest.mark.asyncio
@patch("main.anops_pb2_grpc.AnOpsStub")
async def test_predict_unexpected_exception(mock_stub):
    # Simulate unexpected exception
    mock_predict = AsyncMock()
    mock_predict.side_effect = Exception("Unexpected!")
    mock_stub.return_value.Predict = mock_predict

    response = await client.post("/predict", json={"input_data": "test"})
    assert response.status_code == 500
    assert "Internal server error" in response.json()["detail"]


def test_predict_malformed_request():
    # Missing required field 'input_data'
    response = client.post("/predict", json={})
    assert response.status_code == 422
    assert "input_data" in response.text


def test_predict_env_var(monkeypatch):
    # Patch the environment variable and reload the app
    monkeypatch.setenv("MODEL_SERVICE_URL", "mockhost:12345")
    # The app should pick up the env var (this is a basic check, deeper checks require more refactoring)
    assert main.MODEL_SERVICE_URL == "mockhost:12345"
