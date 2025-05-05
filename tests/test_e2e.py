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


if __name__ == "__main__":
    main()
