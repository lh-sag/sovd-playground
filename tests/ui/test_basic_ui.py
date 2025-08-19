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

import pytest
import requests
from playwright.sync_api import Page, expect


@pytest.fixture
def gateway_features():
    return ["ui"]


def test_page_loads(page: Page, gateway_url: str, check_playwright_installed):
    """Test that the UI page loads successfully at /ui path."""
    # Extract base URL without the /opensovd path
    base_url = gateway_url.replace("/opensovd", "")

    # Check if UI feature is enabled by testing if /ui endpoint exists
    ui_url = f"{base_url}/ui"
    try:
        response = requests.get(ui_url, timeout=2)
        if response.status_code == 404:
            pytest.skip("UI feature not enabled. Run with --osovd-gateway-features='ui' to enable")
    except requests.RequestException:
        pytest.skip("Could not reach UI endpoint. UI feature may not be enabled")

    # Navigate to the UI endpoint at /ui
    page.goto(ui_url)

    # Wait for the Vue app to mount
    page.wait_for_selector("#app", timeout=5000)

    # Verify the page title
    expect(page).to_have_title("OpenSOVD")

    # Verify the app element is visible
    expect(page.locator("#app")).to_be_visible()

    # Verify the topbar is present
    expect(page.locator(".topbar")).to_be_visible()

    # Verify the sidebar is present
    expect(page.locator(".sidebar")).to_be_visible()

    # Verify the main panel is present
    expect(page.locator(".main-panel")).to_be_visible()
