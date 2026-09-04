"""
PRODUCTION STORY:
Tesla Kubernetes Crypto-Mining Hijack (2018)
Attackers gained access to an unauthenticated Kubernetes dashboard and compromised containers mounting
the host's /var/run/docker.sock, escalating directly to root privileges on the underlying AWS EC2 cluster.
"""

import requests


def test_deploy_config_validation():
    # Anti-pattern: Only checking that a SAFE config is accepted -- never proving the
    # deploy-config validator actually catches the dangerous case it exists to catch.
    # TODO: Also send a config that mounts /var/run/docker.sock and assert it is
    # rejected with HTTP 403 and error "DOCKER_SOCKET_MOUNT_FORBIDDEN".
    res = requests.post(
        "http://localhost:8081/api/security/validate-deploy-config",
        json={"image": "cherenkov/worker:latest", "volumes": ["/app/data:/app/data"]},
    )
    assert res.status_code == 200
