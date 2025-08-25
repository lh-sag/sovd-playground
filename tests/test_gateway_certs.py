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

import sys
from pathlib import Path

import pytest
import requests


@pytest.fixture
def gateway_args(openssl: dict[str, Path]):
    return [
        "--url",
        "https://127.0.0.1:0/opensovd",
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


@pytest.mark.skip("UNSUPPORTED")
@pytest.mark.skipif(sys.platform == "darwin", reason="Not applicable on macOS")
def test_start_stop_mtls(gateway, openssl):
    assert gateway.is_running()
    try:
        response = requests.get(
            gateway.base_url + "/version-info",
            cert=(str(openssl["client_cert"]), str(openssl["client_key"])),
            verify=str(openssl["ca_cert"]),
            headers={"content-type": "application/json"},
            timeout=10,
        )
        assert response.status_code == 200, f"Unexpected status code: {response.status_code}"

    except (requests.exceptions.SSLError, requests.exceptions.ConnectionError) as e:
        pytest.fail(f"Failed to connect to HTTPS server: {e}")
