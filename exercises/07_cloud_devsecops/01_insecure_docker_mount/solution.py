from fastapi.testclient import TestClient
from crucible.backend.app import app

client = TestClient(app)


def test_deploy_config_rejects_docker_socket_mount():
    # A config without the docker.sock mount is accepted.
    res_safe = client.post(
        "/api/security/validate-deploy-config",
        json={"image": "cherenkov/worker:latest", "volumes": ["/app/data:/app/data"]},
    )
    assert res_safe.status_code == 200
    assert res_safe.json()["status"] == "valid"

    # A config mounting the host Docker socket must be rejected -- it grants
    # root-equivalent host control (the exact vector Tesla's attackers used in 2018).
    res_unsafe = client.post(
        "/api/security/validate-deploy-config",
        json={"image": "ubuntu:latest", "volumes": ["/var/run/docker.sock:/var/run/docker.sock"]},
    )
    assert res_unsafe.status_code == 403
    assert res_unsafe.json()["error"] == "DOCKER_SOCKET_MOUNT_FORBIDDEN"
