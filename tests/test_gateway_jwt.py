# SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
# SPDX-License-Identifier: Apache-2.0
#

"""
Tests for JWT authentication.
"""

import time
from pathlib import Path
from typing import Any

import jwt
import pytest
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric import rsa

# JWT Configuration constants
_RSA_KEY_SIZE = 2048
_RSA_PUBLIC_EXPONENT = 65537
_TOKEN_EXPIRY_SECONDS = 3600
_TOKEN_EXPIRED_SECONDS_AGO = 3600  # 1 hour ago
_TOKEN_ISSUED_SECONDS_AGO = 7200  # 2 hours ago

# Default JWT payload
_DEFAULT_JWT_PAYLOAD: dict[str, Any] = {
    "sub": "test-user",
    "aud": "sovd",
    "iss": "sovd-test",
}


class JWTHelper:
    """Helper class for generating and managing JWT tokens in tests."""

    def __init__(self, private_pem: bytes, public_pem_path: Path):
        self.private_pem = private_pem
        self.public_pem_path = public_pem_path

    def create_jwt(self, payload: dict[str, Any] | None = None) -> str:
        """
        Create a JWT token with the given payload.

        Args:
            payload: Optional additional payload fields to include in the token.
                    Will be merged with default payload and timestamps.

        Returns:
            Encoded JWT token string
        """
        jwt_payload: dict[str, Any] = _DEFAULT_JWT_PAYLOAD.copy()
        jwt_payload.update(
            {
                "exp": int(time.time()) + _TOKEN_EXPIRY_SECONDS,
                "iat": int(time.time()),
            }
        )
        if payload:
            jwt_payload.update(payload)

        return jwt.encode(jwt_payload, self.private_pem, algorithm="RS256")


@pytest.fixture(scope="module")
def jwt_keys(tmp_path_factory) -> JWTHelper:
    """Generate RSA key pair for JWT testing."""
    tmp_dir = tmp_path_factory.mktemp("jwt_keys")

    # Generate RSA key pair
    private_key = rsa.generate_private_key(
        public_exponent=_RSA_PUBLIC_EXPONENT,
        key_size=_RSA_KEY_SIZE,
    )
    public_key = private_key.public_key()

    # Save public key as PEM
    public_pem_path = tmp_dir / "public_key.pem"
    public_pem = public_key.public_bytes(
        encoding=serialization.Encoding.PEM, format=serialization.PublicFormat.SubjectPublicKeyInfo
    )
    public_pem_path.write_bytes(public_pem)

    # Get private key as PEM for token generation
    private_pem = private_key.private_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PrivateFormat.PKCS8,
        encryption_algorithm=serialization.NoEncryption(),
    )

    return JWTHelper(private_pem, public_pem_path)


@pytest.fixture
def gateway_args(jwt_keys) -> list[str]:
    """Get default additional arguments for sovd-gateway."""
    return ["--url", "http://127.0.0.1:0/sovd", "--auth-jwt", str(jwt_keys.public_pem_path)]


async def test_unprotected_endpoint_without_token(client):
    """Test that unprotected endpoints work without authentication."""
    response = await client.get("/version-info")
    assert response.status_code == 200
    assert "sovd_info" in response.json()


async def test_protected_endpoint_without_token(client):
    """Test that protected endpoints reject requests without tokens."""
    response = await client.get("/v1/components")
    assert response.status_code == 401


async def test_protected_endpoint_with_valid_token(client, jwt_keys):
    """
    Test that protected endpoints accept valid JWT tokens.

    Note: This test accepts both 200 (components exist) and 404 (no components)
    as valid responses, since the goal is to verify JWT authentication works,
    not data availability. Both responses indicate successful authentication.
    A 401 would indicate auth failure.
    """
    token = jwt_keys.create_jwt()
    headers = {"Authorization": f"Bearer {token}"}
    response = await client.get("/v1/components", headers=headers)

    # Authentication passed if we get 200 (data found) or 404 (no data)
    # Authentication failed would return 401
    assert response.status_code == 200, (
        f"Expected 200 or 404 (auth success), got {response.status_code} (possibly auth failure)"
    )

    # Verify we got a proper JSON response (not an error page)
    assert response.headers.get("content-type") == "application/json"


async def test_protected_endpoint_with_invalid_token(client):
    """Test that protected endpoints reject invalid tokens."""
    headers = {"Authorization": "Bearer invalid.token.here"}
    response = await client.get("/v1/components", headers=headers)
    assert response.status_code == 401


async def test_protected_endpoint_with_expired_token(client, jwt_keys):
    """Test that protected endpoints reject expired tokens."""
    expired_payload = {
        "exp": int(time.time()) - _TOKEN_EXPIRED_SECONDS_AGO,
        "iat": int(time.time()) - _TOKEN_ISSUED_SECONDS_AGO,
    }
    token = jwt_keys.create_jwt(payload=expired_payload)
    headers = {"Authorization": f"Bearer {token}"}
    response = await client.get("/v1/components", headers=headers)
    assert response.status_code == 401
