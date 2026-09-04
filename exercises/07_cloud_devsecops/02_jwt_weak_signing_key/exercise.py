"""
PRODUCTION STORY:
The 'alg: none' bypass in JWT libraries (disclosed 2015)
Libraries across several languages let the token name the algorithm used to verify it. Setting the
header to {"alg": "none"} and dropping the signature produced a token those servers accepted as valid,
so any claim -- including {"role": "admin"} -- could be forged without knowing a key.
"""

import base64
import json

import requests


def _make_alg_none_token(payload: dict) -> str:
    def b64url(data: bytes) -> str:
        return base64.urlsafe_b64encode(data).rstrip(b"=").decode()

    header = b64url(json.dumps({"alg": "none", "typ": "JWT"}).encode())
    body = b64url(json.dumps(payload).encode())
    return f"{header}.{body}."


def test_jwt_algorithm_validation():
    # Anti-pattern: Only proving a legitimately-issued token works -- never proving
    # a forged token naming its own algorithm as "none" is rejected.
    # TODO: Craft a token via _make_alg_none_token() claiming {"role": "admin"}, send
    # it to /auth/me, and assert the server rejects it with HTTP 401.
    login = requests.post(
        "http://localhost:8081/auth/login", json={"username": "sdet_student", "password": "any"}
    )
    token = login.json()["access_token"]
    res = requests.get(
        "http://localhost:8081/auth/me", headers={"Authorization": f"Bearer {token}"}
    )
    assert res.status_code == 200
