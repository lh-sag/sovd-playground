# SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
# SPDX-License-Identifier: Apache-2.0
#

import ssl
import sys
from pathlib import Path

import httpx
import pytest

# HTTPS client timeout for certificate-based connections
_HTTPS_CLIENT_TIMEOUT = 10.0


@pytest.fixture
def gateway_args(openssl: dict[str, Path]):
    return [
        "--url",
        "https://localhost:0/sovd",
        "--cert",
        str(openssl["server_cert"]),
        "--key",
        str(openssl["server_key"]),
        "--cacert",
        str(openssl["ca_cert"]),
    ]


@pytest.fixture
def gateway_features():
    return ["openssl"]


@pytest.mark.skipif(sys.platform in ["darwin", "win32"], reason="Not applicable on macOS and Windows")
async def test_start_stop_mtls(gateway, openssl):
    """Test mTLS (mutual TLS) connection to HTTPS gateway with client certificates."""
    assert gateway.is_running()

    # Create custom SSL context for httpx with proper hostname verification
    # Using localhost URL allows hostname checking since cert SAN includes DNS:localhost
    ssl_context = ssl.create_default_context(cafile=str(openssl["ca_cert"]))
    ssl_context.load_cert_chain(str(openssl["client_cert"]), str(openssl["client_key"]))
    ssl_context.check_hostname = True  # Enabled for proper security
    ssl_context.verify_mode = ssl.CERT_REQUIRED

    try:
        async with httpx.AsyncClient(
            verify=ssl_context,
            timeout=_HTTPS_CLIENT_TIMEOUT,
        ) as client:
            response = await client.get(gateway.base_url + "/version-info")
            assert response.status_code == 200, f"Unexpected status code: {response.status_code}"

            # Verify we got a JSON response
            json_response = response.json()
            assert "sovd_info" in json_response, "Response missing sovd_info"

    except (httpx.ConnectError, httpx.RequestError) as e:
        pytest.fail(f"Failed to connect to HTTPS server: {e}")
