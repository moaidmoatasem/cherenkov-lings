# Hints: Drill 01 - Insecure Docker Mount

## Hint 1 (Architectural Nudge)
Mounting /var/run/docker.sock into a container gives the container full root access to the host machine's Docker daemon. This is equivalent to running the container as root and is a major security vulnerability.

## Hint 2 (Code Diff)
Remove the docker.sock mount from the 'volumes' array.
