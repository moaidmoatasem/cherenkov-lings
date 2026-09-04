# Hints: Drill 01 - Insecure Docker Mount

## Hint 1 (Architectural Nudge)
Mounting `/var/run/docker.sock` into a container gives that container full root access to the host's Docker daemon -- equivalent to running it as root on the host itself. A test that only checks the safe path never proves the validator actually blocks the dangerous one.

## Hint 2 (API Pattern)
Send a second request to `/api/security/validate-deploy-config` whose `volumes` list includes `"/var/run/docker.sock:/var/run/docker.sock"`, and assert on HTTP 403 with `error: "DOCKER_SOCKET_MOUNT_FORBIDDEN"`.

## Hint 3 (Code Diff)
```diff
+ res_unsafe = requests.post(
+     "http://localhost:8081/api/security/validate-deploy-config",
+     json={"image": "ubuntu:latest", "volumes": ["/var/run/docker.sock:/var/run/docker.sock"]},
+ )
+ assert res_unsafe.status_code == 403
+ assert res_unsafe.json()["error"] == "DOCKER_SOCKET_MOUNT_FORBIDDEN"
```
