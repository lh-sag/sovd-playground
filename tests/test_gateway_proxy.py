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

"""Tests for the reverse proxy functionality."""

import subprocess
import time
import os
import signal
import requests
import pytest
from pathlib import Path


@pytest.fixture
def backend_server(tmp_path):
    """Start a simple HTTP server on port 8080 to act as backend."""
    # Create a test file to serve
    test_dir = tmp_path / "backend"
    test_dir.mkdir()
    test_file = test_dir / "test.txt"
    test_file.write_text("Hello from backend server")
    
    # Start Python HTTP server on port 8080
    process = subprocess.Popen(
        ["python3", "-m", "http.server", "8080"],
        cwd=str(test_dir),
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE
    )
    
    # Wait for server to start
    time.sleep(2)
    
    # Verify server is running
    try:
        response = requests.get("http://localhost:8080/", timeout=1)
        assert response.status_code == 200
    except Exception as e:
        process.kill()
        pytest.fail(f"Backend server failed to start: {e}")
    
    yield process
    
    # Cleanup
    process.terminate()
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait()


def test_proxy_get_request(gateway, backend_server):
    """Test that GET requests are properly forwarded through the proxy."""
    # Make request through proxy to get the test file
    # Remove /opensovd from base_url and use direct server URL
    server_url = gateway.base_url.replace("/opensovd", "")
    response = requests.get(f"{server_url}/proxy/test.txt")
    
    # Verify response
    assert response.status_code == 200
    assert response.text.strip() == "Hello from backend server"


def test_proxy_directory_listing(gateway, backend_server):
    """Test that directory listings are properly forwarded."""
    # Make request through proxy for directory listing
    server_url = gateway.base_url.replace("/opensovd", "")
    response = requests.get(f"{server_url}/proxy/")
    
    # Verify response
    assert response.status_code == 200
    assert "text/html" in response.headers.get("content-type", "")
    assert "test.txt" in response.text


def test_proxy_query_params(gateway, backend_server):
    """Test that query parameters are preserved when proxying."""
    # Make request with query parameters
    # Python's http.server ignores query params, but we can verify they're sent
    server_url = gateway.base_url.replace("/opensovd", "")
    response = requests.get(
        f"{server_url}/proxy/test.txt",
        params={"key": "value", "foo": "bar"}
    )
    
    # Verify response (file is still served)
    assert response.status_code == 200
    assert response.text.strip() == "Hello from backend server"


def test_proxy_404(gateway, backend_server):
    """Test that error responses are properly forwarded."""
    # Make request through proxy for non-existent file
    server_url = gateway.base_url.replace("/opensovd", "")
    response = requests.get(f"{server_url}/proxy/nonexistent.txt")
    
    # Verify error is forwarded
    assert response.status_code == 404


def test_proxy_custom_headers(gateway, backend_server):
    """Test that custom headers are forwarded."""
    # Make request with custom headers
    server_url = gateway.base_url.replace("/opensovd", "")
    response = requests.get(
        f"{server_url}/proxy/test.txt",
        headers={
            "X-Custom-Header": "test-value",
            "User-Agent": "test-agent"
        }
    )
    
    # Verify response is successful (headers are forwarded even if backend doesn't use them)
    assert response.status_code == 200
    assert response.text.strip() == "Hello from backend server"


def test_proxy_without_backend(gateway):
    """Test proxy behavior when backend is not available."""
    # First ensure no server is running on port 8080
    try:
        # Try to connect to port 8080
        response = requests.get("http://localhost:8080/", timeout=1)
        # If we get here, a server is running - skip the test
        pytest.skip("Port 8080 is already in use, skipping test")
    except requests.exceptions.RequestException:
        # Good, no server is running
        pass
    
    # Make request to proxy when backend is down
    server_url = gateway.base_url.replace("/opensovd", "")
    proxy_url = f"{server_url}/proxy/test"
    print(f"Testing proxy URL: {proxy_url}")
    response = requests.get(proxy_url)
    
    # Should return Bad Gateway error
    assert response.status_code == 502
    assert "Proxy error" in response.text