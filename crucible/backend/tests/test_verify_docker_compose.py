"""Automated test harness verifying Micro-Crucible Docker Compose Expansion (R3).

Validates:
1. `docker compose config` syntax, schema validity, and exit code 0.
2. Service inventory (backend, frontend, kafka, otel-collector).
3. Host port mappings and uniqueness (zero port collisions).
4. Kafka KRaft mode configuration (zero ZooKeeper dependency, dual listeners, quorum, CLI healthcheck).
5. OpenTelemetry Collector configuration (OTLP grpc/http, CORS, batch processor, debug exporter, healthcheck extension).
6. Graceful live container startup or environment skip check.
"""

import socket
import subprocess
import time
from pathlib import Path
import pytest
import yaml

ROOT_DIR = Path(__file__).resolve().parents[3]
COMPOSE_FILE = ROOT_DIR / "docker-compose.yml"
OTEL_CONFIG_FILE = ROOT_DIR / "otel-collector-config.yaml"


def test_compose_and_otel_files_exist():
    """Verify docker-compose.yml and otel-collector-config.yaml exist in the workspace root."""
    assert COMPOSE_FILE.is_file(), f"docker-compose.yml not found at {COMPOSE_FILE}"
    assert OTEL_CONFIG_FILE.is_file(), f"otel-collector-config.yaml not found at {OTEL_CONFIG_FILE}"


def test_docker_compose_config_command():
    """Verify `docker compose config` executes cleanly with exit code 0 and valid YAML output."""
    cmd = ["docker", "compose", "-f", str(COMPOSE_FILE), "config"]
    proc = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT_DIR))
    assert proc.returncode == 0, f"docker compose config failed (exit code {proc.returncode}):\n{proc.stderr}"

    # Verify stdout parses as valid YAML
    parsed = yaml.safe_load(proc.stdout)
    assert isinstance(parsed, dict), "docker compose config output did not parse as a dict"
    assert "services" in parsed, "Parsed compose config missing 'services' key"


def test_compose_structure_and_services():
    """Verify parsed YAML contains all four required services attached to crucible-network."""
    with open(COMPOSE_FILE, "r", encoding="utf-8") as f:
        data = yaml.safe_load(f)

    services = data.get("services", {})
    required = {"backend", "frontend", "kafka", "otel-collector"}
    assert required.issubset(services.keys()), f"Missing required services: {required - set(services.keys())}"

    # Verify bridge network definition
    networks = data.get("networks", {})
    assert "crucible-network" in networks, "Missing crucible-network definition"
    net_cfg = networks["crucible-network"]
    if isinstance(net_cfg, dict):
        assert net_cfg.get("driver") == "bridge", f"crucible-network driver should be bridge, got {net_cfg.get('driver')}"

    # Verify all services are attached to crucible-network
    for svc in required:
        svc_networks = services[svc].get("networks", [])
        assert "crucible-network" in svc_networks, f"Service {svc} is not attached to crucible-network"

    # Verify backend environment configuration
    backend_env = services["backend"].get("environment", {})
    if isinstance(backend_env, list):
        backend_env = dict(item.split("=", 1) for item in backend_env)

    assert backend_env.get("KAFKA_BOOTSTRAP_SERVERS") == "kafka:29092", (
        f"KAFKA_BOOTSTRAP_SERVERS in backend should be kafka:29092, got {backend_env.get('KAFKA_BOOTSTRAP_SERVERS')}"
    )
    assert backend_env.get("OTEL_EXPORTER_OTLP_ENDPOINT") == "http://otel-collector:4318", (
        f"OTEL_EXPORTER_OTLP_ENDPOINT in backend should be http://otel-collector:4318, got {backend_env.get('OTEL_EXPORTER_OTLP_ENDPOINT')}"
    )


def test_port_uniqueness_and_mappings():
    """Verify no host port collisions occur and all specified ports are mapped."""
    with open(COMPOSE_FILE, "r", encoding="utf-8") as f:
        data = yaml.safe_load(f)

    services = data.get("services", {})
    assigned_ports = {}

    for svc_name, svc_cfg in services.items():
        ports = svc_cfg.get("ports", [])
        for p in ports:
            # Handle string "8081:8081", int, or dict (long syntax)
            if isinstance(p, dict):
                host_port = str(p.get("published"))
            else:
                host_port = str(p).split(":")[0]
            assert host_port not in assigned_ports, (
                f"Port collision on host port {host_port}: claimed by {assigned_ports[host_port]} and {svc_name}"
            )
            assigned_ports[host_port] = svc_name

    expected_ports = {
        "8081": "backend",
        "8080": "frontend",
        "9092": "kafka",
        "4317": "otel-collector",
        "4318": "otel-collector",
        "8888": "otel-collector",
        "13133": "otel-collector",
    }
    for port, svc in expected_ports.items():
        assert port in assigned_ports, f"Expected port {port} for service {svc} not published"
        assert assigned_ports[port] == svc, f"Port {port} expected for {svc}, but found {assigned_ports[port]}"


