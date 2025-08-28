#
# Copyright (c) 2025 The Contributors to Eclipse OpenSOVD.
#
# See the NOTICE file(s) distributed with this work for additional
# information regarding copyright ownership.
#
# This program and the accompanying materials are made available under the
# terms of the Apache License Version 2.0 which is available at
# https://www.apache.org/licenses/LICENSE-2.0
#
# SPDX-License-Identifier: Apache-2.0
#

"""
Tests for JWT authentication.
"""

import time
import pytest
import requests
import jwt
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography.hazmat.primitives import serialization

# JWT Configuration
RSA_KEY_SIZE = 2048
RSA_PUBLIC_EXPONENT = 65537
TOKEN_EXPIRY_SECONDS = 3600

# Default JWT payload
DEFAULT_JWT_PAYLOAD = {
    "sub": "test-user",
    "aud": "opensovd",
    "iss": "opensovd-test",
}


@pytest.fixture(scope="module")
def jwt_keys(tmp_path_factory):
    """Generate RSA key pair for JWT testing."""
    tmp_dir = tmp_path_factory.mktemp("jwt_keys")
    
    # Generate RSA key pair
    private_key = rsa.generate_private_key(
        public_exponent=RSA_PUBLIC_EXPONENT,
        key_size=RSA_KEY_SIZE,
    )
    public_key = private_key.public_key()
    
    # Save public key as PEM
    public_pem_path = tmp_dir / "public_key.pem"
    public_pem = public_key.public_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PublicFormat.SubjectPublicKeyInfo
    )
    public_pem_path.write_bytes(public_pem)
    
    # Get private key as PEM for token generation
    private_pem = private_key.private_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PrivateFormat.PKCS8,
        encryption_algorithm=serialization.NoEncryption()
    )
    
    class JWTHelper:
        def __init__(self, private_pem, public_pem_path):
            self.private_pem = private_pem
            self.public_pem_path = public_pem_path
            
        def create_jwt(self, payload=None):
            jwt_payload = DEFAULT_JWT_PAYLOAD.copy()
            jwt_payload.update({
                "exp": int(time.time()) + TOKEN_EXPIRY_SECONDS,
                "iat": int(time.time()),
            })
            if payload:
                jwt_payload.update(payload)
            
            return jwt.encode(
                jwt_payload,
                self.private_pem,
                algorithm="RS256"
            )
    
    return JWTHelper(private_pem, public_pem_path)


@pytest.fixture(scope="module")
def gateway_args(jwt_keys) -> list[str]:
    """Get default additional arguments for osovd-gateway."""
    return [
        "--url", "http://127.0.0.1:9000/opensovd",
        "--auth-jwt", str(jwt_keys.public_pem_path)
    ]


def test_unprotected_endpoint_without_token(gateway):
    """Test that unprotected endpoints work without authentication."""
    headers = {"Content-Type": "application/json"}
    response = requests.get(f"{gateway.base_url}/version-info", headers=headers)
    assert response.status_code == 200
    assert "sovd_info" in response.json()


def test_protected_endpoint_without_token(gateway):
    """Test that protected endpoints reject requests without tokens."""
    headers = {"Content-Type": "application/json"}
    response = requests.get(f"{gateway.base_url}/v1/components", headers=headers)
    assert response.status_code == 401


def test_protected_endpoint_with_valid_token(gateway, jwt_keys):
    """Test that protected endpoints accept valid JWT tokens."""
    token = jwt_keys.create_jwt()
    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json"
    }
    response = requests.get(f"{gateway.base_url}/v1/components", headers=headers)
    # 404 is expected when no components exist, but it means auth passed
    assert response.status_code in [200, 404]
    # If we got through auth, we should get a JSON response
    assert response.headers.get("content-type") == "application/json"


def test_protected_endpoint_with_invalid_token(gateway):
    """Test that protected endpoints reject invalid tokens."""
    headers = {
        "Authorization": "Bearer invalid.token.here",
        "Content-Type": "application/json"
    }
    response = requests.get(f"{gateway.base_url}/v1/components", headers=headers)
    assert response.status_code == 401


def test_protected_endpoint_with_expired_token(gateway, jwt_keys):
    """Test that protected endpoints reject expired tokens."""
    expired_payload = {
        "exp": int(time.time()) - 3600,
        "iat": int(time.time()) - 7200
    }
    token = jwt_keys.create_jwt(payload=expired_payload)
    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json"
    }
    response = requests.get(f"{gateway.base_url}/v1/components", headers=headers)
    assert response.status_code == 401