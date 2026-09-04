import base64
import json

from fastapi.testclient import TestClient
from crucible.backend.app import app

client = TestClient(app)


def _make_alg_none_token(payload: dict) -> str:
    def b64url(data: bytes) -> str:
        return base64.urlsafe_b64encode(data).rstrip(b"=").decode()

    header = b64url(json.dumps({"alg": "none", "typ": "JWT"}).encode())
    body = b64url(json.dumps(payload).encode())
    return f"{header}.{body}."


def test_jwt_rejects_forged_alg_none_token():
    # A legitimately-issued, properly signed token still works.
    login = client.post("/auth/login", json={"username": "sdet_student", "password": "any"})
    assert login.status_code == 200
    real_token = login.json()["access_token"]

    real_resp = client.get("/auth/me", headers={"Authorization": f"Bearer {real_token}"})
    assert real_resp.status_code == 200

    # A forged token that names its own algorithm as "none" and carries no signature
    # must be rejected -- the server's `algorithms=["HS256"]` allowlist in jwt.decode()
    # should refuse to honor an attacker-chosen verification method (2015 disclosure).
    forged_token = _make_alg_none_token({"sub": "attacker", "role": "admin"})
    forged_resp = client.get("/auth/me", headers={"Authorization": f"Bearer {forged_token}"})
    assert forged_resp.status_code == 401
