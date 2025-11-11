# SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
# SPDX-License-Identifier: Apache-2.0


async def test_start_stop(gateway):
    assert gateway.is_running()


async def test_version(client, gateway_url):
    response = await client.get("/version-info")
    assert response.status_code == 200

    data = response.json()
    assert "sovd_info" in data
    assert len(data["sovd_info"]) >= 1
    assert data["sovd_info"][0]["base_uri"] == f"{gateway_url}/v1"


async def test_components(client):
    response = await client.get("/v1/components")
    assert response.status_code == 200

    data = response.json()
    assert "items" in data and len(data["items"]) >= 1

    for item in data["items"]:
        response = await client.get(f"/v1/components/{item['id']}")
        assert response.status_code == 200