def test_kafka_kraft_configuration():
    """Verify Kafka broker satisfies KRaft mode requirements with zero ZooKeeper dependency."""
    with open(COMPOSE_FILE, "r", encoding="utf-8") as f:
        data = yaml.safe_load(f)

    # Assure ZooKeeper is absent
    assert "zookeeper" not in data.get("services", {}), "Found unexpected ZooKeeper service in KRaft compose file"

    kafka_cfg = data["services"]["kafka"]
    assert "apache/kafka" in kafka_cfg.get("image", ""), f"Unexpected kafka image: {kafka_cfg.get('image')}"

    env = kafka_cfg.get("environment", {})
    if isinstance(env, list):
        env = dict(item.split("=", 1) for item in env)

    # KRaft node roles
    process_roles = str(env.get("KAFKA_PROCESS_ROLES", ""))
    assert "broker" in process_roles, "KAFKA_PROCESS_ROLES missing broker"
    assert "controller" in process_roles, "KAFKA_PROCESS_ROLES missing controller"

    # Dual listeners and quorum
    assert "KAFKA_NODE_ID" in env, "Missing KAFKA_NODE_ID"
    listeners = env.get("KAFKA_LISTENERS", "")
    assert "29092" in listeners, "Internal listener 29092 missing in KAFKA_LISTENERS"
    assert "9092" in listeners, "Host listener 9092 missing in KAFKA_LISTENERS"
    assert "9093" in listeners, "Controller listener 9093 missing in KAFKA_LISTENERS"

    advertised = env.get("KAFKA_ADVERTISED_LISTENERS", "")
    assert "kafka:29092" in advertised, "Internal advertised listener kafka:29092 missing"
    assert "localhost:9092" in advertised, "Host advertised listener localhost:9092 missing"

    quorum = env.get("KAFKA_CONTROLLER_QUORUM_VOTERS", "")
    assert "kafka:9093" in quorum, "Controller quorum voter missing kafka:9093"

    assert str(env.get("KAFKA_NUM_PARTITIONS")) == "1", "KAFKA_NUM_PARTITIONS must be 1"

    # CLI healthcheck
    healthcheck = kafka_cfg.get("healthcheck", {})
    hc_test = healthcheck.get("test", [])
    hc_str = " ".join(hc_test) if isinstance(hc_test, list) else str(hc_test)
    assert "/opt/kafka/bin/kafka-broker-api-versions.sh" in hc_str, (
        f"Kafka healthcheck test does not use kafka-broker-api-versions.sh: {hc_str}"
    )


def test_otel_collector_configuration():
    """Verify OTel Collector config file defines OTLP receivers, CORS, batch processor, debug exporter, and healthcheck."""
    with open(OTEL_CONFIG_FILE, "r", encoding="utf-8") as f:
        otel_cfg = yaml.safe_load(f)

    # Receivers: OTLP gRPC (4317) and HTTP (4318) with CORS
    receivers = otel_cfg.get("receivers", {})
    assert "otlp" in receivers, "Missing otlp receiver"
    otlp = receivers["otlp"].get("protocols", {})
    assert "grpc" in otlp, "Missing gRPC protocol in otlp receiver"
    assert "http" in otlp, "Missing HTTP protocol in otlp receiver"

    http_cfg = otlp["http"]
    cors_cfg = http_cfg.get("cors", {})
    allowed_origins = cors_cfg.get("allowed_origins", [])
    assert any("8080" in origin for origin in allowed_origins), (
        f"OTel HTTP receiver CORS missing 8080 origin: {allowed_origins}"
    )

    # Processors: batch
    processors = otel_cfg.get("processors", {})
    assert "batch" in processors, "Missing batch processor"

    # Exporters: debug
    exporters = otel_cfg.get("exporters", {})
    assert any(exp in exporters for exp in ("debug", "logging")), (
        f"Missing debug or logging exporter: {list(exporters.keys())}"
    )

    # Extensions: health_check on 13133
    extensions = otel_cfg.get("extensions", {})
    assert "health_check" in extensions, "Missing health_check extension"
    assert "13133" in str(extensions["health_check"].get("endpoint", "")), (
        f"health_check endpoint does not use 13133: {extensions['health_check']}"
    )

    # Service pipelines
    service = otel_cfg.get("service", {})
    assert "health_check" in service.get("extensions", []), "health_check extension not enabled under service.extensions"
    pipelines = service.get("pipelines", {})
    assert "traces" in pipelines, "traces pipeline missing in OTel config"
    assert "metrics" in pipelines, "metrics pipeline missing in OTel config"
    assert "logs" in pipelines, "logs pipeline missing in OTel config"

    # Compose volume mounting
    with open(COMPOSE_FILE, "r", encoding="utf-8") as f:
        compose_data = yaml.safe_load(f)

    otel_svc = compose_data["services"]["otel-collector"]
    assert "otel/opentelemetry-collector-contrib" in otel_svc.get("image", "")
    assert any("otel-collector-config.yaml" in str(v) for v in otel_svc.get("volumes", [])), (
        f"otel-collector missing volume mount for otel-collector-config.yaml: {otel_svc.get('volumes')}"
    )
    assert any("--config" in str(arg) for arg in otel_svc.get("command", [])), (
        f"otel-collector missing --config command flag: {otel_svc.get('command')}"
    )


def test_live_docker_daemon_startup_or_skip():
    """Verify container startup when Docker engine daemon is running, otherwise skip gracefully."""
    check = subprocess.run(["docker", "info"], capture_output=True, text=True)
    if check.returncode != 0:
        pytest.skip(f"Docker engine daemon is not running on host: {check.stderr.strip()[:100]}")

    # Live startup lifecycle
    try:
        up = subprocess.run(
            ["docker", "compose", "-f", str(COMPOSE_FILE), "up", "-d"],
            capture_output=True,
            text=True,
            cwd=str(ROOT_DIR),
        )
        assert up.returncode == 0, f"docker compose up failed: {up.stderr}"

        # Allow containers initial startup
        time.sleep(10)

        # Probe ports for active listeners
        for port in [8081, 8080, 9092, 4317, 4318, 13133]:
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
                s.settimeout(2.0)
                res = s.connect_ex(("127.0.0.1", port))
                assert res == 0, f"Port {port} did not accept connections after compose up"
    finally:
        subprocess.run(
            ["docker", "compose", "-f", str(COMPOSE_FILE), "down", "-v"],
            capture_output=True,
            cwd=str(ROOT_DIR),
        )
