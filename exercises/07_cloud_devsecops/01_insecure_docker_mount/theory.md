# Theoretical Context: Docker Socket Mount Vulnerabilities and Container Breakout

## Production Incident: Tesla Kubernetes Crypto-Mining Hijack (2018)

In February 2018, cybersecurity researchers discovered that an unauthenticated Kubernetes dashboard deployed within Tesla's Amazon Web Services (AWS) infrastructure had been compromised by malicious actors. The attackers accessed a container running on a worker node that had the host's UNIX Docker socket (`/var/run/docker.sock`) bind-mounted into the container filesystem. Utilizing standard Docker API commands directly over the mounted socket, the attackers escaped the container sandbox, spawned privileged sibling containers with root access to the host's underlying storage and compute resources, exfiltrated sensitive telemetry and telemetry credentials, and installed cryptocurrency mining software across the cluster. The breach highlighted the immense security danger of exposing host container runtime sockets to application workloads.

## The Underlying Mechanism

Containerization relies on Linux kernel primitives (namespaces and control groups - cgroups) to provide process isolation. However, the Docker daemon runs with root privileges on the host system:

1. **The Docker Daemon Socket**: The UNIX domain socket `/var/run/docker.sock` is the primary communication channel for the Docker daemon API. Any process capable of writing to this socket possesses effective root control over the host operating system.
2. **The Container Escape Mechanism**:
   - When a container mounts `/var/run/docker.sock`, any process inside the container can invoke the Docker API.
   - An attacker executes: `docker run -v /:/host -it alpine chroot /host`
   - This command instructs the host Docker daemon to create a new container that mounts the host's entire root filesystem (`/`) into `/host`, granting the attacker full read/write root execution on the host machine.
3. **Hardened DevSecOps Remediation**:
   - **Never Mount the Docker Socket**: Use isolated builder engines (e.g., Kaniko, Buildah) that build container images in unprivileged user space without a Docker daemon.
   - **Run as Non-Root User**: Configure `USER 10001:10001` in Dockerfile.
   - **Read-Only Root Filesystems**: Mount container root filesystems as read-only (`--read-only`).
   - **Drop Linux Capabilities**: Drop `ALL` capabilities and explicitly add only necessary privileges (`--cap-drop=ALL`).

```
[Insecure Anti-Pattern: Docker Socket Bind Mount Escape]
┌────────────────────────────────────────────────────────┐
│ Compromised App Container (Non-root user)              │
│  └── Mounts: /var/run/docker.sock                      │
│        │                                               │
│        ▼ (Sends Docker API command to spawn container) │
└────────┼───────────────────────────────────────────────┘
         ▼
┌────────────────────────────────────────────────────────┐
│ Host Docker Daemon (Runs as ROOT on Host OS)           │
│  └── Spawns Privileged Root Container with -v /:/host   │
│        │                                               │
│        ▼                                               │
│  TOTAL HOST COMPROMISE & ARBITRARY ROOT EXECUTION ❌    │
└────────────────────────────────────────────────────────┘

[Resilient DevSecOps Pattern: Rootless Unprivileged Sandbox]
┌────────────────────────────────────────────────────────┐
│ App Container (USER 10001, Read-Only rootfs)           │
│  ├── No host socket mounts (/var/run/docker.sock ❌)   │
│  ├── Capabilities Dropped: ALL                         │
│  └── Seccomp & AppArmor Profiles Active                │
└────────────────────────────────────────────────────────┘
            │
            ▼
Container Breakout Prevented — Sandbox Secure ✅
```

Auditing container runtime mounts and enforcing rootless security postures prevents lateral privilege escalation and secures cloud-native infrastructure.

You will now simulate this in the Crucible: audit Dockerfile and container mount configurations to detect insecure Docker socket exposures and enforce hardened container isolation.
