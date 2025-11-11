# SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
# SPDX-License-Identifier: Apache-2.0

import time
from pathlib import Path
from typing import Any

import jwt
import pytest
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric import rsa

_RSA_KEY_SIZE = 2048
_RSA_PUBLIC_EXPONENT = 65537
_TOKEN_EXPIRY_SECONDS = 3600
_TOKEN_EXPIRED_SECONDS_AGO = 3600
_TOKEN_ISSUED_SECONDS_AGO = 7200

_DEFAULT_JWT_PAYLOAD: dict[str, Any] = {
    "sub": "test-user",
    "aud": "sovd",
    "iss": "sovd-test",
}


class JWTHelper:
    def __init__(self, private_pem: bytes, public_pem_path: Path):
        self.private_pem = private_pem
        self.public_pem_path = public_pem_path

    def create_jwt(self, payload: dict[str, Any] | None = None) -> str:
        now = int(time.time())
        jwt_payload = _DEFAULT_JWT_PAYLOAD | {"exp": now + _TOKEN_EXPIRY_SECONDS, "iat": now} | (payload or {})
        return jwt.encode(jwt_payload, self.private_pem, algorithm="RS256")


@pytest.fixture(scope="module")
def jwt_keys(tmp_path_factory) -> JWTHelper:
    tmp_dir = tmp_path_factory.mktemp("jwt_keys")

    private_key = rsa.generate_private_key(
        public_exponent=_RSA_PUBLIC_EXPONENT,
        key_size=_RSA_KEY_SIZE,
    )
    public_key = private_key.public_key()

    public_pem_path = tmp_dir / "public_key.pem"
    public_pem = public_key.public_bytes(
        encoding=serialization.Encoding.PEM, format=serialization.PublicFormat.SubjectPublicKeyInfo
    )
    public_pem_path.write_bytes(public_pem)

    private_pem = private_key.private_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PrivateFormat.PKCS8,
        encryption_algorithm=serialization.NoEncryption(),
    )

    return JWTHelper(private_pem, public_pem_path)


@pytest.fixture
def gateway_args(jwt_keys) -> list[str]:
    return ["--url", "http://127.0.0.1:0/sovd", "--auth-jwt", str(jwt_keys.public_pem_path)]


async def test_unprotected_endpoint_without_token(client):
    response = await client.get("/version-info")
    assert response.status_code == 200
    assert "sovd_info" in response.json()


async def test_protected_endpoint_without_token(client):
    response = await client.get("/v1/components")
    assert response.status_code == 401


async def test_protected_endpoint_with_valid_token(client, jwt_keys):
    token = jwt_keys.create_jwt()
    headers = {"Authorization": f"Bearer {token}"}
    response = await client.get("/v1/components", headers=headers)

    assert response.status_code == 200
    assert response.headers.get("content-type") == "application/json"


async def test_protected_endpoint_with_invalid_token(client):
    headers = {"Authorization": "Bearer invalid.token.here"}
    response = await client.get("/v1/components", headers=headers)
    assert response.status_code == 401


async def test_protected_endpoint_with_expired_token(client, jwt_keys):
    now = int(time.time())
    expired_payload = {
        "exp": now - _TOKEN_EXPIRED_SECONDS_AGO,
        "iat": now - _TOKEN_ISSUED_SECONDS_AGO,
    }
    token = jwt_keys.create_jwt(payload=expired_payload)
    headers = {"Authorization": f"Bearer {token}"}
    response = await client.get("/v1/components", headers=headers)
    assert response.status_code == 401
