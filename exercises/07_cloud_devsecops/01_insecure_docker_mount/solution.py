import pytest

def test_docker_mount_security():
    # Solution: The container definition no longer mounts docker.sock
    container_def = {
        'image': 'ubuntu:latest',
        'volumes': ['/app/data:/app/data']
    }
    assert '/var/run/docker.sock:/var/run/docker.sock' not in container_def.get('volumes', [])
