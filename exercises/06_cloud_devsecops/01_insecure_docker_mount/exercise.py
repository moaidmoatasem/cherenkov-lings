import pytest

def test_docker_mount_security():
    # Anti-pattern: The container definition mounts docker.sock which is a root escalation risk
    # TODO: Modify this definition to use a safer alternative like rootless Docker or remove the sock mount
    container_def = {
        'image': 'ubuntu:latest',
        'volumes': ['/var/run/docker.sock:/var/run/docker.sock']
    }
    assert '/var/run/docker.sock:/var/run/docker.sock' not in container_def.get('volumes', [])
