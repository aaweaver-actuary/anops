import subprocess
import time
import requests
import sys

API_URL = "http://localhost:8000/predict"
HEALTH_URL = "http://localhost:8000/health"


def wait_for_health(timeout=60):
    print("Waiting for API service to become healthy...")
    start = time.time()
    while time.time() - start < timeout:
        try:
            resp = requests.get(HEALTH_URL, timeout=2)
            if resp.status_code == 200 and resp.json().get("status") == "ok":
                print("API service is healthy.")
                return True
        except Exception:
            pass
        time.sleep(2)
    print("Timed out waiting for API service health.")
    return False


def main():
    print("Bringing up services with docker-compose...")
    up = subprocess.run(["docker-compose", "up", "--build", "-d"], capture_output=True)
    if up.returncode != 0:
        print("docker-compose up failed:", up.stderr.decode())
        sys.exit(1)

    try:
        if not wait_for_health():
            raise RuntimeError("API service did not become healthy.")

        print("Sending test request to /predict...")
        resp = requests.post(API_URL, json={"input_data": "hello world"}, timeout=5)
        print("Response status:", resp.status_code)
        print("Response body:", resp.text)
        assert resp.status_code == 200, f"Expected 200, got {resp.status_code}"
        data = resp.json()
        assert "output_data" in data, "Missing output_data in response"
        assert "HELLO WORLD" in data["output_data"], (
            f"Unexpected output: {data['output_data']}"
        )
        print("End-to-end test PASSED.")
    finally:
        print("Tearing down services...")
        subprocess.run(["docker-compose", "down"], capture_output=True)


def test_api_when_model_service_down():
    print("Bringing up only api-service (model-service down)...")
    # Start only api-service, do not start model-service
    up = subprocess.run(
        ["docker-compose", "up", "--build", "-d", "api-service"], capture_output=True
    )
    if up.returncode != 0:
        print("docker-compose up api-service failed:", up.stderr.decode())
        assert False, "Failed to start api-service only"
    try:
        if not wait_for_health():
            print(
                "API service did not become healthy (as expected if model-service is required)."
            )
            return
        print("Sending request to /predict with model-service down...")
        resp = requests.post(API_URL, json={"input_data": "test"}, timeout=5)
        print("Response status:", resp.status_code)
        print("Response body:", resp.text)
        assert resp.status_code in (503, 500), (
            f"Expected 503/500, got {resp.status_code}"
        )
        assert "unavailable" in resp.text.lower() or "error" in resp.text.lower()
        print("Negative-path test PASSED.")
    finally:
        print("Tearing down services...")
        subprocess.run(["docker-compose", "down"], capture_output=True)


if __name__ == "__main__":
    main()
