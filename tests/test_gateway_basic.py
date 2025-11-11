# SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
# SPDX-License-Identifier: Apache-2.0
#


async def test_start_stop(gateway):
    assert gateway.is_running()


async def test_version(client, gateway_url):
    response = await client.get("/version-info")
    assert response.status_code == 200
    json = response.json()
    assert "sovd_info" in json
    assert len(json["sovd_info"]) >= 1
    assert "base_uri" in json["sovd_info"][0]
    assert json["sovd_info"][0]["base_uri"] == f"{gateway_url}/v1"


async def test_components(client):
    response = await client.get("/v1/components")
    assert response.status_code == 200
    json = response.json()
    assert "items" in json and len(json["items"]) >= 1
    for item in json["items"]:
        component_id = item["id"]
        response = await client.get(f"/v1/components/{component_id}")
        assert response.status_code == 200
